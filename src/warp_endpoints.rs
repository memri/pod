use crate::api_model::BulkAction;
use crate::api_model::CreateItem;
use crate::api_model::PayloadWrapper;
use crate::api_model::RunDownloader;
use crate::api_model::RunImporter;
use crate::api_model::RunIndexer;
use crate::api_model::UpdateItem;
use crate::configuration;
use crate::database_migrate_refinery;
use crate::database_migrate_schema;
use crate::error::Error;
use crate::error::Result;
use crate::internal_api;
use crate::services_api;
use blake2::digest::Update;
use blake2::digest::VariableOutput;
use blake2::VarBlake2b;
use lazy_static::lazy_static;
use log::error;
use log::info;
use rusqlite::Connection;
use serde_json::Value;
use std::collections::HashSet;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::RwLock;
use warp::http::status::StatusCode;

pub fn get_item(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<i64>,
) -> Result<Vec<Value>> {
    let conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &body.database_key)?;
    internal_api::get_item(&conn, body.payload)
}

pub fn get_all_items(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<()>,
) -> Result<Vec<Value>> {
    let conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &body.database_key)?;
    internal_api::get_all_items(&conn)
}

pub fn create_item(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<CreateItem>,
) -> Result<i64> {
    let mut conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &body.database_key)?;
    let tx = conn.transaction()?;
    let result = internal_api::create_item_tx(&tx, body.payload.fields)?;
    tx.commit()?;
    Ok(result)
}

pub fn update_item(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<UpdateItem>,
) -> Result<()> {
    let mut conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &body.database_key)?;
    let tx = conn.transaction()?;
    internal_api::update_item_tx(&tx, body.payload.uid, body.payload.fields)?;
    tx.commit()?;
    Ok(())
}

pub fn bulk_action(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<BulkAction>,
) -> Result<()> {
    let mut conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &body.database_key)?;
    let tx = conn.transaction()?;
    internal_api::bulk_action_tx(&tx, body.payload)?;
    tx.commit()?;
    Ok(())
}

pub fn delete_item(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<i64>,
) -> Result<()> {
    let mut conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &body.database_key)?;
    let tx = conn.transaction()?;
    internal_api::delete_item_tx(&tx, body.payload)?;
    tx.commit()?;
    Ok(())
}

pub fn search_by_fields(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<Value>,
) -> Result<Vec<Value>> {
    let mut conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &body.database_key)?;
    let tx = conn.transaction()?;
    let result = internal_api::search_by_fields(&tx, body.payload)?;
    tx.commit()?;
    Ok(result)
}

pub fn get_items_with_edges(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<Vec<i64>>,
) -> Result<Vec<Value>> {
    let mut conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &body.database_key)?;
    let tx = conn.transaction()?;
    let result = internal_api::get_items_with_edges_tx(&tx, &body.payload)?;
    tx.commit()?;
    Ok(result)
}

pub fn run_downloader(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<RunDownloader>,
) -> Result<()> {
    let conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &body.database_key)?;
    conn.execute_batch("SELECT 1 FROM items;")?; // Check DB access
    services_api::run_downloader(body.payload.service, body.payload.data_type)
}

pub fn run_importer(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<RunImporter>,
) -> Result<()> {
    let conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &body.database_key)?;
    conn.execute_batch("SELECT 1 FROM items;")?; // Check DB access
    services_api::run_importer(body.payload.data_type)
}

pub fn run_indexer(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<RunIndexer>,
) -> Result<()> {
    let conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &body.database_key)?;
    conn.execute_batch("SELECT 1 FROM items;")?; // Check DB access
    services_api::run_indexers(&conn, body.payload.uid)
}

//
// helper functions:
//

/// Two methods combined into one to prevent creating a database connection without owner checks.
/// As additional failsafe to the fact that non-owners don't have the database key.
fn check_owner_and_initialize_db(
    owner: &str,
    init_db: &RwLock<HashSet<String>>,
    database_key: &str,
) -> Result<Connection> {
    check_owner(owner)?;
    initialize_db(owner, init_db, database_key)
}

fn initialize_db(
    owner: &str,
    init_db: &RwLock<HashSet<String>>,
    database_key: &str,
) -> Result<Connection> {
    let database_path = format!("{}.db", &owner);
    let database_path = PathBuf::from(configuration::DATABASE_DIR).join(database_path);
    let mut conn = Connection::open(database_path).unwrap();
    let pragma_sql = format!("PRAGMA key = \"x'{}'\";", database_key);
    conn.execute_batch(&pragma_sql)?;
    let mut init_db = init_db.write()?;
    if !init_db.contains(owner) {
        init_db.insert(owner.to_string());
        database_migrate_refinery::migrate(&mut conn)?;
        database_migrate_schema::migrate(&conn).map_err(|err| Error {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: format!("Failed to migrate database according to schema, {}", err),
        })?;
    }
    Ok(conn)
}

fn allowed_owner_hashes_fn() -> HashSet<Vec<u8>> {
    if let Some(owners) = configuration::pod_owners() {
        let mut result = HashSet::new();
        for owner in owners.split(',') {
            let hexed = hex::decode(owner).unwrap_or_else(|err| {
                error!(
                    "POD_OWNER_HASHES is invalid, failed to decode {} from hex, {}",
                    owner, err
                );
                std::process::exit(1);
            });
            result.insert(hexed);
        }
        result
    } else {
        error!("No POD_OWNER_HASHES configured for Pod. Without owners to trust, Pod will not be able to answer any HTTP requests.");
        HashSet::new()
    }
}

lazy_static! {
    static ref ALLOWED_OWNER_HASHES: HashSet<Vec<u8>> = allowed_owner_hashes_fn();
}

fn hash_hex(hex_string: &str) -> Result<Box<[u8]>> {
    let mut possible_hash: VarBlake2b = VarBlake2b::new(32).expect("Invalid output size");
    let hex_string = hex::decode(hex_string)?;
    possible_hash.update(hex_string);
    Ok(possible_hash.finalize_boxed())
}

fn check_owner(possible_owner: &str) -> Result<()> {
    if configuration::pod_owners().iter().any(|e| e == "ANY") {
        return Ok(());
    };
    let possible_hash = hash_hex(possible_owner)?;
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
