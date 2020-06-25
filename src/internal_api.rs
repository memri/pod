use crate::error::Error;
use crate::error::Result;
use log::debug;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::types::ToSqlOutput;
use rusqlite::types::ValueRef;
use rusqlite::Rows;
use rusqlite::ToSql;
use rusqlite::NO_PARAMS;
use serde_json::value::Value::Object;
use serde_json::Map;
use serde_json::Value;
use std::str;
use warp::http::status::StatusCode;

/// Get project version as seen by Cargo.
pub fn get_project_version() -> &'static str {
    debug!("Returning API version...");
    env!("CARGO_PKG_VERSION")
}

fn sqlite_value_to_json(value: ValueRef) -> Value {
    match value {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(i) => Value::from(i),
        ValueRef::Real(f) => Value::from(f),
        ValueRef::Text(t) => {
            Value::from(str::from_utf8(t).expect("Non UTF-8 data in TEXT field of the database"))
        }
        ValueRef::Blob(_) => panic!("BLOB conversion to JSON not supported"),
    }
}

/// Convert an SQLite result set into array of JSON objects
fn sqlite_rows_to_json(mut rows: Rows) -> rusqlite::Result<Vec<Value>> {
    let mut result = Vec::new();
    while let Some(row) = rows.next()? {
        let mut json_object = Map::new();
        for i in 0..row.column_count() {
            let name = row.column_name(i)?.to_string();
            json_object.insert(name, sqlite_value_to_json(row.get_raw(i)));
        }
        result.push(Value::from(json_object));
    }
    Ok(result)
}

/// Get an item from the SQLite database.
/// None if the `id` doesn't exist in DB, Some(json) if it does.
/// `syncState` is added to the returned json,
/// based on the version in DB and if properties are all included.
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

/// Create an item presuming it didn't exist before.
/// Return Some(uid) if the data item's `memriID` cannot be found in the DB,
/// and was successfully created in DB.
/// Return None if the data item had `uid` field, indicating it already exists,
/// should use `update` for changing the item.
/// Dgraph returns `uid` in hexadecimal and should be converted to decimal.
/// `syncState` field of the json from client is processed and removed,
/// before the json is inserted into dgraph.
/// The new item will be created with `version = 1`.
pub fn create_item(_sqlite: &Pool<SqliteConnectionManager>, json: Value) -> Option<u64> {
    debug!("Creating item {}", json);
    unimplemented!()
}

/// First verify if `mid` exists, if so, then update the already existing item.
/// Parameters:
///     - `mid`: memriID of the item to be updated.
///     - `json`: the json sent by client.
/// Return `false` if dgraph didn't have a node with this `mid`.
/// Return `true` if dgraph had a node with this `mid` and it was successfully updated.
/// The `version` that is sent to us by the client should be completely ignored.
/// Use `expand(_all_)` to get all properties of an item, only works if data has a `dgraph.type`.
/// `syncState` from client json is processed and removed.
/// A successful update operation should also increase the version in dgraph as `version += 1`.
pub fn update_item(_sqlite: &Pool<SqliteConnectionManager>, memri_id: String, json: Value) -> bool {
    debug!("Updating item {} with {}", memri_id, json);
    unimplemented!()
}

/// Delete an already existing item.
/// Return `false` if dgraph didn't have a node with this memriID.
/// Return `true` if dgraph had a node with this memriID and it was successfully deleted.
pub fn delete_item(_sqlite: &Pool<SqliteConnectionManager>, memri_id: String) -> bool {
    debug!("Deleting item {}", memri_id);
    unimplemented!()
}

fn json_value_to_sqlite_parameter(json: &Value) -> ToSqlOutput<'_> {
    match json {
        Value::Null => ToSqlOutput::Borrowed(ValueRef::Null),
        Value::String(s) => ToSqlOutput::Borrowed(ValueRef::Text(s.as_bytes())),
        Value::Number(n) => {
            if let Some(int) = n.as_i64() {
                ToSqlOutput::Borrowed(ValueRef::Integer(int))
            } else if let Some(float) = n.as_f64() {
                ToSqlOutput::Borrowed(ValueRef::Real(float))
            } else {
                panic!("Unsupported number precision (non-f64) of a JSON value.")
            }
        }
        Value::Array(_) => panic!("Cannot convert JSON array to an SQL parameter"),
        Value::Bool(_) => panic!("Cannot convert boolean to SQLite parameter. Use 0 or 1..."),
        Value::Object(_) => panic!("Cannot convert JSON object to an SQL parameter"),
    }
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
        let field: &str = field;
        sql_params.push((field, json_value_to_sqlite_parameter(value)));
    }
    let sql_params: Vec<_> = sql_params
        .iter()
        .map(|(field, value)| (*field, value as &dyn ToSql))
        .collect();
    let conn = sqlite.get()?;
    let mut stmt = conn.prepare_cached(&sql_body)?;
    let rows = stmt.query_named(sql_params.as_slice())?;
    let json = sqlite_rows_to_json(rows)?;
    Ok(json)
}
