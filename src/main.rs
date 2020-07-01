extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

mod database_init;
mod error;
mod internal_api;
mod sql_converters;
mod warp_api;

use chrono::Utc;
use env_logger::Env;
use lazy_static::lazy_static;
use log::info;
use pnet::datalink;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use regex::RegexSet;
use std::fs::create_dir_all;
use std::io::Write;
use std::path::PathBuf;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./res/migrations");
}

pub fn abort_at_public_ip(ip: &str) {
    lazy_static! {
        static ref REGEXP: RegexSet = RegexSet::new(&[
            r"10\.\d{1,3}\.\d{1,3}\.\d{1,3}",
            r"172\.1[6-9]\.\d{1,3}\.\d{1,3}",
            r"172\.2[0-9]\.\d{1,3}\.\d{1,3}",
            r"172\.3[0-1]\.\d{1,3}\.\d{1,3}",
            r"192\.168\.\d{1,3}\.\d{1,3}",
            r"127\.\d{1,3}\.\d{1,3}\.\d{1,3}",
        ])
        .expect("Cannot create regex");
    }
    if !REGEXP.is_match(ip) {
        panic!(format!("DO NOT RUN WITH PUBLIC IP {}", ip));
    }
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
    let sqlite = SqliteConnectionManager::file(&sqlite_file);
    let sqlite: Pool<SqliteConnectionManager> =
        r2d2::Pool::new(sqlite).expect("Failed to create r2d2 SQLite connection pool");

    // Create a new rusqlite connection for migration, this is a suboptimal solution for now,
    // and should be improved later to use the existing connection manager (TODO)
    let mut conn = rusqlite::Connection::open(&sqlite_file)
        .expect("Failed to open database for refinery migrations");
    embedded::migrations::runner()
        .run(&mut conn)
        .expect("Failed to run refinery migrations");
    conn.close().expect("Failed to close connection");

    // Add auto-generated schema into database
    database_init::init(&sqlite);

    // Warn if pod is running with a public IP
    for interface in datalink::interfaces() {
        for ip in interface.ips {
            if ip.is_ipv4() {
                abort_at_public_ip(&ip.ip().to_string());
            }
        }
    }

    // Start web framework
    warp_api::run_server(sqlite).await;
}
