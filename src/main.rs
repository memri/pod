extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

mod database_schema_generator;
mod error;
mod internal_api;
mod warp_api;

use chrono::Utc;
use env_logger::Env;
use r2d2_sqlite::SqliteConnectionManager;
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

    let sqlite_file = "pod.db";
    let sqlite = SqliteConnectionManager::file(sqlite_file);

    // Start web framework
    warp_api::run_server(sqlite).await;
}
