use crate::api_model::BulkAction;
use crate::api_model::CreateItem;
use crate::api_model::GetFile;
use crate::api_model::InsertTreeItem;
use crate::api_model::PayloadWrapper;
use crate::api_model::RunDownloader;
use crate::api_model::RunImporter;
use crate::api_model::RunIndexer;
use crate::api_model::SearchByFields;
use crate::api_model::UpdateItem;
use crate::command_line_interface;
use crate::command_line_interface::CLIOptions;
use crate::constants;
use crate::database_migrate_refinery;
use crate::database_migrate_schema;
use crate::error::Error;
use crate::error::Result;
use crate::file_api;
use crate::internal_api;
use crate::services_api;
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
    in_transaction(&mut conn, |tx| {
        internal_api::create_item_tx(&tx, body.payload.fields)
    })
}

pub fn update_item(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<UpdateItem>,
) -> Result<()> {
    let mut conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &body.database_key)?;
    in_transaction(&mut conn, |tx| {
        internal_api::update_item_tx(&tx, body.payload.uid, body.payload.fields)
    })
}

pub fn bulk_action(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<BulkAction>,
) -> Result<()> {
    let mut conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &body.database_key)?;
    in_transaction(&mut conn, |tx| {
        internal_api::bulk_action_tx(&tx, body.payload)
    })
}

pub fn delete_item(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<i64>,
) -> Result<()> {
    let mut conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &body.database_key)?;
    in_transaction(&mut conn, |tx| {
        internal_api::delete_item_tx(&tx, body.payload)
    })
}

pub fn insert_tree(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<InsertTreeItem>,
    shared_server: bool,
) -> Result<i64> {
    let mut conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &body.database_key)?;
    in_transaction(&mut conn, |tx| {
        internal_api::insert_tree(&tx, body.payload, shared_server)
    })
}

pub fn search_by_fields(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<SearchByFields>,
) -> Result<Vec<Value>> {
    let mut conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &body.database_key)?;
    in_transaction(&mut conn, |tx| {
        internal_api::search_by_fields(&tx, body.payload)
    })
}

pub fn get_items_with_edges(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<Vec<i64>>,
) -> Result<Vec<Value>> {
    let mut conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &body.database_key)?;
    in_transaction(&mut conn, |tx| {
        internal_api::get_items_with_edges_tx(&tx, &body.payload)
    })
}

//
// Services
//

pub fn run_downloader(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<RunDownloader>,
    cli_options: &CLIOptions,
) -> Result<()> {
    let conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &body.database_key)?;
    conn.execute_batch("SELECT 1 FROM items;")?; // Check DB access
    services_api::run_downloader(&conn, body.payload, cli_options)
}

pub fn run_importer(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<RunImporter>,
    cli_options: &CLIOptions,
) -> Result<()> {
    let conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &body.database_key)?;
    conn.execute_batch("SELECT 1 FROM items;")?; // Check DB access
    services_api::run_importer(&conn, body.payload, cli_options)
}

pub fn run_indexer(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    body: PayloadWrapper<RunIndexer>,
    cli_options: &CLIOptions,
) -> Result<()> {
    let conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &body.database_key)?;
    conn.execute_batch("SELECT 1 FROM items;")?; // Check DB access
    services_api::run_indexers(&conn, body.payload, cli_options)
}

pub fn upload_file(
    owner: String,
    init_db: &RwLock<HashSet<String>>,
    database_key: String,
    expected_sha256: String,
    body: &[u8],
) -> Result<()> {
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
    let mut conn: Connection = check_owner_and_initialize_db(&owner, &init_db, &body.database_key)?;
    conn.execute_batch("SELECT 1 FROM items;")?; // Check DB access
    in_transaction(&mut conn, |tx| {
        file_api::get_file(tx, &owner, &body.payload.sha256)
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
    let database_path = PathBuf::from(constants::DATABASE_DIR).join(database_path);
    let mut conn = Connection::open(database_path).unwrap();
    let pragma_sql = format!("PRAGMA key = \"x'{}'\";", database_key);
    conn.execute_batch(&pragma_sql)?;
    let mut init_db = init_db.write()?;
    if !init_db.contains(owner) {
        database_migrate_refinery::migrate(&mut conn)?;
        database_migrate_schema::migrate(&conn).map_err(|err| Error {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: format!("Failed to migrate database according to schema, {}", err),
        })?;
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
