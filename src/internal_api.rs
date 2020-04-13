use dgraph::Dgraph;
use log::debug;
use std::sync::Arc;

pub fn version() -> &'static str {
    debug!("Returning API version...");
    env!("CARGO_PKG_VERSION")
}

pub fn hello(user_name: String, server_name: &str) -> String {
    debug!("Saying hello to user {}", user_name);
    format!("Hello, {} from server {}!", user_name, &server_name)
}

pub fn get_item(_dgraph: &Arc<Dgraph>, uid: u64) -> String {
    debug!("Getting item {}", uid);
    unimplemented!();
}
