use crate::data_model;
use crate::sync_state;
use dgraph::Dgraph;
use log::debug;
use serde_json::Map;
use serde_json::Value;
use std::collections::HashMap;
use std::str;
use std::sync::Arc;

/// Check if the `uid` exists in DB, or if DB contains any items
fn item_not_found(result: &Value) -> bool {
    let result = result.get("items").unwrap().as_array().unwrap();
    let empty = result.len() == 0 || result.first().unwrap().get("type") == Option::None;
    if empty {
        true
    } else {
        false
    }
}

/// Get project version as seen by Cargo.
pub fn get_project_version() -> &'static str {
    debug!("Returning API version...");
    env!("CARGO_PKG_VERSION")
}

/// Get an item from the dgraph database.
/// None if the `uid` doesn't exist in DB, Some(json) if it does.
/// `syncState` is added to the returned json,
/// based on the version in dgraph and if properties are all included.
pub fn get_item(_dgraph: &Arc<Dgraph>, uid: u64) -> Option<String> {
    debug!("Getting item {}", uid);
    let query = r#"query all($a: string){
        items(func: uid($a)) {
            uid
            type : dgraph.type
            expand(_all_)
        }
    }"#;

    let mut vars = HashMap::new();
    vars.insert("$a".to_string(), uid.to_string());

    let resp = _dgraph
        .new_readonly_txn()
        .query_with_vars(query, vars)
        .expect("query");

    let result: Value = serde_json::from_slice(&resp.json).unwrap();
    if item_not_found(&result) {
        None
    } else {
        Some(sync_state::set_syncstate(result))
    }
}

/// Get an array all items from the dgraph database.
/// `syncState` is added to the returned json for each item,
/// based on the version in dgraph and if properties are all included.
pub fn get_all_items(_dgraph: &Arc<Dgraph>) -> Option<String> {
    let query = r#"{
            items(func: has(version)) {
                uid
                type : dgraph.type
                expand(_all_)
                }
            }"#;
    let resp = _dgraph.new_readonly_txn().query(query).expect("query");

    let result: Value = serde_json::from_slice(&resp.json).unwrap();
    if item_not_found(&result) {
        None
    } else {
        Some(sync_state::set_syncstate_all(result))
    }
}

/// Create an item presuming it didn't exist before.
/// Return Some(uid) if the data item did not have a `uid` field,
/// and was successfully created in DB.
/// Return None if the data item had `uid` field, indicating it already exists,
/// should use `update` for changing the item.
/// Dgraph returns `uid` in hexadecimal and should be converted to decimal.
/// `syncState` field of the json from client is processed and removed,
/// before the json is inserted into dgraph.
/// The new item will be created with `version = 1`.
pub fn create_item(_dgraph: &Arc<Dgraph>, _json: Value) -> Option<u64> {
    debug!("Creating item {}", _json);
    let new_uid: Map<String, Value> = _json.clone().as_object().unwrap().clone();
    let new_uid = new_uid.get("uid").unwrap();
    if new_uid.is_string() {
        let mut txn = _dgraph.new_txn();
        let mut mutation = dgraph::Mutation::new();

        let version = 1;
        mutation.set_set_json(
            serde_json::to_vec(&sync_state::get_syncstate(_json, version))
                .expect("Failed to serialize JSON."),
        );
        let resp = txn.mutate(mutation).expect("Failed to create data.");
        txn.commit().expect("Failed to commit mutation");

        let hex_uids = resp.uids.values().next().unwrap();
        let without_pre = hex_uids.trim_start_matches("0x");
        let uid = u64::from_str_radix(without_pre, 16).unwrap();
        Some(uid)
    } else {
        None
    }
}

/// First verify if `uid` exists, if so, then update the already existing item.
/// Parameters:
///     - `uid`: uid of the item to be updated.
///     - `_json`: the json sent by client.
/// Return `false` if dgraph didn't have a node with this `uid`.
/// Return `true` if dgraph had a node with this `uid` and it was successfully updated.
/// The `version` that is sent to us by the client should be completely ignored.
/// Use `expand(_all_)` to get all properties of an item, only works if data has a `dgraph.type`.
/// `syncState` from client json is processed and removed.
/// A successful update operation should also increase the version in dgraph as `version += 1`.
pub fn update_item(_dgraph: &Arc<Dgraph>, uid: u64, mut _json: Value) -> bool {
    debug!("Updating item {} with {}", uid, _json);
    let found;

    let query = r#"query all($a: string){
        items(func: uid($a)) {
            expand(_all_)
        }
    }"#;

    let mut vars = HashMap::new();
    vars.insert("$a".to_string(), uid.to_string());
    // Get response of query with `uid`
    let resp = _dgraph
        .new_readonly_txn()
        .query_with_vars(query, vars)
        .expect("query");

    let items: Value = serde_json::from_slice(&resp.json).unwrap();
    let null_item: Value = serde_json::from_str(r#"{"items": []}"#).unwrap();
    // Check if the returned item for the `uid` is an empty item.
    if items == null_item {
        found = false;
    } else {
        // version += 1
        let root: data_model::Items = serde_json::from_slice(&resp.json).unwrap();
        let new_ver = root.items.first().unwrap().version + 1;
        let _json = sync_state::get_syncstate(_json, new_ver);
        // Update via mutation
        let mut txn = _dgraph.new_txn();
        let mut mutation = dgraph::Mutation::new();

        mutation.set_set_json(serde_json::to_vec(&_json).expect("Failed to serialize JSON."));
        let _resp = txn.mutate(mutation).expect("Failed to create data.");
        txn.commit().expect("Failed to commit mutation");

        found = true;
    }

    found
}

/// Delete an already existing item.
/// Return `false` if dgraph didn't have a node with this `uid`.
/// Return `true` if dgraph had a node with this `uid` and it was successfully deleted.
pub fn delete_item(_dgraph: &Arc<Dgraph>, uid: u64) -> bool {
    debug!("Deleting item {}", uid);
    let deleted;

    let query = r#"query all($a: string){
        items(func: uid($a)) {
            expand(_all_)
        }
    }"#;

    let mut vars = HashMap::new();
    vars.insert("$a".to_string(), uid.to_string());
    // Get response of query with `uid`
    let resp = _dgraph
        .new_readonly_txn()
        .query_with_vars(query, vars)
        .expect("query");
    // Verify if `uid` exists
    let items: Value = serde_json::from_slice(&resp.json).unwrap();
    let null_item: Value = serde_json::from_str(r#"{"items": []}"#).unwrap();

    if items == null_item {
        deleted = false;
    } else {
        let mut txn = _dgraph.new_txn();
        let mut mutation = dgraph::Mutation::new();

        let _json = serde_json::json!({ "uid": uid });

        mutation.set_delete_json(serde_json::to_vec(&_json).expect("Failed to serialize JSON."));
        let _resp = txn.mutate(mutation).expect("Failed to create data.");
        txn.commit().expect("Failed to commit mutation");

        deleted = true;
    }

    deleted
}
