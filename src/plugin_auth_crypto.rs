use crate::api_model::PluginAuth;
// use crate::api_model::PluginAuthData;
use crate::error::Error;
use crate::error::Result;
use crate::global_static;
use chacha20poly1305::aead::Aead;
use chacha20poly1305::aead::NewAead;
use chacha20poly1305::Key;
use chacha20poly1305::XChaCha20Poly1305;
use chacha20poly1305::XNonce;
use warp::hyper::StatusCode;

pub fn generate_xchacha20poly1305_cipher() -> XChaCha20Poly1305 {
    let key: [u8; 32] = rand::random();
    let key = Key::from_slice(&key);
    XChaCha20Poly1305::new(key)
}

// // Given a request that was already authorized and has a `database_key`,
// // create a new PluginAuth to be passed to external Plugins
// pub fn create_plugin_auth(database_key: &str) -> Result<PluginAuth> {
//     let nonce: [u8; 24] = rand::random();
//     let nonce = XNonce::from_slice(&nonce); // MUST be unique
//     let cipher = &global_static::CIPHER;
//     let encrypted = cipher.encrypt(nonce, database_key.as_bytes())?;
//     Ok(PluginAuth {
//         data: PluginAuthData {
//             nonce: hex::encode(nonce),
//             encrypted_permissions: hex::encode(encrypted),
//         },
//     })
// }

/// Given a request that uses PluginAuth, decrypt its `database_key`.
pub fn extract_database_key(plugin_auth: &PluginAuth) -> Result<String> {
    let nonce = hex::decode(&plugin_auth.data.nonce)?;
    let encrypted_permissions = hex::decode(&plugin_auth.data.encrypted_permissions)?;
    if nonce.len() != 24 {
        return Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg: "PluginAuth nonce should be 48 hex characters".to_string(),
        });
    }
    let nonce = XNonce::from_slice(&nonce);
    let cipher = &global_static::CIPHER;
    let decrypted = cipher.decrypt(&nonce, encrypted_permissions.as_ref())?;
    match String::from_utf8(decrypted) {
        Ok(ok) if hex::decode(&ok).is_ok() => Ok(ok),
        Ok(_never_log_this) => Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg: "Incorrect permissions. This should never be possible, please report this"
                .to_string(),
        }),
        Err(_never_log_this) => Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg: "Failed to decrypt permissions, internal non-valid UTF".to_string(),
        }),
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_encrypt_decrypt() {
//         let db_key = "11223344";
//         let auth = create_plugin_auth(db_key).unwrap();
//         assert!(!auth.data.nonce.contains(db_key));
//         assert!(!auth.data.encrypted_permissions.contains(db_key));
//         let decrypted = extract_database_key(&auth).unwrap();
//         assert_eq!(&decrypted, db_key);
//     }
// }
