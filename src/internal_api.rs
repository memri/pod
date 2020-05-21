use crate::data_model;
use crate::sync_state;
use bytes::Bytes;
use dgraph::Dgraph;
use log::debug;
use serde_json::Map;
use serde_json::Value;
use std::collections::HashMap;
use std::ops::Deref;
use std::str;
use std::sync::Arc;

/// Verify if the response contains the content of the requested node.
/// Return `true` if response doesn't contain any content.
/// Return `false` if the node is found and its content is included in the response.
fn response_is_empty(result: &Value) -> bool {
    let result = result.get("items").unwrap().as_array().unwrap();
    let empty = result.len() == 0 || result.first().unwrap().get("type") == Option::None;
    empty
}

/// Get project version as seen by Cargo.
pub fn get_project_version() -> &'static str {
    debug!("Returning API version...");
    env!("CARGO_PKG_VERSION")
}

/// Get an item from the dgraph database.
/// None if the `memriID` doesn't exist in DB, Some(json) if it does.
/// `syncState` is added to the returned json,
/// based on the version in dgraph and if properties are all included.
pub fn get_item(_dgraph: &Arc<Dgraph>, mid: u64) -> Option<String> {
    debug!("Getting item {}", mid);
    let query = r#"query all($a: string){
        items(func: eq(memriID, $a)) {
            type : dgraph.type
            expand(_all_)
        }
    }"#;

    let mut vars = HashMap::new();
    vars.insert("$a".to_string(), mid.to_string());

    let resp = _dgraph
        .new_readonly_txn()
        .query_with_vars(query, vars)
        .expect("query");

    let result: Value = serde_json::from_slice(&resp.json).unwrap();
    if response_is_empty(&result) {
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
                type : dgraph.type
                expand(_all_)
                }
            }"#;
    let resp = _dgraph.new_readonly_txn().query(query).expect("query");

    let result: Value = serde_json::from_slice(&resp.json).unwrap();
    if response_is_empty(&result) {
        None
    } else {
        Some(sync_state::set_syncstate_all(result))
    }
}

/// Create an item presuming it didn't exist before.
/// Return Some(uid) if the data item's `memriID` cannot be found in the DB,
/// and was successfully created in DB.
/// Return None if the data item had `memriID` field, indicating it already exists,
/// should use `update` for changing the item.
/// Dgraph returns `uid` in hexadecimal and should be converted to decimal.
/// `syncState` field of the json from client is processed and removed,
/// before the json is inserted into dgraph.
/// The new item will be created with `version = 1`.
pub fn create_item(_dgraph: &Arc<Dgraph>, _json: Value) -> Option<u64> {
    debug!("Creating item {}", _json);
    let new_json: Map<String, Value> = _json.clone().as_object().unwrap().clone();
    // Check for blank node labels
    let has_blank_label = new_json.contains_key("uid");
    if !has_blank_label
        || (has_blank_label
            && new_json
                .get("uid")
                .unwrap()
                .as_str()
                .unwrap()
                .starts_with("_:"))
    {
        let mut txn = _dgraph.new_txn();
        let mut mutation = dgraph::Mutation::new();

        let version = 1;
        mutation.set_set_json(
            serde_json::to_vec(&sync_state::get_syncstate(_json, version, Value::Null))
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

/// First verify if `mid` exists, if so, then update the already existing item.
/// Parameters:
///     - `mid`: memriID of the item to be updated.
///     - `_json`: the json sent by client.
/// Return `false` if dgraph didn't have a node with this `mid`.
/// Return `true` if dgraph had a node with this `mid` and it was successfully updated.
/// The `version` that is sent to us by the client should be completely ignored.
/// Use `expand(_all_)` to get all properties of an item, only works if data has a `dgraph.type`.
/// `syncState` from client json is processed and removed.
/// A successful update operation should also increase the version in dgraph as `version += 1`.
pub fn update_item(_dgraph: &Arc<Dgraph>, mid: u64, mut _json: Value) -> bool {
    debug!("Updating item {} with {}", mid, _json);
    let found;

    let query = r#"query all($a: string){
        items(func: eq(memriID, $a)) {
            uid
            expand(_all_)
        }
    }"#;

    let mut vars = HashMap::new();
    vars.insert("$a".to_string(), mid.to_string());

    let resp = _dgraph
        .new_readonly_txn()
        .query_with_vars(query, vars)
        .expect("query");

    let items: Value = serde_json::from_slice(&resp.json).unwrap();
    let null_item: Value = serde_json::from_str(r#"{"items": []}"#).unwrap();

    if items == null_item {
        found = false;
    } else {
        // version += 1
        let root: data_model::Items = serde_json::from_slice(&resp.json).unwrap();
        let new_ver = root.items.first().unwrap().version + 1;
        let uid = items
            .as_object()
            .unwrap()
            .get("items")
            .unwrap()
            .as_array()
            .unwrap()
            .first()
            .unwrap()
            .get("uid")
            .unwrap();
        let _json = sync_state::get_syncstate(_json, new_ver, uid.clone());

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
/// Return `false` if dgraph didn't have a node with this memriID.
/// Return `true` if dgraph had a node with this memriID and it was successfully deleted.
pub fn delete_item(_dgraph: &Arc<Dgraph>, mid: u64) -> bool {
    debug!("Deleting item {}", mid);
    let deleted;

    let query = r#"query all($a: string){
        items(func: eq(memriID, $a)) {
            uid
            expand(_all_)
        }
    }"#;

    let mut vars = HashMap::new();
    vars.insert("$a".to_string(), mid.to_string());

    let resp = _dgraph
        .new_readonly_txn()
        .query_with_vars(query, vars)
        .expect("query");

    let items: Value = serde_json::from_slice(&resp.json).unwrap();
    let null_item: Value = serde_json::from_str(r#"{"items": []}"#).unwrap();

    if items == null_item {
        deleted = false;
    } else {
        let mut txn = _dgraph.new_txn();
        let mut mutation = dgraph::Mutation::new();

        let uid = items
            .as_object()
            .unwrap()
            .get("items")
            .unwrap()
            .as_array()
            .unwrap()
            .first()
            .unwrap()
            .get("uid")
            .unwrap();
        let _json = serde_json::json!({ "uid": uid });

        mutation.set_delete_json(serde_json::to_vec(&_json).expect("Failed to serialize JSON."));
        let _resp = txn.mutate(mutation).expect("Failed to create data.");
        txn.commit().expect("Failed to commit mutation");

        deleted = true;
    }

    deleted
}

/// Query a subset of items.
/// Return `None` if response doesn't contain any items, Some(json) if it does.
/// `syncState` is added to the returned json for the root-level items.
pub fn query(_dgraph: &Arc<Dgraph>, body: Bytes) -> Option<String> {
    let query = std::str::from_utf8(body.deref()).unwrap();
    debug!("Query {}", query);

    let resp = _dgraph.new_readonly_txn().query(query).expect("query");
    let result: Value = serde_json::from_slice(&resp.json).unwrap();
    if response_is_empty(&result) {
        None
    } else {
        Some(sync_state::set_syncstate_all(result))
    }
}
