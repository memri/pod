use crate::api_model::AuthKey;
use crate::api_model::Bulk;
use crate::api_model::CreateItem;
use crate::api_model::GetFile;
use crate::api_model::PayloadWrapper;
use crate::api_model::Search;
use crate::api_model::UpdateItem;
use crate::command_line_interface;
use crate::command_line_interface::CLIOptions;
use crate::constants;
use crate::database_api;
use crate::database_migrate_refinery;
use crate::error::Error;
use crate::error::Result;
use crate::file_api;
use crate::internal_api;
use crate::plugin_auth_crypto;
use crate::plugin_auth_crypto::DatabaseKey;
use lazy_static::lazy_static;
use log::error;
use log::info;
use rusqlite::Connection;
use rusqlite::Transaction;
use serde_json::Value;
use sha2::digest::generic_array::typenum::U32;
use sha2::digest::generic_array::GenericArray;
use sha2::Digest;
use std::collections::HashSet;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::RwLock;
use warp::http::status::StatusCode;

//
// Items API:
//

pub fn get_item(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<String>,
) -> Result<Vec<Value>> {
    let auth = body.auth;
    let payload = body.payload;
    let database_key = auth_to_database_key(auth)?;
    let mut conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &database_key)?;
    in_transaction(&mut conn, |tx| {
        let schema = database_api::get_schema(&tx)?;
        internal_api::get_item_tx(&tx, &schema, &payload)
    })
}

pub fn create_item(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<CreateItem>,
    cli: &CLIOptions,
) -> Result<i64> {
    let auth = body.auth;
    let payload = body.payload;
    let database_key = auth_to_database_key(auth)?;
    let mut conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &database_key)?;
    in_transaction(&mut conn, |tx| {
        let schema = database_api::get_schema(&tx)?;
        internal_api::create_item_tx(&tx, &schema, payload, &owner, cli, &database_key)
    })
}

pub fn update_item(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<UpdateItem>,
) -> Result<()> {
    let auth = body.auth;
    let payload = body.payload;
    let database_key = auth_to_database_key(auth)?;
    let mut conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &database_key)?;
    in_transaction(&mut conn, |tx| {
        let schema = database_api::get_schema(&tx)?;
        internal_api::update_item_tx(&tx, &schema, &payload.id, payload.fields)
    })
}

pub fn bulk(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<Bulk>,
    cli: &CLIOptions,
) -> Result<()> {
    let auth = body.auth;
    let payload = body.payload;
    let database_key = auth_to_database_key(auth)?;
    let mut conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &database_key)?;
    in_transaction(&mut conn, |tx| {
        let schema = database_api::get_schema(&tx)?;
        internal_api::bulk_tx(&tx, &schema, payload, &owner, cli, &database_key)
    })
}

pub fn delete_item(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<String>,
) -> Result<()> {
    let auth = body.auth;
    let payload = body.payload;
    let database_key = auth_to_database_key(auth)?;
    let mut conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &database_key)?;
    in_transaction(&mut conn, |tx| {
        let schema = database_api::get_schema(&tx)?;
        internal_api::delete_item_tx(&tx, &schema, &payload)
    })
}

pub fn search(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<Search>,
) -> Result<Vec<Value>> {
    let auth = body.auth;
    let payload = body.payload;
    let database_key = auth_to_database_key(auth)?;
    let mut conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &database_key)?;
    in_transaction(&mut conn, |tx| {
        let schema = database_api::get_schema(&tx)?;
        internal_api::search(&tx, &schema, payload)
    })
}

//
// Files API:
//

pub fn upload_file(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    database_key: String,
    expected_sha256: String,
    body: &[u8],
) -> Result<()> {
    // auth_wrapper and the HTTP request will be re-done for more proper structure later
    let database_key = DatabaseKey::from(database_key);
    let mut conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &database_key)?;
    conn.execute_batch("SELECT 1 FROM items;")?; // Check DB access
    in_transaction(&mut conn, |tx| {
        file_api::upload_file(tx, owner, expected_sha256, body)
    })
}

pub fn get_file(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<GetFile>,
) -> Result<Vec<u8>> {
    let auth = body.auth;
    let payload = body.payload;
    let database_key = auth_to_database_key(auth)?;
    let mut conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &database_key)?;
    conn.execute_batch("SELECT 1 FROM items;")?; // Check DB access
    in_transaction(&mut conn, |tx| {
        file_api::get_file(tx, &owner, &payload.sha256)
    })
}

//
// helper functions:
//

fn in_transaction<T, F: FnOnce(&Transaction) -> Result<T>>(
    conn: &mut Connection,
    func: F,
) -> Result<T> {
    let tx = conn.transaction()?;
    let result = func(&tx)?; // Note that this function needs to exit early in case of error
    tx.commit()?;
    Ok(result)
}

fn auth_to_database_key(auth: AuthKey) -> Result<DatabaseKey> {
    match auth {
        AuthKey::ClientAuth(c) => Ok(DatabaseKey::from(c.database_key)),
        AuthKey::PluginAuth(p) => plugin_auth_crypto::extract_database_key(&p),
    }
}

/// Two methods combined into one to prevent creating a database connection without owner checks.
/// As additional failsafe to the fact that non-owners don't have the database key.
fn check_owner_and_initialize_db(
    owner: &str,
    init_db: &RwLock<HashSet<String>>,
    database_key: &DatabaseKey,
) -> Result<Connection> {
    check_owner(owner)?;
    initialize_db(owner, init_db, database_key)
}

fn initialize_db(
    owner: &str,
    init_db: &RwLock<HashSet<String>>,
    database_key: &DatabaseKey,
) -> Result<Connection> {
    let database_path = format!("{}{}", &owner, constants::DATABASE_SUFFIX);
    let database_path = PathBuf::from(constants::DATABASE_DIR).join(database_path);
    let mut conn = Connection::open(database_path).unwrap();
    DatabaseKey::execute_sqlite_pragma(database_key, &conn)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    let mut init_db = init_db.write()?;
    if !init_db.contains(owner) {
        database_migrate_refinery::migrate(&mut conn)?;
        init_db.insert(owner.to_string());
    }
    Ok(conn)
}

fn allowed_owner_hashes_fn(cli_options: &CLIOptions) -> HashSet<Vec<u8>> {
    let owners: &str = &cli_options.owners;
    let mut result = HashSet::new();
    for owner in owners.split(',') {
        let hexed = hex::decode(owner).unwrap_or_else(|err| {
            error!(
                "Pod owners are invalid, failed to decode hex {}, {}",
                owner, err
            );
            std::process::exit(1);
        });
        result.insert(hexed);
    }
    result
}

lazy_static! {
    static ref ALLOWED_OWNER_HASHES: HashSet<Vec<u8>> =
        allowed_owner_hashes_fn(&command_line_interface::PARSED);
}

fn hash_of_hex(hex_string: &str) -> Result<GenericArray<u8, U32>> {
    let hex_string: Vec<u8> = hex::decode(hex_string)?;
    let mut hash = sha2::Sha256::new();
    hash.update(&hex_string);
    Ok(hash.finalize())
}

fn check_owner(possible_owner: &str) -> Result<()> {
    if command_line_interface::PARSED.owners == "ANY" {
        return Ok(());
    };
    let possible_hash = hash_of_hex(possible_owner)?;
    if ALLOWED_OWNER_HASHES.contains(possible_hash.deref()) {
        Ok(())
    } else {
        info!(
            "Denying unexpected owner {} with hash {}",
            possible_owner,
            hex::encode(possible_hash)
        );
        Err(Error {
            code: StatusCode::FORBIDDEN,
            msg: "Unexpected owner".to_string(),
        })
    }
}
