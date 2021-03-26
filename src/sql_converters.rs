use crate::database_migrate_schema;
use crate::error::Error;
use database_migrate_schema::ALL_COLUMN_TYPES;
use database_migrate_schema::BOOL_COLUMNS;
use database_migrate_schema::DATE_TIME_COLUMNS;
use database_migrate_schema::INTEGER_COLUMNS;
use database_migrate_schema::REAL_COLUMNS;
use database_migrate_schema::TEXT_COLUMNS;
use log::warn;
use rusqlite::types::ToSqlOutput;
use rusqlite::types::ValueRef;
use rusqlite::Row;
use serde_json::Map;
use serde_json::Value;
use warp::http::status::StatusCode;

pub fn sqlite_row_to_map(row: &Row, partial: bool) -> rusqlite::Result<Map<String, Value>> {
    let mut row_map = Map::new();
    for i in 0..row.column_count() {
        let name = row.column_name(i)?.to_string();
        if let Some(value) = sqlite_value_to_json(row.get_raw(i), &name) {
            row_map.insert(name, value);
        }
    }
    if partial {
        row_map.insert("_partial".to_string(), Value::from(true));
    }
    Ok(row_map)
}

pub fn sqlite_value_to_json(value: ValueRef, column_name: &str) -> Option<Value> {
    match value {
        ValueRef::Null => None,
        ValueRef::Integer(i) => {
            if !database_migrate_schema::BOOL_COLUMNS.contains(column_name) {
                Some(Value::from(i))
            } else if i == 0 {
                Some(Value::from(false))
            } else if i == 1 {
                Some(Value::from(true))
            } else {
                warn!("Column {} should be a boolean, got {} in the database instead. Did column definitions change?", column_name, i);
                None
            }
        }
        ValueRef::Real(f) => Some(Value::from(f)),
        ValueRef::Text(t) => match std::str::from_utf8(t) {
            Ok(t) => Some(Value::from(t)),
            Err(err) => {
                warn!("Non-UTF-8 TEXT in the database, {}", err);
                None
            }
        },
        ValueRef::Blob(_) => panic!("BLOB value found in the database, this should never happen"),
    }
}

// pub fn json_value_and_schema_to_sqlite<'a>(
//     _json: &'a Value,
//     _property_name: &str,
//     _schema: &Schema,
// ) -> crate::error::Result<ToSqlOutput<'a>> {
//     unimplemented!()
// }

pub fn json_value_to_sqlite<'a>(
    json: &'a Value,
    column: &str,
) -> crate::error::Result<ToSqlOutput<'a>> {
    match json {
        Value::Null => Ok(ToSqlOutput::Borrowed(ValueRef::Null)),
        Value::String(s) if TEXT_COLUMNS.contains(column) => {
            Ok(ToSqlOutput::Borrowed(ValueRef::Text(s.as_bytes())))
        }
        Value::Number(n) if INTEGER_COLUMNS.contains(column) => {
            if let Some(int) = n.as_i64() {
                Ok(ToSqlOutput::Borrowed(ValueRef::Integer(int)))
            } else {
                Err(Error {
                    code: StatusCode::BAD_REQUEST,
                    msg: format!("Failed to parse JSON number {} to i64 ({})", n, column),
                })
            }
        }
        Value::Number(n) if REAL_COLUMNS.contains(column) => {
            if let Some(int) = n.as_f64() {
                Ok(ToSqlOutput::Borrowed(ValueRef::Real(int)))
            } else {
                Err(Error {
                    code: StatusCode::BAD_REQUEST,
                    msg: format!("Failed to parse JSON number {} to f64 ({})", n, column),
                })
            }
        }
        Value::Bool(b) if BOOL_COLUMNS.contains(column) => Ok((if *b { 1 } else { 0 }).into()),
        Value::Number(n) if DATE_TIME_COLUMNS.contains(column) => {
            if let Some(int) = n.as_i64() {
                Ok(ToSqlOutput::Borrowed(ValueRef::Integer(int)))
            } else if let Some(float) = n.as_f64() {
                warn!("Using float-to-integer conversion property {}, value {}. This might not be supported in the future, please use a compatible DateTime format https://gitlab.memri.io/memri/pod#understanding-the-schema", float, column);
                Ok((float.round() as i64).into())
            } else {
                Err(Error {
                    code: StatusCode::BAD_REQUEST,
                    msg: "Unsupported number precision (non-f64) of a JSON value.".to_string(),
                })
            }
        }
        _ => {
            if let Some(dbtype) = ALL_COLUMN_TYPES.get(column) {
                Err(Error {
                    code: StatusCode::BAD_REQUEST,
                    msg: format!(
                        "Failed to parse json value {} to {:?} ({})",
                        json, dbtype, column
                    ),
                })
            } else {
                Err(Error {
                    code: StatusCode::BAD_REQUEST,
                    msg: format!(
                        "Property {} not defined in schema (attempted to use it for json value {})",
                        column, json,
                    ),
                })
            }
        }
    }
}
