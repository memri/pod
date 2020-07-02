use crate::api_model::BulkActions;
use crate::error::Error;
use crate::error::Result;
use crate::sql_converters::borrow_sql_params;
use crate::sql_converters::fields_mapping_to_owned_sql_params;
use crate::sql_converters::json_value_to_sqlite;
use crate::sql_converters::sqlite_rows_to_json;
use crate::sql_converters::validate_field_name;
use chrono::Utc;
use log::debug;
use r2d2::Pool;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::ToSql;
use rusqlite::Transaction;
use rusqlite::NO_PARAMS;
use serde_json::value::Value::Object;
use serde_json::Value;
use std::collections::HashMap;
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

    let sql_params = fields_mapping_to_owned_sql_params(&fields_map)?;
    let sql_params = borrow_sql_params(&sql_params);

    let conn = sqlite.get()?;
    let mut stmt = conn.prepare_cached(&sql_body)?;
    stmt.execute_named(sql_params.as_slice())?;
    let json = serde_json::json!({"uid": conn.last_insert_rowid()});
    Ok(json)
}

fn is_array_or_object(value: &Value) -> bool {
    match value {
        Value::Array(_) => true,
        Value::Object(_) => true,
        _ => false,
    }
}

fn write_sql_body(sql: &mut String, keys: &[&String], separator: &str) {
    let mut first = true;
    for key in keys {
        if !first {
            sql.push_str(separator);
        }
        sql.push_str(key);
        first = false;
    }
}

fn execute_sql(tx: &Transaction, sql: &str, fields: &HashMap<String, Value>) -> Result<()> {
    let mut sql_params = Vec::new();
    for (key, value) in fields {
        sql_params.push((format!(":{}", key), json_value_to_sqlite(value)?));
    }
    let sql_params: Vec<_> = sql_params
        .iter()
        .map(|(field, value)| (field.as_str(), value as &dyn ToSql))
        .collect();
    let mut stmt = tx.prepare_cached(&sql)?;
    stmt.execute_named(&sql_params)?;
    Ok(())
}

/// Create an item presuming consistency checks were already done
fn create_item_tx(tx: &Transaction, fields: HashMap<String, Value>) -> Result<()> {
    let mut fields: HashMap<String, Value> = fields
        .into_iter()
        .filter(|(k, v)| !is_array_or_object(v) && validate_field_name(k).is_ok())
        .collect();
    let millis_now = Utc::now().timestamp_millis();
    fields.insert("dateCreated".to_string(), millis_now.into());
    fields.insert("dateModified".to_string(), millis_now.into());
    fields.insert("version".to_string(), Value::from(1));

    let mut sql = "INSERT INTO items (".to_string();
    let keys: Vec<_> = fields.keys().collect();
    write_sql_body(&mut sql, &keys, ", ");
    sql.push_str(") VALUES (:");
    write_sql_body(&mut sql, &keys, ", :");
    sql.push_str(");");
    execute_sql(tx, &sql, &fields)
}

/// Update an item presuming all dangerous fields were already removed, and "uid" is present
fn update_item_tx(tx: &Transaction, fields: HashMap<String, Value>) -> Result<()> {
    let mut fields: HashMap<String, Value> = fields
        .into_iter()
        .filter(|(k, v)| !is_array_or_object(v) && validate_field_name(k).is_ok())
        .collect();
    let millis_now = Utc::now().timestamp_millis();
    fields.remove("_type");
    fields.remove("dateCreated");
    fields.insert("dateModified".to_string(), millis_now.into());
    fields.remove("deleted");
    fields.remove("version");
    let mut sql = "UPDATE items SET ".to_string();
    let mut after_first = false;
    for key in fields.keys() {
        if after_first {
            sql.push_str(", ");
        }
        after_first = true;
        sql.push_str(key);
        sql.push_str(" = :");
        sql.push_str(key);
    }
    sql.push_str(", version = version + 1 ");
    sql.push_str("WHERE uid = :uid ;");
    execute_sql(tx, &sql, &fields)
}

/// Create an edge presuming consistency checks were already done
fn create_edge(tx: &Transaction, fields: HashMap<String, Value>) -> Result<()> {
    let fields: HashMap<String, Value> = fields
        .into_iter()
        .filter(|(k, v)| !is_array_or_object(v) && validate_field_name(k).is_ok())
        .collect();
    let mut sql = "INSERT INTO edges (".to_string();
    let keys: Vec<_> = fields.keys().collect();
    write_sql_body(&mut sql, &keys, ", ");
    sql.push_str(") VALUES (:");
    write_sql_body(&mut sql, &keys, ", :");
    sql.push_str(");");
    execute_sql(tx, &sql, &fields)
}

fn bulk_action_tx(tx: &Transaction, json_actions: BulkActions) -> Result<()> {
    for mut item in json_actions.create_items {
        item.fields.insert("uid".to_string(), item.uid.into());
        create_item_tx(tx, item.fields)?;
    }
    for mut item in json_actions.update_items {
        item.fields.insert("uid".to_string(), item.uid.into());
        update_item_tx(tx, item.fields)?;
    }
    for mut edge in json_actions.create_edges {
        edge.fields
            .insert("_source".to_string(), edge._source.into());
        edge.fields
            .insert("_target".to_string(), edge._target.into());
        edge.fields.insert("_type".to_string(), edge._type.into());
        create_edge(tx, edge.fields)?;
    }
    Ok(())
}

/// Perform a bulk of operations simultaneously.
/// All changes are guaranteed to happen at the same time - or not at all.
/// See `api_model::BulkActions` for input JSON information.
pub fn bulk_action(sqlite: &Pool<SqliteConnectionManager>, json: Value) -> Result<()> {
    debug!("Performing bulk action {}", json);
    let json: BulkActions = serde_json::from_value(json)?;
    let mut conn = sqlite.get()?;
    let tx = conn.transaction()?;
    bulk_action_tx(&tx, json)?;
    tx.commit()?;
    Ok(())
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

    let mut sql_params = fields_mapping_to_owned_sql_params(&fields_map)?;
    let uid = Value::from(uid);
    sql_params.push((":uid".into(), json_value_to_sqlite(&uid)?));
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

    let sql_params = fields_mapping_to_owned_sql_params(&fields_map)?;
    let sql_params = borrow_sql_params(&sql_params);
    let conn = sqlite.get()?;
    let mut stmt = conn.prepare_cached(&sql_body)?;
    let rows = stmt.query_named(sql_params.as_slice())?;
    let json = sqlite_rows_to_json(rows)?;
    Ok(json)
}
