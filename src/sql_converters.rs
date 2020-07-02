use crate::database_init;
use lazy_static::lazy_static;
use regex::Regex;
use rusqlite::types::ToSqlOutput;
use rusqlite::types::ValueRef;
use rusqlite::Rows;
use rusqlite::ToSql;
use serde_json::Map;
use serde_json::Value;
use std::collections::HashSet;
use warp::http::status::StatusCode;

/// Convert an SQLite result set into array of JSON objects
pub fn sqlite_rows_to_json(mut rows: Rows) -> rusqlite::Result<Vec<Value>> {
    let mut result = Vec::new();
    while let Some(row) = rows.next()? {
        let mut json_object = Map::new();
        for i in 0..row.column_count() {
            let name = row.column_name(i)?.to_string();
            let value = sqlite_value_to_json(row.get_raw(i), &name);
            json_object.insert(name, value);
        }
        result.push(Value::from(json_object));
    }
    Ok(result)
}

pub fn sqlite_value_to_json(value: ValueRef, column_name: &str) -> Value {
    match value {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(i) => {
            if !database_init::BOOL_COLUMNS.contains(column_name) {
                Value::from(i)
            } else if i == 0 {
                Value::from(false)
            } else if i == 1 {
                Value::from(true)
            } else {
                panic!("Column {} should be a boolean, got {} in the database instead. Did column definitions change?", column_name, i)
            }
        }
        ValueRef::Real(f) => Value::from(f),
        ValueRef::Text(t) => Value::from(
            std::str::from_utf8(t).expect("Non UTF-8 data in TEXT field of the database"),
        ),
        ValueRef::Blob(_) => panic!("BLOB conversion to JSON not supported"),
    }
}

pub fn fields_mapping_to_owned_sql_params(
    fields_map: &Map<String, serde_json::Value>,
) -> Vec<(String, ToSqlOutput)> {
    let mut sql_params = Vec::new();
    for (field, value) in fields_map {
        match value {
            Value::Array(_) => continue,
            Value::Object(_) => continue,
            _ => (),
        };
        let field = format!(":{}", field);
        sql_params.push((field, json_value_to_sqlite_parameter(value)));
    }
    sql_params
}

pub fn borrow_sql_params<'a>(
    sql_params: &'a [(String, ToSqlOutput)],
) -> Vec<(&'a str, &'a dyn ToSql)> {
    sql_params
        .iter()
        .map(|(field, value)| (field.as_str(), value as &dyn ToSql))
        .collect()
}

pub fn json_value_to_sqlite_parameter(json: &Value) -> ToSqlOutput<'_> {
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
        Value::Bool(b) => ToSqlOutput::Borrowed(ValueRef::Integer(if *b { 1 } else { 0 })),
        Value::Array(_) => panic!("Cannot convert JSON array to an SQL parameter"),
        Value::Object(_) => panic!("Cannot convert JSON object to an SQL parameter"),
    }
}

/// Field name is valid only if it contains less than or equal to 18 characters and
/// characters from 'a' to 'z', 'A' to 'Z'.
pub fn validate_field_name(field: &str) -> crate::error::Result<()> {
    lazy_static! {
        static ref REGEXP: Regex = Regex::new(r"^[a-zA-Z]{1,18}$").expect("Cannot create regex");
    }
    if REGEXP.is_match(field) && !BLACKLIST_COLUMN_NAMES.contains(field) {
        Ok(())
    } else {
        Err(crate::error::Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!("Invalid field (database column) name {}", field),
        })
    }
}

// Taken from the official documentation https://www.sqlite.org/lang_keywords.html
const BLACKLIST_COLUMN_NAMES_ARRAY: &[&str] = &[
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
    pub static ref BLACKLIST_COLUMN_NAMES: HashSet<String> = {
        BLACKLIST_COLUMN_NAMES_ARRAY
            .iter()
            .map(|w| w.to_string())
            .collect()
    };
}
