use crate::constants;
use crate::database_api;
use crate::error::Error;
use crate::error::Result;
use chacha20poly1305::aead::Aead;
use chacha20poly1305::aead::NewAead;
use chacha20poly1305::Key;
use chacha20poly1305::XChaCha20Poly1305;
use chacha20poly1305::XNonce;
use log::warn;
use rand::random;
use rusqlite::Transaction;
use sha2::Digest;
use sha2::Sha256;
use std::fs::create_dir_all;
use std::fs::OpenOptions;
use std::io::Write;
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;
use warp::http::status::StatusCode;

pub fn upload_file(
    tx: &Transaction,
    owner: String,
    expected_sha256: String,
    body: &[u8],
) -> Result<()> {
    if file_exists_on_disk(&owner, &expected_sha256)? {
        // Note that checking once for file existence here is not enough.
        // To prevent TOCTOU attack, we also need to check file existence below.
        // We could avoid doing a check here at all, but we do it to avoid spending CPU power
        // on hash calculation for conflicting files,
        // which are detected by a relatively cheap test.
        return Err(Error {
            code: StatusCode::CONFLICT,
            msg: "File already exists".to_string(),
        });
    };
    validate_hash(&expected_sha256, body)?;

    let key: [u8; 32] = random();
    let key = Key::from_slice(&key);
    let cipher = XChaCha20Poly1305::new(key);

    let nonce: [u8; 24] = rand::random();
    let nonce = XNonce::from_slice(&nonce); // unique per file
    let body = cipher.encrypt(nonce, body)?;
    update_key_and_nonce(tx, key.deref(), nonce.deref(), &expected_sha256)?;

    let file = final_path(&owner, &expected_sha256)?;
    let file = OpenOptions::new().write(true).create_new(true).open(file);
    let mut file = file.map_err(|err| {
        if err.raw_os_error() == Some(libc::EEXIST) {
            Error {
                code: StatusCode::CONFLICT,
                msg: "File already exists".to_string(),
            }
        } else {
            Error {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                msg: format!("Failed to create target file, {}", err),
            }
        }
    })?;
    file.write_all(&body).map_err(|err| Error {
        code: StatusCode::INTERNAL_SERVER_ERROR,
        msg: format!("Failed to write data to target file, {}", err),
    })?;
    Ok(())
}

pub fn get_file(tx: &Transaction, owner: &str, sha256: &str) -> Result<Vec<u8>> {
    let file = final_path(owner, sha256)?;
    let file = std::fs::read(file).map_err(|err| Error {
        code: StatusCode::INTERNAL_SERVER_ERROR,
        msg: format!("Failed to read data from target file, {}", err),
    })?;
    let (key, nonce) = find_key_and_nonce_by_sha256(tx, sha256)?;
    let (key, nonce) = (Key::from_slice(&key), XNonce::from_slice(&nonce));
    let cipher = XChaCha20Poly1305::new(key);
    let plaintext = cipher.decrypt(&nonce, file.as_ref())?;
    Ok(plaintext)
}

fn file_exists_on_disk(owner: &str, sha256: &str) -> Result<bool> {
    let file = final_path(owner, sha256)?;
    Ok(file.exists())
}

fn final_path(owner: &str, sha256: &str) -> Result<PathBuf> {
    let result = files_dir()?;
    let final_dir = result.join(owner).join(FINAL_DIR);
    create_dir_all(&final_dir).map_err(|err| Error {
        code: StatusCode::INTERNAL_SERVER_ERROR,
        msg: format!(
            "Failed to create files directory for owner {}, {}",
            owner, err
        ),
    })?;
    Ok(final_dir.join(sha256))
}

fn validate_hash(expected_sha256: &str, data: &[u8]) -> Result<()> {
    let expected_sha256_vec = hex::decode(&expected_sha256)?;
    let mut real_sha256 = Sha256::new();
    real_sha256.update(data);
    let real_sha256 = real_sha256.finalize();
    if real_sha256.as_slice() != expected_sha256_vec.as_slice() {
        Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!(
                "Expected file sha256 hash differs from calculated {}",
                hex::encode(real_sha256)
            ),
        })
    } else {
        Ok(())
    }
}

/// Update `key` and `nonce` in DB for items that have the given `sha256`
fn update_key_and_nonce(
    tx: &Transaction,
    key: &[u8],
    nonce: &[u8],
    for_sha256: &str,
) -> Result<()> {
    let item_rowids = database_api::search_strings(tx, "sha256", for_sha256)?;
    if item_rowids.is_empty() {
        Err(Error {
            code: StatusCode::NOT_FOUND,
            msg: format!("Item with sha256 {} not found in database", for_sha256),
        })
    } else {
        let key = hex::encode(key);
        let nonce = hex::encode(nonce);
        for item in item_rowids {
            database_api::insert_string(tx, item, "key", &key)?;
            database_api::insert_string(tx, item, "nonce", &nonce)?;
        }
        Ok(())
    }
}

/// Find first `key` and `nonce` pair in the database for an item with the desired `sha256`
fn find_key_and_nonce_by_sha256(tx: &Transaction, sha256: &str) -> Result<(Vec<u8>, Vec<u8>)> {
    let item_rowids = database_api::search_strings(tx, "sha256", sha256)?;
    if let Some(rowid) = item_rowids.first() {
        let mut other_props = database_api::get_strings_for_item(tx, *rowid)?;
        let key = other_props.remove("key").ok_or_else(|| Error {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: format!(
                "Item with required hash {} found, but it does not have a 'key' property",
                sha256
            ),
        })?;
        let nonce = other_props.remove("nonce").ok_or_else(|| Error {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: format!(
                "Item with required hash {} found, but it does not have a 'nonce' property",
                sha256
            ),
        })?;
        let key = hex::decode(key)?;
        let nonce = hex::decode(nonce)?;
        Ok((key, nonce))
    } else {
        return Err(Error {
            code: StatusCode::NOT_FOUND,
            msg: format!("Item with sha256={} not found", sha256),
        });
    }
}

/// Directory where files (e.g. media) are stored
fn files_dir() -> Result<PathBuf> {
    PathBuf::from_str(constants::FILES_DIR).map_err(|err| {
        warn!("Failed to create file upload path {}", constants::FILES_DIR);
        Error {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: format!("Failed to create file upload path, {}", err),
        }
    })
}

/// Directory where fully uploaded and hash-checked files are stored
/// (in future, the files should also be s3-uploaded).
const FINAL_DIR: &str = "final";
