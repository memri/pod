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

pub fn set_item(_dgraph: &Arc<Dgraph>, json: Value) -> u64 {
    debug!("Setting item {}", json);
    unimplemented!();
}
