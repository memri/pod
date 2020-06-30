use crate::error::Error;
use crate::error::Result;
use crate::sql_converters::borrow_sql_params;
use crate::sql_converters::fields_mapping_to_owned_sql_params;
use crate::sql_converters::json_value_to_sqlite_parameter;
use crate::sql_converters::sqlite_rows_to_json;
use chrono::Utc;
use lazy_static::lazy_static;
use log::debug;
use r2d2::Pool;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use regex::Regex;
use rusqlite::NO_PARAMS;
use serde_json::value::Value::Object;
use serde_json::Value;
use std::collections::HashSet;
use std::str;
use warp::http::status::StatusCode;

/// Validate field name.
/// Field name is valid only if it contains less than 14 characters and
/// characters from 'a' to 'z', 'A' to 'Z'.
fn field_name_validator(field: &str) -> HashSet<&str> {
    lazy_static! {
        static ref FIELD_RE: Regex = Regex::new(r"$[a-zA-Z]{1,14}^").unwrap();
    }
    FIELD_RE.find_iter(field).map(|m| m.as_str()).collect()
}

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
    fields_map.remove("dateAccessed");
    fields_map.insert("version".to_string(), Value::from(1));

    let mut sql_body = "INSERT INTO items (".to_string();
    let mut sql_body_params = "".to_string();
    let mut first_parameter = true;
    for (field, value) in &fields_map {
        match value {
            Value::Array(_) => continue,
            Value::Bool(_) => continue,
            Value::Object(_) => continue,
            _ => (),
        };
        if !first_parameter {
            sql_body.push_str(", ");
            sql_body_params.push_str(", :")
        };
        first_parameter = false;
        println!("{:#?}", field_name_validator(field));
        sql_body.push_str(field); // TODO: prevent SQL injection! See GitLab issue #84
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
/// Json `null` fields will be erased from the database.
/// Nonexisting or reserved properties like "version" will cause error (TODO).
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

    fields_map.insert(
        "dateModified".to_string(),
        Utc::now().timestamp_millis().into(),
    );

    let mut sql_body = "UPDATE items SET ".to_string();
    let mut first_parameter = true;
    for (field, value) in &fields_map {
        match value {
            Value::Array(_) => continue,
            Value::Bool(_) => continue,
            Value::Object(_) => continue,
            _ => (),
        };
        if !first_parameter {
            sql_body.push_str(", ");
        };
        first_parameter = false;
        sql_body.push_str(field); // TODO: prevent SQL injection! See GitLab issue #84
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
/// * `query` - json with the desired fields, e.g. { "author": "Vasili", "type": "note" }
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
        match value {
            Value::Array(_) => continue,
            Value::Bool(_) => continue,
            Value::Object(_) => continue,
            _ => (),
        };
        if !first_parameter {
            sql_body.push_str(" AND ")
        };
        first_parameter = false;
        sql_body.push_str(field); // TODO: prevent SQL injection! See GitLab issue #84
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
