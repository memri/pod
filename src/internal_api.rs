use crate::data_model;
use crate::sync_state;
use dgraph::Dgraph;
use log::debug;
use serde_json::Value;
use std::collections::HashMap;
use std::str;
use std::sync::Arc;

/// Get Cargo project version.
/// Return project version, &str.
pub fn version() -> &'static str {
    debug!("Returning API version...");
    env!("CARGO_PKG_VERSION")
}

/// Get an item from the dgraph database.
/// Return an array.
/// None if the `uid` doesn't exist in DB, Some(json) if it does.
/// `syncState` is added to the returned json, based on the version in dgraph and if properties are all included.
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

    Some(sync_state::set_syncstate(resp.json))
}

/// Get all items from the dgraph database.
/// Return an array of all nodes.
/// /// `syncState` is added to the returned json for each item, based on the version in dgraph and if properties are all included.
pub fn get_all_items(_dgraph: &Arc<Dgraph>) -> Option<String> {
    let query = r#"{
            items(func: has(version)) {
                uid
                type : dgraph.type
                expand(_all_)
                }
            }"#;
    let resp = _dgraph.new_readonly_txn().query(query).expect("query");

    Some(sync_state::set_syncstate_all(resp.json))
}

/// Create an item presuming it didn't exist before.
/// Return Some(uid) if the data item did not have a `uid` field, and was successfully created in DB.
/// Return None if the data item had `uid` field, indicating it already exists, should use `update` for changing the item.
/// Parameter:
///     _json: client json, serde_json::Value.
/// Returned `uid` without "0x" prefix.
/// `syncState` from client json is processed and removed, new item will be created with `version = 1`.
pub fn create_item(_dgraph: &Arc<Dgraph>, _json: Value) -> Option<u64> {
    debug!("Creating item {}", _json);
    let mut txn = _dgraph.new_txn();
    let mut mutation = dgraph::Mutation::new();

    mutation.set_set_json(
        serde_json::to_vec(&sync_state::get_syncstate(_json, 1))
            .expect("Failed to serialize JSON."),
    );
    let resp = txn.mutate(mutation).expect("Failed to create data.");
    txn.commit().expect("Failed to commit mutation");

    let hex_uids = resp.uids.values().next().unwrap();
    let without_pre = hex_uids.trim_start_matches("0x");
    let uid = u64::from_str_radix(without_pre, 16).unwrap();

    Some(uid)
}

/// First verify if `uid` exists, if so, then update the already existing item.
/// Parameters:
///     uid: uid of item to be udpated, u64.
///     _json: client json, serde_json::Value.
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
    // Verify if `uid` exists
    let items: Value = serde_json::from_slice(&resp.json).unwrap();
    let null_item: Value = serde_json::from_str(r#"{"items": []}"#).unwrap();

    if items == null_item {
        found = false;
    } else {
        // version += 1
        let root: data_model::Items = serde_json::from_slice(&resp.json).unwrap();
        let new_ver = root.items.first().unwrap().version + 1;
        let new_json = sync_state::get_syncstate(_json.clone(), new_ver);
        // Update via mutation
        let mut txn = _dgraph.new_txn();
        let mut mutation = dgraph::Mutation::new();

        mutation.set_set_json(serde_json::to_vec(&new_json).expect("Failed to serialize JSON."));
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
        // Delete item with `uid` via mutation
        let _json = serde_json::json!({ "uid": uid });

        mutation.set_delete_json(serde_json::to_vec(&_json).expect("Failed to serialize JSON."));
        let _resp = txn.mutate(mutation).expect("Failed to create data.");
        txn.commit().expect("Failed to commit mutation");

        deleted = true;
    }

    deleted
}
