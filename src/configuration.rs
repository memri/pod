// Not a real configuration, but one place that lists all environment variable with their usages.

use std::env;

pub const DATABASE_DIR: &str = "./data/db";

pub fn pod_is_in_docker() -> bool {
    env::var_os("POD_IS_IN_DOCKER").is_some()
}

pub fn pod_address() -> Option<String> {
    std::env::var("POD_ADDRESS").ok()
}

pub const HTTPS_CERTIFICATE_ENV_NAME: &str = "POD_HTTPS_CERTIFICATE";

pub fn https_certificate_file() -> Option<String> {
    std::env::var(HTTPS_CERTIFICATE_ENV_NAME).ok()
}

pub const USE_INSECURE_NON_TLS_ENV_NAME: &str = "POD_USE_INSECURE_NON_TLS";

pub fn use_insecure_non_tls() -> bool {
    std::env::var(USE_INSECURE_NON_TLS_ENV_NAME).is_ok()
}

/// Comma-separated list of ownerKey-s (hashes) that the server should respond to.
///
/// See /docs/HTTP_API.md on how it works
pub fn pod_owners() -> Option<String> {
    env::var("POD_OWNER_HASHES").ok()
}
