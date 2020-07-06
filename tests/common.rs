extern crate pod;

use lazy_static::lazy_static;
use pod::database_init;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::fs::create_dir_all;
use std::fs::remove_file;
use std::path::PathBuf;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./res/migrations");
}

// Create a SQL connection pool to a tests database
lazy_static! {
    pub static ref SQLITE: Pool<SqliteConnectionManager> = {
        let sqlite_file = PathBuf::from("./tests/test.db");
        let sqlite_dir = sqlite_file
            .parent()
            .expect("Failed to get parent directory for database");
        remove_file("./tests/test.db").expect("Failed to remove database file");
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
        sqlite
    };
}
