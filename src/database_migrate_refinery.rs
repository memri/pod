use r2d2::ManageConnection;
use r2d2_sqlite::SqliteConnectionManager;

pub mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./res/migrations");
}

pub fn migrate(sqlite: &SqliteConnectionManager) {
    let mut refinery_connection = sqlite
        .connect()
        .expect("Failed to open a connection for refinery database migrations");
    // Run "refinery" migrations to bring the core structure of items/edges up-to-date
    embedded::migrations::runner()
        .run(&mut refinery_connection)
        .expect("Failed to run refinery migrations");
}
