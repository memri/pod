extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

mod api_model;
mod command_line_interface;
mod constants;
mod database_migrate_refinery;
pub mod database_migrate_schema;
mod error;
pub mod file_api;
pub mod internal_api;
pub mod services_api;
mod sql_converters;
mod warp_api;
mod warp_endpoints;

use chrono::Utc;
use env_logger::Env;
use log::error;
use log::info;
use std::fs::create_dir_all;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

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
    let opt = command_line_interface::CLIOptions::from_args();
    println!("{:#?}", opt);

    if std::env::args().any(|a| a == "--version" || a == "--help") {
        eprintln!("Done");
        std::process::exit(0)
    };

    create_config_directory(constants::DATABASE_DIR);
    create_config_directory(constants::MEDIA_DIR);

    // Start web framework
    warp_api::run_server().await;
}

fn create_config_directory(path: &str) {
    if let Ok(path) = PathBuf::from_str(path) {
        create_dir_all(path).expect("Failed to create database directory");
    } else {
        error!("Failed to parse database directory {:?}", path);
        std::process::exit(1)
    };
}
