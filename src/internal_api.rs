use crate::data_model;
use crate::data_model::AuditAccessLog;
use crate::data_model::NodeReference;
use crate::data_model::UID;
use crate::sync_state;
use bytes::Bytes;
use chrono::DateTime;
use chrono::Utc;
use dgraph::Dgraph;
use log::debug;
use log::trace;
use serde_json::to_string_pretty;
use serde_json::Map;
use serde_json::Value;
use std::collections::HashMap;
use std::ops::Deref;
use std::str;

/// Verify if the response contains the content of the requested node.
/// Return `true` if response doesn't contain any content.
/// Return `false` if the node is found and its content is included in the response.
fn response_is_empty(result: &Value) -> bool {
    let result = result.get("items").unwrap().as_array().unwrap();
    result.is_empty() || result.first().unwrap().get("type").is_none()
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
pub fn get_item(dgraph: &Dgraph, memri_id: String) -> Option<String> {
    debug!("Getting item {}", memri_id);
    let query = r#"query all($a: string){
        items(func: eq(memriID, $a)) {
            type : dgraph.type
            expand(_all_)
        }
    }"#;

    let mut vars = HashMap::new();
    vars.insert("$a".to_string(), memri_id);

    let resp = dgraph
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
pub fn get_all_items(dgraph: &Dgraph) -> Option<String> {
    let query = r#"{
            items(func: has(version)) {
                type : dgraph.type
                expand(_all_)
                }
            }"#;
    let resp = dgraph.new_readonly_txn().query(query).expect("query");

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
pub fn create_item(dgraph: &Dgraph, json: Value) -> Option<u64> {
    debug!("Creating item {}", json);
    let new_json: Map<String, Value> = json.clone().as_object().unwrap().clone();
    // Check for blank node labels
    let has_blank_label = new_json.contains_key("uid");
    if !has_blank_label
        || new_json
            .get("uid")
            .unwrap()
            .as_str()
            .unwrap()
            .starts_with("_:")
    {
        let mut txn = dgraph.new_txn();
        let mut mutation = dgraph::Mutation::new();

        let version = 1;
        mutation.set_set_json(
            serde_json::to_vec(&sync_state::get_syncstate(json, version, Value::Null))
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
///     - `json`: the json sent by client.
/// Return `false` if dgraph didn't have a node with this `mid`.
/// Return `true` if dgraph had a node with this `mid` and it was successfully updated.
/// The `version` that is sent to us by the client should be completely ignored.
/// Use `expand(_all_)` to get all properties of an item, only works if data has a `dgraph.type`.
/// `syncState` from client json is processed and removed.
/// A successful update operation should also increase the version in dgraph as `version += 1`.
pub fn update_item(dgraph: &Dgraph, memri_id: String, json: Value) -> bool {
    debug!("Updating item {} with {}", memri_id, json);
    let found;

    let query = r#"query all($a: string){
        items(func: eq(memriID, $a)) {
            uid
            expand(_all_)
        }
    }"#;

    let mut vars = HashMap::new();
    vars.insert("$a".to_string(), memri_id);

    let resp = dgraph
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
        let json = sync_state::get_syncstate(json, new_ver, uid.clone());

        // Update via mutation
        let mut txn = dgraph.new_txn();
        let mut mutation = dgraph::Mutation::new();

        mutation.set_set_json(serde_json::to_vec(&json).expect("Failed to serialize JSON."));
        let _resp = txn.mutate(mutation).expect("Failed to create data.");
        txn.commit().expect("Failed to commit mutation");

        found = true;
    }
    found
}

/// Delete an already existing item.
/// Return `false` if dgraph didn't have a node with this memriID.
/// Return `true` if dgraph had a node with this memriID and it was successfully deleted.
pub fn delete_item(dgraph: &Dgraph, memri_id: String) -> bool {
    debug!("Deleting item {}", memri_id);
    let deleted;

    let query = r#"query all($a: string){
        items(func: eq(memriID, $a)) {
            uid
            expand(_all_)
        }
    }"#;

    let mut vars = HashMap::new();
    vars.insert("$a".to_string(), memri_id);

    let resp = dgraph
        .new_readonly_txn()
        .query_with_vars(query, vars)
        .expect("query");

    let items: Value = serde_json::from_slice(&resp.json).unwrap();
    let null_item: Value = serde_json::from_str(r#"{"items": []}"#).unwrap();

    if items == null_item {
        deleted = false;
    } else {
        let mut txn = dgraph.new_txn();
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
pub fn query(dgraph: &Dgraph, body: Bytes) -> Option<String> {
    let query = std::str::from_utf8(body.deref()).unwrap();
    debug!("Query {}", query);

    let resp = dgraph.new_readonly_txn().query(query).expect("query");
    let result: Value = serde_json::from_slice(&resp.json).unwrap();
    if response_is_empty(&result) {
        None
    } else {
        Some(sync_state::set_syncstate_all(result))
    }
}

pub fn _write_access_audit_log(dgraph: &Dgraph, underlying_uid: UID) {
    let audit = AuditAccessLog {
        audit_target: NodeReference {
            uid: underlying_uid,
        },
        date_created: Utc::now(),
    };
    trace!("Adding audit entry: {}", to_string_pretty(&audit).unwrap());
    let mut mutation = dgraph::Mutation::new();
    mutation.set_set_json(serde_json::to_vec(&audit).expect("Failed to serialize to JSON"));

    let mut tx = dgraph.new_txn();
    tx.mutate(mutation).expect("Failed to create audit log");
    tx.commit().expect("Failed to commit mutation");
}

/// Given a node "type" and a date,
/// find all nodes of the specified "type"
/// that were accessed by the user after the specified date.
/// Return the specified fields only (parameter `fields`).
///
/// User access is defined in terms of access log entries.
pub fn _get_updates(
    _dgraph: &Dgraph,
    node_type: &str,
    date_from: DateTime<Utc>,
    fields: &[&str],
) -> bool {
    debug!(
        "Getting updates for node type {} starting from {} and limiting the result fields to {:?}",
        node_type, date_from, fields
    );
    // research what it means to write in dgraph
    // research how to filter audit logs by "audit" type and date
    // research how, given all audit logs, get all uid-s

    panic!()
}
