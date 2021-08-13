use crate::api_model::PluginAuth;
use crate::api_model::PluginAuthData;
use crate::error::Error;
use crate::error::Result;
use crate::global_static;
use chacha20poly1305::aead::Aead;
use chacha20poly1305::aead::NewAead;
use chacha20poly1305::Key;
use chacha20poly1305::XChaCha20Poly1305;
use chacha20poly1305::XNonce;
use rusqlite::Connection;
use warp::hyper::StatusCode;
use zeroize::Zeroize;

pub fn generate_xchacha20poly1305_cipher() -> XChaCha20Poly1305 {
    let key: [u8; 32] = rand::random();
    let key = Key::from_slice(&key);
    XChaCha20Poly1305::new(key)
}

/// Given a `database_key` from an already authorized request,
/// create a new PluginAuth to be passed to external Plugins.
pub fn create_plugin_auth(database_key: &str) -> Result<PluginAuth> {
    let nonce: [u8; 24] = rand::random();
    let nonce = XNonce::from_slice(&nonce); // MUST be unique
    let cipher = &global_static::CIPHER;
    let database_key: Vec<u8> = hex::decode(database_key)?;
    let encrypted = cipher.encrypt(nonce, database_key.as_slice())?;
    Ok(PluginAuth {
        data: PluginAuthData {
            nonce: hex::encode(nonce),
            encrypted_permissions: hex::encode(encrypted),
        },
    })
}

/// Given a request that uses PluginAuth, decrypt its `database_key`.
pub fn extract_database_key(plugin_auth: &PluginAuth) -> Result<DatabaseKey> {
    if plugin_auth.data.nonce.len() != 48 {
        return Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg: "PluginAuth nonce should be 48 hex characters".to_string(),
        });
    }
    let nonce = hex::decode(&plugin_auth.data.nonce)?;
    let encrypted_permissions = hex::decode(&plugin_auth.data.encrypted_permissions)?;
    let nonce = XNonce::from_slice(&nonce);
    let cipher = &global_static::CIPHER;
    let decrypted = cipher.decrypt(&nonce, encrypted_permissions.as_ref())?;
    if decrypted.len() != 32 && decrypted.len() != 0 {
        return Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!("Key has incorrect length: {}", decrypted.len()),
        });
    }
    Ok(DatabaseKey {
        database_key: hex::encode_upper(&decrypted),
    })
}

/// Database key stored in raw String format (as supplied to sqlcipher).
pub struct DatabaseKey {
    /// PRIVATE key (intentionally not public)!
    /// The wrapper struct only exposes "safe" methods to work with the key.
    database_key: String,
}

impl Drop for DatabaseKey {
    fn drop(&mut self) {
        self.database_key.zeroize();
    }
}

impl std::fmt::Debug for DatabaseKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DatabaseKey(hidden_fields)")
    }
}

impl DatabaseKey {
    /// Set encryption for a database connection, taking care of memory zeroing in the process.
    pub fn execute_sqlite_pragma(&self, conn: &Connection) -> Result<()> {
        let mut encoded = hex::encode(&self.database_key);
        let mut sql = format!("PRAGMA key = \"x'{}'\";", &encoded);
        conn.execute_batch(&sql)?;
        sql.zeroize();
        encoded.zeroize();
        Ok(())
    }

    pub fn from(mut key_as_hex_string: String) -> Result<DatabaseKey> {
        key_as_hex_string.make_ascii_uppercase();
        for c in key_as_hex_string.chars() {
            if !(('A'..='Z').contains(&c) || ('0'..='9').contains(&c)) {
                key_as_hex_string.zeroize();
                return Err(Error {
                    code: StatusCode::BAD_REQUEST,
                    // important: don't log the actual char
                    msg: "Invalid database key character".to_string(),
                });
            }
        }
        Ok(DatabaseKey {
            database_key: key_as_hex_string,
        })
    }
    pub fn create_plugin_auth(&self) -> Result<PluginAuth> {
        create_plugin_auth(&self.database_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let db_key = "0".repeat(64);
        let db_key_struct = DatabaseKey::from(db_key.clone()).unwrap();
        let auth = db_key_struct.create_plugin_auth().unwrap();
        assert!(!auth.data.nonce.contains(&db_key));
        assert!(!auth.data.encrypted_permissions.contains(&db_key));
        assert_eq!(
            auth.data.encrypted_permissions.len(),
            db_key.len() + 32,
            "Encrypted permissions should include the 16-byte (32 hex characters) TAG."
        );
        let decrypted = extract_database_key(&auth).unwrap();
        assert_eq!(decrypted.database_key, db_key);
    }
}
