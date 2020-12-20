use crate::database_migrate_schema;
use crate::error::Error;
use database_migrate_schema::ALL_COLUMN_TYPES;
use database_migrate_schema::BOOL_COLUMNS;
use database_migrate_schema::DATE_TIME_COLUMNS;
use database_migrate_schema::INTEGER_COLUMNS;
use database_migrate_schema::REAL_COLUMNS;
use database_migrate_schema::TEXT_COLUMNS;
use lazy_static::lazy_static;
use log::warn;
use regex::Regex;
use rusqlite::types::ToSqlOutput;
use rusqlite::types::ValueRef;
use rusqlite::Row;
use rusqlite::Rows;
use rusqlite::ToSql;
use serde_json::Map;
use serde_json::Value;
use std::collections::HashSet;
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

pub fn sqlite_rows_to_json(mut rows: Rows, partial: bool) -> rusqlite::Result<Vec<Value>> {
    let mut result = Vec::new();
    while let Some(row) = rows.next()? {
        let json_object = sqlite_row_to_map(row, partial)?;
        result.push(Value::from(json_object));
    }
    Ok(result)
}

fn sqlite_value_to_json(value: ValueRef, column_name: &str) -> Option<Value> {
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
                panic!("Column {} should be a boolean, got {} in the database instead. Did column definitions change?", column_name, i)
            }
        }
        ValueRef::Real(f) => Some(Value::from(f)),
        ValueRef::Text(t) => Some(Value::from(
            std::str::from_utf8(t).expect("Non UTF-8 data in TEXT field of the database"),
        )),
        ValueRef::Blob(_) => panic!("BLOB conversion to JSON not supported"),
    }
}

pub fn borrow_sql_params<'a>(
    sql_params: &'a [(String, ToSqlOutput)],
) -> Vec<(&'a str, &'a dyn ToSql)> {
    sql_params
        .iter()
        .map(|(field, value)| (field.as_str(), value as &dyn ToSql))
        .collect()
}

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
        Value::Number(n) if BOOL_COLUMNS.contains(column) => {
            if let Some(int) = n.as_f64() {
                Ok(ToSqlOutput::Borrowed(ValueRef::Real(int)))
            } else {
                Err(Error {
                    code: StatusCode::BAD_REQUEST,
                    msg: format!("Failed to parse JSON number {} to BOOL ({})", n, column),
                })
            }
        }
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

pub fn validate_property_name(property: &str) -> crate::error::Result<()> {
    lazy_static! {
        static ref REGEXP: Regex =
            Regex::new(r"^[_a-zA-Z][_a-zA-Z0-9]{1,30}$").expect("Cannot create regex");
    }
    if BLOCKLIST_COLUMN_NAMES.contains(property) {
        Err(crate::error::Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!("Blocklisted item property {}", property),
        })
    } else if !REGEXP.is_match(property) {
        Err(crate::error::Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!(
                "Item property name {} does not satisfy the format {}",
                property,
                REGEXP.as_str()
            ),
        })
    } else {
        Ok(())
    }
}

// Taken from the official documentation https://www.sqlite.org/lang_keywords.html
const BLOCKLIST_COLUMN_NAMES_ARRAY: &[&str] = &[
    "ABORT",
    "ACTION",
    "ADD",
    "AFTER",
    "ALL",
    "ALTER",
    "ALWAYS",
    "ANALYZE",
    "AND",
    "AS",
    "ASC",
    "ATTACH",
    "AUTOINCREMENT",
    "BEFORE",
    "BEGIN",
    "BETWEEN",
    "BY",
    "CASCADE",
    "CASE",
    "CAST",
    "CHECK",
    "COLLATE",
    "COLUMN",
    "COMMIT",
    "CONFLICT",
    "CONSTRAINT",
    "CREATE",
    "CROSS",
    "CURRENT",
    "CURRENT_DATE",
    "CURRENT_TIME",
    "CURRENT_TIMESTAMP",
    "DATABASE",
    "DEFAULT",
    "DEFERRABLE",
    "DEFERRED",
    "DELETE",
    "DESC",
    "DETACH",
    "DISTINCT",
    "DO",
    "DROP",
    "EACH",
    "ELSE",
    "END",
    "ESCAPE",
    "EXCEPT",
    "EXCLUDE",
    "EXCLUSIVE",
    "EXISTS",
    "EXPLAIN",
    "FAIL",
    "FILTER",
    "FIRST",
    "FOLLOWING",
    "FOR",
    "FOREIGN",
    "FROM",
    "FULL",
    "GENERATED",
    "GLOB",
    "GROUP",
    "GROUPS",
    "HAVING",
    "IF",
    "IGNORE",
    "IMMEDIATE",
    "IN",
    "INDEX",
    "INDEXED",
    "INITIALLY",
    "INNER",
    "INSERT",
    "INSTEAD",
    "INTERSECT",
    "INTO",
    "IS",
    "ISNULL",
    "JOIN",
    "KEY",
    "LAST",
    "LEFT",
    "LIKE",
    "LIMIT",
    "MATCH",
    "NATURAL",
    "NO",
    "NOT",
    "NOTHING",
    "NOTNULL",
    "NULL",
    "NULLS",
    "OF",
    "OFFSET",
    "ON",
    "OR",
    "ORDER",
    "OTHERS",
    "OUTER",
    "OVER",
    "PARTITION",
    "PLAN",
    "PRAGMA",
    "PRECEDING",
    "PRIMARY",
    "QUERY",
    "RAISE",
    "RANGE",
    "RECURSIVE",
    "REFERENCES",
    "REGEXP",
    "REINDEX",
    "RELEASE",
    "RENAME",
    "REPLACE",
    "RESTRICT",
    "RIGHT",
    "ROLLBACK",
    "ROW",
    "ROWS",
    "SAVEPOINT",
    "SELECT",
    "SET",
    "TABLE",
    "TEMP",
    "TEMPORARY",
    "THEN",
    "TIES",
    "TO",
    "TRANSACTION",
    "TRIGGER",
    "UNBOUNDED",
    "UNION",
    "UNIQUE",
    "UPDATE",
    "USING",
    "VACUUM",
    "VALUES",
    "VIEW",
    "VIRTUAL",
    "WHEN",
    "WHERE",
    "WINDOW",
    "WITH",
    "WITHOUT",
];

lazy_static! {
    static ref BLOCKLIST_COLUMN_NAMES: HashSet<String> = {
        BLOCKLIST_COLUMN_NAMES_ARRAY
            .iter()
            .map(|w| w.to_string())
            .collect()
    };
}
