use crate::api_model::AuthKey;
use crate::api_model::Bulk;
use crate::api_model::CreateEdge;
use crate::api_model::CreateItem;
use crate::api_model::GetEdges;
use crate::api_model::GetFile;
use crate::api_model::PayloadWrapper;
use crate::api_model::Search;
use crate::api_model::SendEmail;
use crate::api_model::UpdateItem;
use crate::command_line_interface;
use crate::command_line_interface::CliOptions;
use crate::constants;
use crate::database_api;
use crate::database_migrate_refinery;
use crate::email;
use crate::error::Error;
use crate::error::ErrorContext;
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
use warp::hyper::body::Bytes;

//
// Items API:
//

pub fn get_item(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: Bytes,
) -> Result<Vec<Value>> {
    let body = &mut serde_json::Deserializer::from_slice(body.deref());
    let body: PayloadWrapper<String> = serde_path_to_error::deserialize(body)?;
    let auth = body.auth;
    let payload = body.payload;
    let database_key = auth_to_database_key(auth)?;
    let mut conn: Connection = check_owner_and_initialize_db(&owner, init_db, &database_key)?;
    in_transaction(&mut conn, |tx| {
        let schema = database_api::get_schema(tx)?;
        internal_api::get_item_tx(tx, &schema, &payload)
    })
}

pub fn create_item(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: Bytes,
    cli: &CliOptions,
) -> Result<String> {
    let body = &mut serde_json::Deserializer::from_slice(body.deref());
    let body: PayloadWrapper<CreateItem> = serde_path_to_error::deserialize(body)?;
    let auth = body.auth;
    let payload = body.payload;
    let database_key = auth_to_database_key(auth)?;
    let mut conn: Connection = check_owner_and_initialize_db(&owner, init_db, &database_key)?;
    in_transaction(&mut conn, |tx| {
        let mut schema = database_api::get_schema(tx)?;
        internal_api::create_item_tx(tx, &mut schema, payload, &owner, cli, &database_key)
    })
}

pub fn update_item(owner: String, init_db: &RwLock<HashSet<String>>, body: Bytes) -> Result<()> {
    let body = &mut serde_json::Deserializer::from_slice(body.deref());
    let body: PayloadWrapper<UpdateItem> = serde_path_to_error::deserialize(body)?;
    let auth = body.auth;
    let payload = body.payload;
    let database_key = auth_to_database_key(auth)?;
    let mut conn: Connection = check_owner_and_initialize_db(&owner, init_db, &database_key)?;
    in_transaction(&mut conn, |tx| {
        let schema = database_api::get_schema(tx)?;
        internal_api::update_item_tx(tx, &schema, &payload.id, payload.fields)
    })
}

pub fn bulk(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: Bytes,
    cli: &CliOptions,
) -> Result<Value> {
    let body = &mut serde_json::Deserializer::from_slice(body.deref());
    let body: PayloadWrapper<Bulk> = serde_path_to_error::deserialize(body)?;
    let auth = body.auth;
    let payload = body.payload;
    let database_key = auth_to_database_key(auth)?;
    let mut conn: Connection = check_owner_and_initialize_db(&owner, init_db, &database_key)?;
    in_transaction(&mut conn, |tx| {
        let mut schema = database_api::get_schema(tx)?;
        internal_api::bulk_tx(tx, &mut schema, payload, &owner, cli, &database_key)
    })
}

pub fn delete_item(owner: String, init_db: &RwLock<HashSet<String>>, body: Bytes) -> Result<()> {
    let body = &mut serde_json::Deserializer::from_slice(body.deref());
    let body: PayloadWrapper<String> = serde_path_to_error::deserialize(body)?;
    let auth = body.auth;
    let payload = body.payload;
    let database_key = auth_to_database_key(auth)?;
    let mut conn: Connection = check_owner_and_initialize_db(&owner, init_db, &database_key)?;
    in_transaction(&mut conn, |tx| {
        let schema = database_api::get_schema(tx)?;
        internal_api::delete_item_tx(tx, &schema, &payload)
    })
}

pub fn create_edge(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: Bytes,
) -> Result<String> {
    let body = &mut serde_json::Deserializer::from_slice(body.deref());
    let body: PayloadWrapper<CreateEdge> = serde_path_to_error::deserialize(body)?;
    let auth = body.auth;
    let payload = body.payload;
    let database_key = auth_to_database_key(auth)?;
    let mut conn: Connection = check_owner_and_initialize_db(&owner, init_db, &database_key)?;
    in_transaction(&mut conn, |tx| internal_api::create_edge(tx, payload))
}

pub fn get_edges(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: Bytes,
) -> Result<Vec<Value>> {
    let body = &mut serde_json::Deserializer::from_slice(body.deref());
    let body: PayloadWrapper<GetEdges> = serde_path_to_error::deserialize(body)?;
    let auth = body.auth;
    let payload = body.payload;
    let database_key = auth_to_database_key(auth)?;
    let mut conn: Connection = check_owner_and_initialize_db(&owner, init_db, &database_key)?;
    in_transaction(&mut conn, |tx| {
        let schema = database_api::get_schema(tx)?;
        internal_api::get_edges(tx, payload, &schema)
    })
}

pub fn search(owner: String, init_db: &RwLock<HashSet<String>>, body: Bytes) -> Result<Vec<Value>> {
    let body = &mut serde_json::Deserializer::from_slice(body.deref());
    let body: PayloadWrapper<Search> = serde_path_to_error::deserialize(body)?;
    let auth = body.auth;
    let payload = body.payload;
    let database_key = auth_to_database_key(auth)?;
    let mut conn: Connection = check_owner_and_initialize_db(&owner, init_db, &database_key)?;
    in_transaction(&mut conn, |tx| {
        let schema = database_api::get_schema(tx)?;
        internal_api::search(tx, &schema, payload)
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
    let database_key = DatabaseKey::from(database_key)?;
    let mut conn: Connection = check_owner_and_initialize_db(&owner, init_db, &database_key)?;
    conn.execute_batch("SELECT 1 FROM items;")?; // Check DB access
    in_transaction(&mut conn, |tx| {
        file_api::upload_file(tx, &owner, &expected_sha256, body)
    })
}

pub fn upload_file_b(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    auth_json: String,
    expected_sha256: String,
    body: &[u8],
) -> Result<()> {
    let auth_json = percent_encoding::percent_decode_str(&auth_json)
        .decode_utf8()
        .map_err(|err| Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!("Failed to decode UTF-8 authorization JSON, {}", err),
        })?;
    let auth: AuthKey = serde_json::from_str(&auth_json)?;
    let database_key = auth_to_database_key(auth)?;
    let mut conn: Connection = check_owner_and_initialize_db(&owner, init_db, &database_key)?;
    conn.execute_batch("SELECT 1 FROM items;")?; // Check DB access
    in_transaction(&mut conn, |tx| {
        file_api::upload_file(tx, &owner, &expected_sha256, body)
    })
}

pub fn get_file(owner: String, init_db: &RwLock<HashSet<String>>, body: Bytes) -> Result<Vec<u8>> {
    let body = &mut serde_json::Deserializer::from_slice(body.deref());
    let body: PayloadWrapper<GetFile> = serde_path_to_error::deserialize(body)?;
    let auth = body.auth;
    let payload = body.payload;
    let database_key = auth_to_database_key(auth)?;
    let mut conn: Connection = check_owner_and_initialize_db(&owner, init_db, &database_key)?;
    conn.execute_batch("SELECT 1 FROM items;")?; // Check DB access
    in_transaction(&mut conn, |tx| {
        file_api::get_file(tx, &owner, &payload.sha256)
    })
}

//
// Email API
//

pub fn send_email(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: Bytes,
    cli: &CliOptions,
) -> Result<()> {
    let body = &mut serde_json::Deserializer::from_slice(body.deref());
    let body: PayloadWrapper<SendEmail> = serde_path_to_error::deserialize(body)?;
    let auth = body.auth;
    let payload = body.payload;
    let database_key = auth_to_database_key(auth)?;
    let conn: Connection = check_owner_and_initialize_db(&owner, init_db, &database_key)?;
    conn.execute_batch("SELECT 1 FROM items;")?; // Check DB access
    email::send_email(payload, cli)
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
        AuthKey::ClientAuth(c) => DatabaseKey::from(c.database_key),
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
    let mut conn = Connection::open(database_path)
        .context_str("Failed to open database file (does Pod have filesystem access?)")?;
    DatabaseKey::execute_sqlite_pragma(database_key, &conn)
        .context_str("Failed to open the database file (did databaseKey change between runs?)")?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    let mut init_db = init_db.write()?;
    if !init_db.contains(owner) {
        database_migrate_refinery::migrate(&mut conn)?;
        init_db.insert(owner.to_string());
    }
    Ok(conn)
}

fn allowed_owner_hashes_fn(cli_options: &CliOptions) -> HashSet<Vec<u8>> {
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
