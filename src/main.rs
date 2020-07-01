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
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::fs::create_dir_all;
use std::io::Write;

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

    create_dir_all("data/db").expect("Failed to create data/db");
    let sqlite_file = "data/db/pod.db";
    let sqlite = SqliteConnectionManager::file(sqlite_file);
    let sqlite: Pool<SqliteConnectionManager> =
        r2d2::Pool::new(sqlite).expect("Failed to create r2d2 SQLite connection pool");

    // Create a new rusqlite connection for migration, this is a suboptimal solution for now,
    // and should be improved later to use the existing connection manager (TODO)
    let mut conn = rusqlite::Connection::open(sqlite_file)
        .unwrap_or_else(|_| panic!("Cannot open database file {}", sqlite_file));
    embedded::migrations::runner()
        .run(&mut conn)
        .expect("Cannot run migration");
    conn.close().expect("Cannot close connection");

    database_init::init(&sqlite);

    // Start web framework
    warp_api::run_server(sqlite).await;
}
