extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

mod api_model;
mod database_init;
mod error;
mod internal_api;
mod sql_converters;
mod warp_api;

use chrono::Utc;
use env_logger::Env;
use log::info;
use log::warn;
use pnet::datalink;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::env;
use std::fs::create_dir_all;
use std::io::Write;
use std::net::IpAddr::V4;
use std::net::IpAddr::V6;
use std::net::Ipv6Addr;
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

    let sqlite_file = PathBuf::from("data/db/pod.db");
    info!("Using SQLite database {:?}", sqlite_file);
    let sqlite_dir = sqlite_file
        .parent()
        .expect("Failed to get parent directory for database");
    create_dir_all(sqlite_dir).expect("Failed to create database directory");

    // Create a new rusqlite connection for migration. This is a suboptimal solution for now,
    // and should be improved later to use the existing connection manager (TODO)
    let mut conn = rusqlite::Connection::open(&sqlite_file)
        .expect("Failed to open database for refinery migrations");
    embedded::migrations::runner()
        .run(&mut conn)
        .expect("Failed to run refinery migrations");
    conn.close().expect("Failed to close connection");

    let sqlite = SqliteConnectionManager::file(&sqlite_file)
        .with_init(|c| c.execute_batch("PRAGMA foreign_keys = ON;"));
    let sqlite: Pool<SqliteConnectionManager> =
        r2d2::Pool::new(sqlite).expect("Failed to create r2d2 SQLite connection pool");
    // Alter schema based on information from auto-generated iOS JSON schema
    database_init::init(&sqlite);

    // Try to prevent Pod from running on a public IP
    for interface in datalink::interfaces() {
        for ip in interface.ips {
            // https://en.wikipedia.org/wiki/Private_network
            let is_private = match ip.ip() {
                V4(v4) => v4.is_private(),
                V6(v6) if v6 == Ipv6Addr::LOCALHOST => true,
                // https://en.wikipedia.org/wiki/Unique_local_address
                // Implementation copied from `v6.is_unique_local()`,
                // which is not yet stabilized in Rust
                V6(v6) if (v6.segments()[0] & 0xfe00) == 0xfc00 => true,
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
