use crate::error::Result;
use refinery::Runner;
use rusqlite::Connection;

pub mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./res/migrations");
}

/// Run "refinery" migrations to bring the core structure of items/edges up-to-date
pub fn migrate(conn: &mut Connection) -> Result<()> {
    let runner: Runner = embedded::migrations::runner();
    runner.run(conn).map(|_report| ()).map_err(|e| e.into())
}
