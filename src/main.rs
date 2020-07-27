extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

mod api_model;
mod configuration;
mod database_migrate_refinery;
pub mod database_migrate_schema;
mod error;
pub mod internal_api;
pub mod services_api;
mod sql_converters;
mod warp_api;
mod warp_endpoints;

use chrono::Utc;
use env_logger::Env;
use log::info;
use std::io::Write;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().filter_or("RUST_LOG", "info"))
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Utc::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();
    info!(
        "Starting Pod version {} (Cargo version {})",
        std::env!("GIT_DESCRIBE"),
        std::env!("CARGO_PKG_VERSION")
    );
    if std::env::args().any(|a| a == "--version" || a == "--help") {
        eprintln!("Done");
        std::process::exit(0)
    };

    // Start web framework
    warp_api::run_server().await;
}
