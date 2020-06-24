use crate::data_model::AuditAccessLog;
use crate::data_model::Item;
use crate::data_model::NodeReference;
use crate::data_model::UID;
use bytes::Bytes;
use chrono::DateTime;
use chrono::Utc;
use log::debug;
use log::trace;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::types::ValueRef;
use serde_json::json;
use serde_json::to_string_pretty;
use serde_json::Value;
use std::str;

/// Get project version as seen by Cargo.
pub fn get_project_version() -> &'static str {
    debug!("Returning API version...");
    env!("CARGO_PKG_VERSION")
}

/// Get an item from the SQLite database.
/// None if the `id` doesn't exist in DB, Some(json) if it does.
/// `syncState` is added to the returned json,
/// based on the version in DB and if properties are all included.
pub fn get_item(sqlite: &Pool<SqliteConnectionManager>, id: i64) -> Option<String> {
    debug!("Getting item {}", id);
    let conn = sqlite.get().expect("Failed to obtain connection");

    let mut stmt = conn
        .prepare("SELECT * FROM items WHERE id = :id")
        .expect("");

    let names = stmt
        .column_names()
        .into_iter()
        .map(|s| String::from(s))
        .collect::<Vec<_>>();
    let mut rows = stmt.query_named(&[(":id", &id.to_string())]).expect("");

    let mut values: Vec<(String, Value)> = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        for i in 0..names.len() {
            let name = names[i].clone();
            match row.get_raw(i) {
                ValueRef::Null => values.push((name, Value::Null)),
                ValueRef::Integer(i) => values.push((name, Value::Number(i))),
                ValueRef::Real(f) => values.push((name, Value::Number(f))),
                ValueRef::Text(t) => values.push((name, Value::String(String::from(t)))),
                _ => {}
            };
        }
    }
    let serialized = serde_json::to_string(&values).unwrap();
    let result: Item = serde_json::from_str(serialized.as_str()).unwrap();
    println!("{:#?}", result);
    Some(json!({"name": "John Doe"}).to_string())
}

/// Get an array all items from the dgraph database.
/// `syncState` is added to the returned json for each item,
/// based on the version in dgraph and if properties are all included.
pub fn get_all_items(_sqlite: &Pool<SqliteConnectionManager>) -> Option<String> {
    unimplemented!()
}

/// Create an item presuming it didn't exist before.
/// Return Some(uid) if the data item's `memriID` cannot be found in the DB,
/// and was successfully created in DB.
/// Return None if the data item had `uid` field, indicating it already exists,
/// should use `update` for changing the item.
/// Dgraph returns `uid` in hexadecimal and should be converted to decimal.
/// `syncState` field of the json from client is processed and removed,
/// before the json is inserted into dgraph.
/// The new item will be created with `version = 1`.
pub fn create_item(_sqlite: &Pool<SqliteConnectionManager>, json: Value) -> Option<u64> {
    debug!("Creating item {}", json);
    unimplemented!()
}

/// First verify if `mid` exists, if so, then update the already existing item.
/// Parameters:
///     - `mid`: memriID of the item to be updated.
///     - `json`: the json sent by client.
/// Return `false` if dgraph didn't have a node with this `mid`.
/// Return `true` if dgraph had a node with this `mid` and it was successfully updated.
/// The `version` that is sent to us by the client should be completely ignored.
/// Use `expand(_all_)` to get all properties of an item, only works if data has a `dgraph.type`.
/// `syncState` from client json is processed and removed.
/// A successful update operation should also increase the version in dgraph as `version += 1`.
pub fn update_item(_sqlite: &Pool<SqliteConnectionManager>, memri_id: String, json: Value) -> bool {
    debug!("Updating item {} with {}", memri_id, json);
    unimplemented!()
}

/// Delete an already existing item.
/// Return `false` if dgraph didn't have a node with this memriID.
/// Return `true` if dgraph had a node with this memriID and it was successfully deleted.
pub fn delete_item(_sqlite: &Pool<SqliteConnectionManager>, memri_id: String) -> bool {
    debug!("Deleting item {}", memri_id);
    unimplemented!()
}

/// Query a subset of items.
/// Return `None` if response doesn't contain any items, Some(json) if it does.
/// `syncState` is added to the returned json for the root-level items.
pub fn query(_sqlite: &Pool<SqliteConnectionManager>, query_bytes: Bytes) -> Option<String> {
    debug!("Query {:?}", query_bytes);
    unimplemented!()
}

pub fn _write_access_audit_log(underlying_uid: UID) {
    let audit = AuditAccessLog {
        audit_target: NodeReference {
            uid: underlying_uid,
        },
        date_created: Utc::now(),
    };
    trace!("Adding audit entry: {}", to_string_pretty(&audit).unwrap());
    unimplemented!()
}

/// Given a node "type" and a date,
/// find all nodes of the specified "type"
/// that were accessed by the user after the specified date.
/// Return the specified fields only (parameter `fields`).
///
/// User access is defined in terms of access log entries.
pub fn _get_updates(node_type: &str, date_from: DateTime<Utc>, fields: &[&str]) -> bool {
    debug!(
        "Getting updates for node type {} starting from {} and limiting the result fields to {:?}",
        node_type, date_from, fields
    );
    // research what it means to write in dgraph
    // research how to filter audit logs by "audit" type and date
    // research how, given all audit logs, get all uid-s

    panic!()
}
