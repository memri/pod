// Not a real configuration, but one place that lists all environment variable with their usages.

use std::env;

pub const DATABASE_DIR: &str = "./data/db";

pub fn pod_is_in_docker() -> bool {
    env::var_os("POD_IS_IN_DOCKER").is_some()
}

/// Comma-separated list of ownerKey-s (hashes) that the server should respond to.
///
/// See /docs/HTTP_API.md on how it works
pub fn pod_owners() -> Option<String> {
    env::var("POD_OWNER_HASHES").ok()
}
