use dgraph::Dgraph;
use log::debug;
use serde_json::Value;
use std::sync::Arc;

pub fn version() -> &'static str {
    debug!("Returning API version...");
    env!("CARGO_PKG_VERSION")
}

/// Get an item from the dgraph database.
/// None if the `uid` doesn't exist in DB, Some(json) if it does.
pub fn get_item(_dgraph: &Arc<Dgraph>, uid: u64) -> Option<Value> {
    debug!("Getting item {}", uid);
    unimplemented!();
}

/// Create an item presuming it didn't exist before.
/// Returns Some(uid) if the data item did not have a `uid` field, and was successfully put in DB.
/// Returns None if the data item had `uid` field, which indicates
/// the misuse of `create` method. Use `update` method instead then.
///
/// New DataItem-s will be created with `version = 1`.
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
