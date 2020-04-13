use dgraph::Dgraph;
use log::debug;
use std::sync::Arc;
use serde_json::Value;

pub fn version() -> &'static str {
    debug!("Returning API version...");
    env!("CARGO_PKG_VERSION")
}

pub fn get_item(_dgraph: &Arc<Dgraph>, uid: u64) -> String {
    debug!("Getting item {}", uid);
    unimplemented!();
}

pub fn set_item(_dgraph: &Arc<Dgraph>, json: Value) -> String {
    debug!("Setting item {}", json);
    unimplemented!();
}
