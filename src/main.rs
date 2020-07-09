extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

mod api_model;
pub mod database_init;
mod error;
pub mod internal_api;
mod sql_converters;
mod warp_api;

use chrono::Utc;
use env_logger::Env;
use log::info;
use log::warn;
use pnet::datalink;
use r2d2::ManageConnection;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::env;
use std::fs::create_dir_all;
use std::io::Write;
use std::net::IpAddr::V4;
use std::net::IpAddr::V6;
use std::path::PathBuf;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./res/migrations");
}

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
        env!("GIT_DESCRIBE"),
        env!("CARGO_PKG_VERSION")
    );

    let sqlite_file = PathBuf::from("data/db/pod.db");
    info!("Using SQLite database {:?}", sqlite_file);
    let sqlite_dir = sqlite_file
        .parent()
        .expect("Failed to get parent directory for database");
    create_dir_all(sqlite_dir).expect("Failed to create database directory");
    let sqlite_manager = SqliteConnectionManager::file(&sqlite_file)
        .with_init(|c| c.execute_batch("PRAGMA foreign_keys = ON;"));

    let mut refinery_connection = sqlite_manager
        .connect()
        .expect("Failed to open a connection for refinery database migrations");
    // Run "refinery" migrations to bring the core structure of items/edges up-to-date
    embedded::migrations::runner()
        .run(&mut refinery_connection)
        .expect("Failed to run refinery migrations");

    let sqlite: Pool<SqliteConnectionManager> =
        r2d2::Pool::new(sqlite_manager).expect("Failed to create r2d2 SQLite connection pool");
    // Run "schema" migrations based on the auto-generated JSON schema.
    // This creates all optional properties in items table, and adds/removes property indices.
    database_init::init(&sqlite);

    // Try to prevent Pod from running on a public IP
    for interface in datalink::interfaces() {
        for ip in interface.ips {
            // https://en.wikipedia.org/wiki/Private_network
            let is_private = match ip.ip() {
                V4(v4) => v4.is_private() || v4.is_loopback() || v4.is_link_local(),
                V6(v6) if v6.is_loopback() => true,
                // https://en.wikipedia.org/wiki/Unique_local_address
                // Implementation copied from `v6.is_unique_local()`,
                // which is not yet stabilized in Rust
                V6(v6) if (v6.segments()[0] & 0xfe00) == 0xfc00 => true,
                // https://en.wikipedia.org/wiki/Link-local_address
                V6(v6) if (v6.segments()[0] & 0xffc0) == 0xfe80 => true,
                _ => false,
            };
            if !is_private && env::var_os("INSECURE_USE_PUBLIC_IP").is_some() {
                warn!("USING INSECURE PUBLIC IP {}.", ip.ip());
            } else if !is_private {
                eprintln!("ERROR! Pod seems to be running on a public network with IP {}. THIS IS NOT SECURE! Please wait until proper authorization is implemented, or do not run Pod on a publicly available network, or set environment variable INSECURE_USE_PUBLIC_IP to any value to override the failsafe.", ip.ip());
                std::process::exit(1)
            }
        }
    }

    // Start web framework
    warp_api::run_server(sqlite).await;
}
