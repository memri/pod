extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

mod data_model;
mod database_schema_generator;
mod internal_api;
mod warp_api;

use chrono::Utc;
use env_logger::Env;
use std::io::Write;
use r2d2_sqlite::SqliteConnectionManager;

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

    let sqlite_file = "pod.db";
    let sqlite = SqliteConnectionManager::file(sqlite_file);
    // .unwrap_or_else(|err| panic!("Could not open {}, {}", sqlite_file, err));

    // Start web framework
    warp_api::run_server(sqlite).await;
}
