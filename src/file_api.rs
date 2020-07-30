use crate::configuration;
use crate::error::Error;
use crate::error::Result;
use bytes::Bytes;
use log::warn;
use sha2::Digest;
use sha2::Sha256;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use warp::http::status::StatusCode;

pub fn upload_file(owner: String, expected_sha256: String, body: Bytes) -> Result<()> {
    let expected_sha256_vec = hex::decode(&expected_sha256)?;
    let mut real_sha256 = Sha256::new();
    real_sha256.update(&body);
    let real_sha256 = real_sha256.finalize();
    if real_sha256.as_slice() != expected_sha256_vec.as_slice() {
        return Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg: "Calculated file sha256 hash differs from expected".to_string(),
        });
    };
    let file = file_path(&owner, &expected_sha256)?;
    let file = OpenOptions::new().write(true).create_new(true).open(file);
    let mut file = file.map_err(|err| {
        if err.raw_os_error() == Some(libc::EEXIST) {
            Error {
                code: StatusCode::CONFLICT,
                msg: format!("File already exists"),
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

fn file_path(owner: &str, sha256: &str) -> Result<PathBuf> {
    let result = media_dir()?;
    Ok(result.join(owner).join(sha256))
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
