extern crate pod;
extern crate rusqlite;

use lazy_static::lazy_static;
use pod::database_init;
use r2d2::ManageConnection;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::OpenFlags;
use std::path::PathBuf;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./res/migrations");
}

// Create a SQL connection pool to a tests database
lazy_static! {
    pub static ref SQLITE: Pool<SqliteConnectionManager> = {
        let sqlite_manager =
            SqliteConnectionManager::file(PathBuf::from("file:memdb?mode=memory&cache=shared"))
                .with_flags(OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_READ_WRITE)
                .with_init(|c| c.execute_batch("PRAGMA foreign_keys = ON;"));
        let mut refinery_connection = sqlite_manager
            .connect()
            .expect("Failed to open a connection for refinery database migrations");
        embedded::migrations::runner()
            .run(&mut refinery_connection)
            .expect("Failed to run refinery migrations");

        let sqlite: Pool<SqliteConnectionManager> =
            r2d2::Pool::new(sqlite_manager).expect("Failed to create r2d2 SQLite connection pool");
        database_init::init(&sqlite);
        sqlite
    };
}
