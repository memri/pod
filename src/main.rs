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
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::fs::create_dir_all;
use std::io::Write;
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
    database_init::init(&sqlite);

    // Start web framework
    warp_api::run_server(sqlite).await;
}
