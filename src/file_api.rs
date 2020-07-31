use crate::configuration;
use crate::error::Error;
use crate::error::Result;
use bytes::Bytes;
use log::warn;
use sha2::Digest;
use sha2::Sha256;
use std::fs::create_dir_all;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use warp::http::status::StatusCode;

pub fn upload_file(owner: String, expected_sha256: String, body: Bytes) -> Result<()> {
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
    }
    let expected_sha256_vec = hex::decode(&expected_sha256)?;
    let mut real_sha256 = Sha256::new();
    real_sha256.update(&body);
    let real_sha256 = real_sha256.finalize();
    if real_sha256.as_slice() != expected_sha256_vec.as_slice() {
        return Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!(
                "Expected file sha256 hash differs from calculated {}",
                hex::encode(real_sha256)
            ),
        });
    };
    let file = file_path(&owner, &expected_sha256)?;
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

pub fn get_file(owner: &str, sha256: &str) -> Result<Vec<u8>> {
    let file = file_path(owner, sha256)?;
    let file = std::fs::read(file).map_err(|err| Error {
        code: StatusCode::INTERNAL_SERVER_ERROR,
        msg: format!("Failed to read data from target file, {}", err),
    })?;
    Ok(file)
}

fn file_exists_on_disk(owner: &str, sha256: &str) -> Result<bool> {
    let file = file_path(owner, sha256)?;
    Ok(file.exists())
}

fn file_path(owner: &str, sha256: &str) -> Result<PathBuf> {
    let result = media_dir()?;
    let owner_dir = result.join(owner);
    create_dir_all(&owner_dir).map_err(|err| Error {
        code: StatusCode::INTERNAL_SERVER_ERROR,
        msg: format!("Failed to create media owner directory, {}", err),
    })?;
    Ok(owner_dir.join(sha256))
}

fn media_dir() -> Result<PathBuf> {
    PathBuf::from_str(configuration::MEDIA_DIR).map_err(|err| {
        warn!(
            "Failed to create file upload path {}",
            configuration::MEDIA_DIR
        );
        Error {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: format!("Failed to create file upload path, {}", err),
        }
    })
}
