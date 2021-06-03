extern crate rusqlite;

mod api_model;
mod command_line_interface;
mod constants;
pub mod database_api;
mod database_migrate_refinery;
mod database_migrate_schema;
mod error;
mod file_api;
mod global_static;
mod internal_api;
mod plugin_auth_crypto;
mod plugin_run;
mod schema;
mod triggers;
mod warp_api;
mod warp_endpoints;

use chrono::Utc;
use command_line_interface::CliOptions;
use env_logger::Env;
use internal_api::get_project_version;
use log::error;
use log::info;
use std::fs::create_dir_all;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

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
    let cli_options: CliOptions = command_line_interface::PARSED.clone();
    if cli_options.validate_schema {
        if let Err(err) = database_migrate_schema::validate_schema_file(&cli_options.schema_file) {
            log::error!("Schema validation failed: {}", err);
            std::process::exit(1)
        } else {
            log::info!("Schema is valid!");
            std::process::exit(0)
        }
    };
    info!("Starting Pod version {}", get_project_version());
    info!("Running Pod with configuration {:#?}", cli_options);

    create_config_directory(constants::DATABASE_DIR);
    create_config_directory(constants::FILES_DIR);

    // Start web framework
    warp_api::run_server(cli_options).await;
}

fn create_config_directory(path: &str) {
    if let Ok(path) = PathBuf::from_str(path) {
        create_dir_all(path).expect("Failed to create database directory");
    } else {
        error!("Failed to parse database directory {:?}", path);
        std::process::exit(1)
    };
}
