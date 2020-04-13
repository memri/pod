use dgraph::Dgraph;
use log::debug;
use serde_json::Value;
use std::sync::Arc;

pub fn version() -> &'static str {
    debug!("Returning API version...");
    env!("CARGO_PKG_VERSION")
}

pub fn get_item(_dgraph: &Arc<Dgraph>, uid: u64) -> Value {
    debug!("Getting item {}", uid);
    unimplemented!();
}

pub fn create_item(_dgraph: &Arc<Dgraph>, json: Value) -> u64 {
    debug!("Creating item {}", json);
    unimplemented!();
}

pub fn update_item(_dgraph: &Arc<Dgraph>, uid: u64, json: Value) {
    debug!("Updating item {} with {}", uid, json);
    unimplemented!();
}
