use dgraph::Dgraph;
use log::debug;
use std::sync::Arc;
use std::collections::HashMap;
use serde_json::Value;

pub fn version() -> &'static str {
    debug!("Returning API version...");
    env!("CARGO_PKG_VERSION")
}


/// Get an item from the dgraph database.
/// None if the `uid` doesn't exist in DB, Some(json) if it does.
pub fn get_item(_dgraph: &Arc<Dgraph>, uid: String) -> Option<String> {
    let query = r#"query all($a: string){
    items(func: uid($a)) {
        uid
        deleted
        starred
        version
    }
    }"#
    .to_string();

    let mut vars = HashMap::new();
    vars.insert("$a".to_string(), uid.to_string());

    let resp = _dgraph
        .new_readonly_txn()
        .query_with_vars(query, vars)
        .expect("query");

    let str = std::str::from_utf8(&resp.json).unwrap();

    Some(str.parse().unwrap())
}

pub fn get_all_item(_dgraph: &Arc<Dgraph>) -> String {
    let query = format!(
        r#"{{
            items(func: has(deleted)) {{
                uid
                deleted
                starred
                version
                }}
            }}"#
    );

    let resp = _dgraph
        .new_readonly_txn()
        .query(query)
        .expect("query");

    let str = std::str::from_utf8(&resp.json).unwrap();

    str.parse().unwrap()
}

/// Create an item presuming it didn't exist before.
/// Some(uid) the new data item did not have `uid` and was successfully pud in DB.
/// None if the data item had `uid` field, which indicates
/// the misuse of `create` method. Use `update` method instead then.
///
/// New DataItems should have `version = 1`.
pub fn create_item(_dgraph: &Arc<Dgraph>, json: Value) -> Option<u64> {
    debug!("Creating item {}", json);
    unimplemented!();
}

/// Update an already existing item.
/// `false` if dgraph didn't have a node with this `uid`.
/// `true` if dgraph had a node with this `uid` and it was successfully updated.
///
/// A successful update operation should also
/// increase `version += 1`, **comparing to the version that was in DB**.
/// The `version` that is sent to us by the client should be completely ignored.
pub fn update_item(_dgraph: &Arc<Dgraph>, uid: u64, json: Value) -> bool {
    debug!("Updating item {} with {}", uid, json);
    unimplemented!();
}

