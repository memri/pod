extern crate pod;

use lazy_static::lazy_static;
use pod::database_migrate_refinery;
use pod::database_migrate_schema;
use pod::error::Error;
use pod::internal_api::*;
use r2d2::ManageConnection;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::OpenFlags;
use serde_json::json;
use serde_json::Value;
use std::path::PathBuf;

lazy_static! {
    static ref SQLITE: Pool<SqliteConnectionManager> = {
        let sqlite =
            SqliteConnectionManager::file(PathBuf::from("file:memdb?mode=memory&cache=shared"))
                .with_flags(OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_READ_WRITE)
                .with_init(|c| c.execute_batch("PRAGMA foreign_keys = ON;"));

        // Cannot re-use `migrate` function from `database_migrate_refinery`.
        // Probably due to some weird lifetime problem, didn't properly investigate yet
        let mut refinery_connection = sqlite
            .connect()
            .expect("Failed to open a connection for refinery database migrations");
        // Run "refinery" migrations to bring the core structure of items/edges up-to-date
        database_migrate_refinery::embedded::migrations::runner()
            .run(&mut refinery_connection)
            .expect("Failed to run refinery migrations");
        // database_migrate_refinery::migrate(&sqlite);

        let sqlite: Pool<SqliteConnectionManager> =
            r2d2::Pool::new(sqlite).expect("Failed to create r2d2 SQLite connection pool");
        database_migrate_schema::migrate(&sqlite)
            .unwrap_or_else(|err| panic!("Failed to migrate schema, {}", err));
        sqlite
    };
}

#[test]
fn test_bulk_action() {
    let sqlite = &SQLITE;

    let json = json!({
        "createItems": [{"uid": 1, "_type": "Person"}, {"uid": 2, "_type": "Person"}],
        "updateItems": [{"uid": 1, "_type": "Person1"}],
        "createEdges": [{"_type": "friend", "_source": 1, "_target": 2, "edgeLabel": "test", "sequence": 1}]
    });

    let bulk = bulk_action(&sqlite, json);

    let edges = get_item_with_edges(&sqlite, 1);

    let json = json!({"_type": "Person"});
    let search = search_by_fields(&sqlite, json);

    assert_eq!(bulk, Ok(()));
    assert!(
        edges.is_ok(),
        "get items with edges failed with: {:?}",
        edges
    );
    assert!(check_has_item(search));
}

fn check_has_item(result: Result<Vec<Value>, Error>) -> bool {
    result.iter().flatten().next().is_some()
}
