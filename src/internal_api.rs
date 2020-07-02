use crate::error::Error;
use crate::error::Result;
use crate::sql_converters::borrow_sql_params;
use crate::sql_converters::fields_mapping_to_owned_sql_params;
use crate::sql_converters::json_value_to_sqlite_parameter;
use crate::sql_converters::map_to_json;
use crate::sql_converters::sqlite_rows_to_json;
use crate::sql_converters::sqlite_rows_to_map;
use crate::sql_converters::validate_field_name;
use chrono::Utc;
use log::debug;
use r2d2::Pool;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::NO_PARAMS;
use serde_json::value::Value::Object;
use serde_json::Map;
use serde_json::Value;
use std::str;
use warp::http::status::StatusCode;

/// Check if item exists by uid
pub fn _check_item_exist(
    conn: &PooledConnection<SqliteConnectionManager>,
    uid: i64,
) -> Result<bool> {
    let sql = format!("SELECT COUNT(*) FROM items WHERE uid = {};", uid);
    let result: i64 = conn.query_row(&sql, NO_PARAMS, |row| row.get(0))?;
    Ok(result != 0)
}

/// Get project version as seen by Cargo.
pub fn get_project_version() -> &'static str {
    debug!("Returning API version...");
    env!("CARGO_PKG_VERSION")
}

/// Get an item by its `uid`
pub fn get_item(sqlite: &Pool<SqliteConnectionManager>, uid: i64) -> Result<Vec<Value>> {
    debug!("Getting item {}", uid);
    let conn = sqlite.get()?;

    let mut stmt = conn.prepare_cached("SELECT * FROM items WHERE uid = :uid")?;
    let rows = stmt.query_named(&[(":uid", &uid)])?;

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

/// Create an item, failing if the `uid` existed before.
/// The new item will be created with `version = 1`.
pub fn create_item(sqlite: &Pool<SqliteConnectionManager>, json: Value) -> Result<Value> {
    debug!("Creating item {}", json);
    let mut fields_map = match json {
        Object(map) => map,
        _ => {
            return Err(Error {
                code: StatusCode::BAD_REQUEST,
                msg: "Expected JSON object".to_string(),
            })
        }
    };

    let millis_now = Utc::now().timestamp_millis();
    fields_map.remove("version");
    fields_map.insert("dateCreated".to_string(), millis_now.into());
    fields_map.insert("dateModified".to_string(), millis_now.into());
    fields_map.insert("version".to_string(), Value::from(1));

    let mut sql_body = "INSERT INTO items (".to_string();
    let mut sql_body_params = "".to_string();
    let mut first_parameter = true;
    for (field, value) in &fields_map {
        validate_field_name(field)?;
        match value {
            Value::Array(_) => continue,
            Value::Object(_) => continue,
            _ => (),
        };
        if !first_parameter {
            sql_body.push_str(", ");
            sql_body_params.push_str(", :")
        };
        first_parameter = false;
        sql_body.push_str(field);
        sql_body_params.push_str(field);
    }
    sql_body.push_str(") VALUES (:");
    sql_body.push_str(sql_body_params.as_str());
    sql_body.push_str(");");

    let sql_params = fields_mapping_to_owned_sql_params(&fields_map);
    let sql_params = borrow_sql_params(&sql_params);

    let conn = sqlite.get()?;
    let mut stmt = conn.prepare_cached(&sql_body)?;
    stmt.execute_named(sql_params.as_slice())?;
    let json = serde_json::json!({"id": conn.last_insert_rowid()});
    Ok(json)
}

/// Update an item with a JSON object.
/// Json `null` properties will be erased from the database.
/// Nonexisting properties will cause error.
/// Reserved properties like "version" will be silently ignored.
/// All existing edges from the item will be removed,
/// and new edges will be created to all Objects and Arrays within the JSON, by their `uid`.
/// The version of the item in the database will be increased `version += 1`.
pub fn update_item(sqlite: &Pool<SqliteConnectionManager>, uid: i64, json: Value) -> Result<()> {
    debug!("Updating item {} with {}", uid, json);
    let mut fields_map = match json {
        Object(map) => map,
        _ => {
            return Err(Error {
                code: StatusCode::BAD_REQUEST,
                msg: "Expected JSON object".to_string(),
            })
        }
    };

    fields_map.remove("uid");
    fields_map.remove("_type");
    fields_map.remove("dateCreated");
    fields_map.remove("dateModified");
    fields_map.remove("deleted");
    fields_map.remove("version");

    fields_map.insert(
        "dateModified".to_string(),
        Utc::now().timestamp_millis().into(),
    );

    let mut sql_body = "UPDATE items SET ".to_string();
    let mut first_parameter = true;
    for (field, value) in &fields_map {
        validate_field_name(field)?;
        match value {
            Value::Array(_) => continue,
            Value::Object(_) => continue,
            _ => (),
        };
        if !first_parameter {
            sql_body.push_str(", ");
        };
        first_parameter = false;
        sql_body.push_str(field);
        sql_body.push_str(" = :");
        sql_body.push_str(field);
    }
    sql_body.push_str(", version = version + 1");
    sql_body.push_str(" WHERE uid = :uid ;");

    let mut sql_params = fields_mapping_to_owned_sql_params(&fields_map);
    let uid = Value::from(uid);
    sql_params.push((":uid".into(), json_value_to_sqlite_parameter(&uid)));
    let sql_params = borrow_sql_params(&sql_params);

    let conn = sqlite.get()?;
    let mut stmt = conn.prepare_cached(&sql_body)?;
    let updated_count = stmt.execute_named(sql_params.as_slice())?;
    if updated_count > 0 {
        Ok(())
    } else {
        Err(Error {
            code: StatusCode::NOT_FOUND,
            msg: "Update failed, uid not found".to_string(),
        })
    }
}

/// Delete an already existing item.
/// Return successfully if item existed and was successfully deleted.
/// Return NOT_FOUND if item does not exist.
pub fn delete_item(sqlite: &Pool<SqliteConnectionManager>, uid: i64) -> Result<()> {
    debug!("Deleting item {}", uid);
    let json = serde_json::json!({
        "deleted": true,
        "dateModified": Utc::now().timestamp_millis(),
    });
    update_item(sqlite, uid, json)
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
    for (field, value) in &fields_map {
        validate_field_name(field)?;
        match value {
            Value::Array(_) => continue,
            Value::Object(_) => continue,
            _ => (),
        };
        if !first_parameter {
            sql_body.push_str(" AND ")
        };
        first_parameter = false;
        sql_body.push_str(field);
        sql_body.push_str(" = :");
        sql_body.push_str(field);
    }
    sql_body.push_str(";");

    let sql_params = fields_mapping_to_owned_sql_params(&fields_map);
    let sql_params = borrow_sql_params(&sql_params);
    let conn = sqlite.get()?;
    let mut stmt = conn.prepare_cached(&sql_body)?;
    let rows = stmt.query_named(sql_params.as_slice())?;
    let json = sqlite_rows_to_json(rows)?;
    Ok(json)
}

/// Get an item by its `uid`, with edges and linked items.
pub fn get_item_with_edges(sqlite: &Pool<SqliteConnectionManager>, uid: i64) -> Result<Vec<Value>> {
    debug!("Getting item {}", uid);
    let conn = sqlite.get()?;

    let mut stmt_item = conn.prepare_cached("SELECT * FROM items WHERE uid = :uid")?;
    let item_rows = stmt_item.query_named(&[(":uid", &uid)])?;
    let mut item_json: Map<String, Value> = sqlite_rows_to_map(item_rows);

    let mut stmt_edge =
        conn.prepare_cached("SELECT _type, _target FROM edges WHERE _source = :_source")?;
    let edge_rows = stmt_edge.query_named(&[(":_source", &uid)])?;
    let edges = sqlite_rows_to_json(edge_rows)?;
    let mut targets = Vec::new();
    for edge in edges {
        let target = edge
            .as_object()
            .expect("Failed to get object")
            .get("_target")
            .expect("Failed to get target")
            .as_i64()
            .expect("Failed to get i64");
        let mut stmt = conn.prepare_cached("SELECT * FROM items WHERE uid = :uid")?;
        let rows = stmt.query_named(&[(":uid", &target)])?;
        targets.push(sqlite_rows_to_map(rows));
    }
    item_json.insert("edges".to_string(), Value::from(targets));

    Ok(map_to_json(item_json)?)
}
