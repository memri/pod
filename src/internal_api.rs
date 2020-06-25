use crate::error::Error;
use crate::error::Result;
use crate::sql_converters::json_value_to_sqlite_parameter;
use crate::sql_converters::sqlite_rows_to_json;
use log::debug;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::ToSql;
use rusqlite::NO_PARAMS;
use serde_json::value::Value::Object;
use serde_json::Value;
use std::str;
use warp::http::status::StatusCode;

/// Get project version as seen by Cargo.
pub fn get_project_version() -> &'static str {
    debug!("Returning API version...");
    env!("CARGO_PKG_VERSION")
}

/// Get an item by its `id`
pub fn get_item(sqlite: &Pool<SqliteConnectionManager>, id: i64) -> Result<Vec<Value>> {
    debug!("Getting item {}", id);
    let conn = sqlite.get()?;

    let mut stmt = conn.prepare_cached("SELECT * FROM items WHERE id = :id")?;
    let rows = stmt.query_named(&[(":id", &id)])?;

    let json = sqlite_rows_to_json(rows)?;
    Ok(json)
}

pub fn get_all_items(sqlite: &Pool<SqliteConnectionManager>) -> Result<Vec<Value>> {
    debug!("Getting all items");
    let conn = sqlite.get()?;
    let mut stmt = conn.prepare_cached("SELECT * FROM items")?;
    let rows = stmt.query(NO_PARAMS)?;
    let json = sqlite_rows_to_json(rows)?;
    Ok(json)
}

/// Create an item, failing if the `id` existed before.
/// The new item will be created with `version = 1`.
pub fn create_item(_sqlite: &Pool<SqliteConnectionManager>, json: Value) -> Option<u64> {
    debug!("Creating item {}", json);
    unimplemented!()
}

/// Update an item with a JSON object.
/// Json `null` fields will be erased from the database.
/// Nonexisting or reserved properties like "version" will cause error (TODO).
/// The version of the item in the database will be increased `version += 1`.
pub fn update_item(_sqlite: &Pool<SqliteConnectionManager>, i64_id: String, json: Value) -> bool {
    debug!("Updating item {} with {}", i64_id, json);
    unimplemented!()
}

/// Delete an already existing item.
/// Return successfully if item existed and was successfully deleted.
/// Return NOT_FOUND if item does not exist.
pub fn delete_item(sqlite: &Pool<SqliteConnectionManager>, id: i64) -> Result<()> {
    debug!("Deleting item {}", id);
    let conn = sqlite.get()?;

    let mut stmt = conn.prepare_cached("DELETE FROM items WHERE id = :id")?;
    let deleted = stmt.execute_named(&[(":id", &id)])?;
    if deleted == 0 {
        return Err(Error {
            code: StatusCode::NOT_FOUND,
            msg: "No such item".to_string(),
        });
    }
    Ok(())
}

/// Search items by their fields
///
/// * `query` - json with the desired fields, e.g. { "author": "Vasili", "_type": "note" }
pub fn search(sqlite: &Pool<SqliteConnectionManager>, query: Value) -> Result<Vec<Value>> {
    debug!("Query {:?}", query);
    let fields_map = match query {
        Object(map) => map,
        _ => {
            return Err(Error {
                code: StatusCode::BAD_REQUEST,
                msg: "Expected JSON object".to_string(),
            })
        }
    };
    let mut sql_body = "SELECT * FROM items WHERE ".to_string();
    let mut first_parameter = true;
    for field in fields_map.keys() {
        if !first_parameter {
            sql_body.push_str(" AND ")
        };
        first_parameter = false;
        sql_body.push_str(field); // TODO: prevent SQL injection! See GitLab issue #84
        sql_body.push_str(" = :");
        sql_body.push_str(field);
    }
    sql_body.push_str(";");

    let mut sql_params = Vec::new();
    for (field, value) in &fields_map {
        let field = format!(":{}", field);
        sql_params.push((field, json_value_to_sqlite_parameter(value)));
    }
    let sql_params: Vec<_> = sql_params
        .iter()
        .map(|(field, value)| (field.as_str(), value as &dyn ToSql))
        .collect();
    let conn = sqlite.get()?;
    let mut stmt = conn.prepare_cached(&sql_body)?;
    let rows = stmt.query_named(sql_params.as_slice())?;
    let json = sqlite_rows_to_json(rows)?;
    Ok(json)
}
