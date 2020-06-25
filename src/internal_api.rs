use crate::error::Error;
use crate::error::Result;
use crate::sql_converters::sqlite_rows_to_json;
use crate::sql_converters::{json_value_to_sqlite_parameter, sqlite_value_to_json};
use chrono::Utc;
use log::debug;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::ToSql;
use rusqlite::NO_PARAMS;
use serde_json::value::Value::Object;
use serde_json::Map;
use serde_json::Value;
use std::str;
use warp::http::status::StatusCode;

/// Check if item exists by id
pub fn item_exist(
    conn: &PooledConnection<SqliteConnectionManager>,
    fields_map: &Map<String, Value>,
) -> bool {
    if let Some(id) = fields_map.get("id") {
        let id = id.as_i64().expect("Value is not i64");
        let sql = format!("SELECT COUNT(*) FROM items WHERE id = {};", id);
        let result: i64 = conn
            .query_row(&sql, NO_PARAMS, |row| row.get(0))
            .expect("Failed to query SQLite column information");
        if result != 0 {
            return true;
        }
    }
    false
}

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
pub fn create_item(sqlite: &Pool<SqliteConnectionManager>, json: Value) -> Result<Value> {
    debug!("Creating item {}", json);
    let fields_map = match json {
        Object(map) => map,
        _ => {
            return Err(Error {
                code: StatusCode::BAD_REQUEST,
                msg: "Expected JSON object".to_string(),
            })
        }
    };

    let conn = sqlite.get()?;
    if item_exist(&conn, &fields_map) {
        return Err(Error {
            code: StatusCode::CONFLICT,
            msg: "Request contains id, use update_item() instead".to_string(),
        });
    }

    let mut sql_body = "INSERT INTO items (".to_string();
    let mut sql_body_params = ") VALUES (:".to_string();
    let mut first_parameter = true;
    for field in fields_map.keys() {
        if !first_parameter {
            sql_body.push_str(", ");
            sql_body_params.push_str(", :")
        };
        first_parameter = false;
        sql_body.push_str(field); // TODO: prevent SQL injection! See GitLab issue #84
        sql_body_params.push_str(field);
    }
    sql_body.push_str(", created_at, modified_at, version");
    sql_body.push_str(sql_body_params.as_str());
    sql_body.push_str(", :created_at, :modified_at, :version");
    sql_body.push_str(");");

    let mut sql_params = Vec::new();
    for (field, value) in &fields_map {
        let field = format!(":{}", field);
        sql_params.push((field, json_value_to_sqlite_parameter(value)));
    }
    let created_at = Value::from(Utc::now().timestamp() as f64);
    sql_params.push((
        ":created_at".to_string(),
        json_value_to_sqlite_parameter(&created_at),
    ));
    let modified_at = Value::from(0 as f64);
    sql_params.push((
        ":modified_at".to_string(),
        json_value_to_sqlite_parameter(&modified_at),
    ));
    let version = Value::from(1);
    sql_params.push((
        ":version".to_string(),
        json_value_to_sqlite_parameter(&version),
    ));
    let sql_params: Vec<_> = sql_params
        .iter()
        .map(|(field, value)| (field.as_str(), value as &dyn ToSql))
        .collect();

    let mut stmt = conn.prepare_cached(&sql_body)?;
    stmt.execute_named(sql_params.as_slice())?;
    let json = serde_json::json!({"id": conn.last_insert_rowid()});
    Ok(json)
}

/// Update an item with a JSON object.
/// Json `null` fields will be erased from the database.
/// Nonexisting or reserved properties like "version" will cause error (TODO).
/// The version of the item in the database will be increased `version += 1`.
pub fn update_item(sqlite: &Pool<SqliteConnectionManager>, id: i64, json: Value) -> Result<()> {
    debug!("Updating item {} with {}", id, json);
    let fields_map = match json {
        Object(map) => map,
        _ => {
            return Err(Error {
                code: StatusCode::BAD_REQUEST,
                msg: "Expected JSON object".to_string(),
            })
        }
    };

    let conn = sqlite.get()?;
    let mut stmt = conn.prepare_cached("SELECT version FROM items WHERE id = :id")?;
    let mut rows = stmt.query_named(&[(":id", &id)])?;
    let mut values = Map::new();
    if let Some(row) = rows.next()? {
        values.insert("version".to_string(), sqlite_value_to_json(row.get_raw(0)));
    } else {
        return Err(Error {
            code: StatusCode::NOT_FOUND,
            msg: "No such item".to_string(),
        });
    }
    let version = values
        .get("version")
        .expect("No value is found")
        .as_i64()
        .expect("Value is not i64");

    let mut sql_body = "UPDATE items SET ".to_string();
    let mut first_parameter = true;
    for field in fields_map.keys() {
        if !first_parameter {
            sql_body.push_str(", ");
        };
        first_parameter = false;
        sql_body.push_str(field); // TODO: prevent SQL injection! See GitLab issue #84
        sql_body.push_str(" = :");
        sql_body.push_str(field);
    }
    sql_body.push_str(", modified_at = :modified_at, version = :version");
    sql_body.push_str(" WHERE id = :id");

    let mut sql_params = Vec::new();
    for (field, value) in &fields_map {
        let field = format!(":{}", field);
        sql_params.push((field, json_value_to_sqlite_parameter(value)));
    }
    let modified_at = Value::from(Utc::now().timestamp() as f64);
    sql_params.push((
        ":modified_at".to_string(),
        json_value_to_sqlite_parameter(&modified_at),
    ));
    let new_version = Value::from(version + 1);
    sql_params.push((
        ":version".to_string(),
        json_value_to_sqlite_parameter(&new_version),
    ));
    let id_value = Value::from(id);
    sql_params.push((":id".to_string(), json_value_to_sqlite_parameter(&id_value)));
    let sql_params: Vec<_> = sql_params
        .iter()
        .map(|(field, value)| (field.as_str(), value as &dyn ToSql))
        .collect();

    let mut stmt = conn.prepare_cached(&sql_body)?;
    stmt.execute_named(sql_params.as_slice())?;
    Ok(())
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
