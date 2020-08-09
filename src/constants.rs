// Constants used in the project. These are "convention over configuration" for now.

use std::env;

// database settings:

pub const DATABASE_DIR: &str = "./data/db";

// docker settings:

pub fn pod_is_in_docker() -> bool {
    env::var_os("POD_IS_IN_DOCKER").is_some()
}

// http settings:

/// Address that Pod will start to listen when started
pub fn pod_listen_address() -> Option<String> {
    std::env::var("POD_LISTEN_ADDRESS").ok()
}

/// If Pod listen address is not set above, this port will be listened to.
pub const DEFAULT_PORT: u32 = 3030;

pub const HTTPS_CERTIFICATE_ENV_NAME: &str = "POD_HTTPS_CERTIFICATE";
pub fn https_certificate_file() -> Option<String> {
    std::env::var(HTTPS_CERTIFICATE_ENV_NAME).ok()
}

pub const USE_INSECURE_NON_TLS_ENV_NAME: &str = "POD_USE_INSECURE_NON_TLS";
pub fn use_insecure_non_tls() -> bool {
    std::env::var(USE_INSECURE_NON_TLS_ENV_NAME).is_ok()
}

// authorization settings:

/// Comma-separated list of ownerKey-s (hashes) that the server should respond to.
///
/// See /docs/HTTP_API.md on how it works
pub fn pod_owners() -> Option<String> {
    env::var("POD_OWNER_HASHES").ok()
}

// file settings:
pub const MEDIA_DIR: &str = "./data/media";
