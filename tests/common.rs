extern crate pod;
extern crate rusqlite;

use lazy_static::lazy_static;
use pod::database_init;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use rusqlite::OpenFlags;
use std::path::PathBuf;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./res/migrations");
}

// Create a SQL connection pool to a tests database
lazy_static! {
    pub static ref SQLITE: Pool<SqliteConnectionManager> = {
        // Create a new rusqlite connection for migration. This is a suboptimal solution for now,
        // and should be improved later to use the existing connection manager (TODO)
        let mut conn = Connection::open_with_flags("file:memdb?mode=memory&cache=shared", OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_READ_WRITE)
        .expect("Failed to open in-memory database for refinery migrations");
        embedded::migrations::runner()
        .run(&mut conn)
        .expect("Failed to run refinery migrations");

        let sqlite = SqliteConnectionManager::file(PathBuf::from("file:memdb?mode=memory&cache=shared")).with_flags(OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_READ_WRITE)
            .with_init(|c| c.execute_batch("PRAGMA foreign_keys = ON;"));
        let sqlite: Pool<SqliteConnectionManager> =
        r2d2::Pool::new(sqlite).expect("Failed to create r2d2 SQLite connection pool");
        database_init::init(&sqlite);
        conn.close().expect("Failed to close connection");
        sqlite
    };
}
