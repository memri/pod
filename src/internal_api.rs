use crate::data_model;
use dgraph::Dgraph;
use log::debug;
use std::str;
use std::sync::Arc;
use std::collections::HashMap;
use serde_json::Value;

pub fn version() -> &'static str {
    debug!("Returning API version...");
    env!("CARGO_PKG_VERSION")
}

/// Get an item from the dgraph database.
/// None if the `uid` doesn't exist in DB, Some(json) if it does.
pub fn get_item(_dgraph: &Arc<Dgraph>, uid: u64) -> Option<String> {
    let query = r#"query all($a: string){
        items(func: uid($a)) {
            uid
            expand(_all_)
        }
    }"#
    .to_string();

    let mut vars = HashMap::new();
    vars.insert("$a".to_string(), uid.to_string());

    let resp = _dgraph
        .new_readonly_txn()
        .query_with_vars(query, vars)
        .expect("query");

    let json_str = str::from_utf8(&resp.json).unwrap();

    Some(json_str).map(String::from)
}

/// Get all items from the dgraph database.
pub fn get_all_item(_dgraph: &Arc<Dgraph>) -> Option<String> {
    let query = format!(
        r#"{{
            items(func: has(version)) {{
                uid
                expand(_all_)
                }}
            }}"#
    );

    let resp = _dgraph
        .new_readonly_txn()
        .query(query)
        .expect("query");

    let json_str = str::from_utf8(&resp.json).unwrap();

    Some(json_str).map(String::from)
}

/// Create an item presuming it didn't exist before.
/// Some(uid) the new data item did not have `uid` and was successfully pud in DB.
/// None if the data item had `uid` field, which indicates
/// the misuse of `create` method. Use `update` method instead then.
///
/// New DataItems should have `version = 1`.
pub fn create_item(_dgraph: &Arc<Dgraph>, _json: Value) -> Option<u64> {
    let mut txn = _dgraph.new_txn();
    let mut mutation = dgraph::Mutation::new();

    mutation.set_set_json(serde_json::to_vec(&_json).expect("Failed to serialize JSON."));
    let resp = txn.mutate(mutation).expect("Failed to create data.");
    txn.commit().expect("Failed to commit mutation");

    let hex_uids = resp.uids.values().next().unwrap();
    let without_pre = hex_uids.trim_start_matches("0x");
    let uid = u64::from_str_radix(without_pre, 16).unwrap();

    Some(uid)
}

/// Update an already existing item.
/// `false` if dgraph didn't have a node with this `uid`.
/// `true` if dgraph had a node with this `uid` and it was successfully updated.
///
/// A successful update operation should also
/// increase `version += 1`, **comparing to the version that was in DB**.
/// The `version` that is sent to us by the client should be completely ignored.
///
/// expand(_all_) only works if data has a dgraph.type
pub fn update_item(_dgraph: &Arc<Dgraph>, uid: u64, mut _json: Value) -> bool {
    let found;

    let query = r#"query all($a: string){
        items(func: uid($a)) {
            expand(_all_)
        }
    }"#
    .to_string();

    let mut vars = HashMap::new();
    vars.insert("$a".to_string(), uid.to_string());

    let resp = _dgraph
    .new_readonly_txn()
    .query_with_vars(query, vars)
    .expect("query");

    let items: Value = serde_json::from_slice(&resp.json).unwrap();
    let null_item: Value = serde_json::from_str(r#"{"items": []}"#).unwrap();

    if items == null_item {
        found = bool::from(false);
    } else {
        let root: data_model::Items = serde_json::from_slice(&resp.json).unwrap();

        let new_ver = root.items.first().unwrap().version + 1;

        *_json.get_mut("version").unwrap() = serde_json::json!(new_ver);

        let mut txn = _dgraph.new_txn();
        let mut mutation = dgraph::Mutation::new();

        mutation.set_set_json(serde_json::to_vec(&_json).expect("Failed to serialize JSON."));
        let _resp = txn.mutate(mutation).expect("Failed to create data.");
        txn.commit().expect("Failed to commit mutation");

        found = bool::from(true);
    }

    found
}

/// Delete an already existing item.
/// `false` if dgraph didn't have a node with this `uid`.
/// `true` if dgraph had a node with this `uid` and it was successfully deleted.
pub fn delete_item(_dgraph: &Arc<Dgraph>, uid: u64) -> bool {
    let deleted;

    let query = r#"query all($a: string){
        items(func: uid($a)) {
            expand(_all_)
        }
    }"#
    .to_string();

    let mut vars = HashMap::new();
    vars.insert("$a".to_string(), uid.to_string());

    let resp = _dgraph
    .new_readonly_txn()
    .query_with_vars(query, vars)
    .expect("query");

    let items: Value = serde_json::from_slice(&resp.json).unwrap();

    let null_item: Value = serde_json::from_str(r#"{"items": []}"#).unwrap();

    if items == null_item {

        deleted = bool::from(false);

    } else {
        let mut txn = _dgraph.new_txn();
        let mut mutation = dgraph::Mutation::new();

        let _json = serde_json::json!({"uid": uid});

        mutation.set_delete_json(serde_json::to_vec(&_json).expect("Failed to serialize JSON."));
        let _resp = txn.mutate(mutation).expect("Failed to create data.");
        txn.commit().expect("Failed to commit mutation");

        deleted = bool::from(true);
    }

    deleted
}
