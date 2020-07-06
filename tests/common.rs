extern crate pod;

use lazy_static::lazy_static;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::fs::create_dir_all;
use std::path::PathBuf;

// Create a SQL connection pool to a tests database
lazy_static! {
    pub static ref SQLITE: Pool<SqliteConnectionManager> = {
        let sqlite_file = PathBuf::from("./data/db/pod.db");
        let sqlite_dir = sqlite_file
            .parent()
            .expect("Failed to get parent directory for database");
        create_dir_all(sqlite_dir).expect("Failed to create database directory");
        let sqlite = SqliteConnectionManager::file(&sqlite_file)
            .with_init(|c| c.execute_batch("PRAGMA foreign_keys = ON;"));
        r2d2::Pool::new(sqlite).expect("Failed to create r2d2 SQLite connection pool")
    };
}
