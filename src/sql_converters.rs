use rusqlite::types::ToSqlOutput;
use rusqlite::types::ValueRef;
use rusqlite::Rows;
use rusqlite::ToSql;
use serde_json::Map;
use serde_json::Value;

/// Convert an SQLite result set into array of JSON objects
pub fn sqlite_rows_to_json(mut rows: Rows) -> rusqlite::Result<Vec<Value>> {
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

pub fn sqlite_value_to_json(value: ValueRef) -> Value {
    match value {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(i) => Value::from(i),
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

// pub fn datetime_to_sqlite<'a>(dt: DateTime<Utc>) -> ToSqlOutput<'a> {
//     let milliseconds = dt.timestamp_millis();
//     ToSqlOutput::Owned(rusqlite::types::Value::from(milliseconds))
// }

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
        Value::Bool(b) => {
            if b.eq(&true) {
                ToSqlOutput::Borrowed(ValueRef::Integer(1))
            } else {
                ToSqlOutput::Borrowed(ValueRef::Integer(0))
            }
        }
        Value::Array(_) => panic!("Cannot convert JSON array to an SQL parameter"),
        Value::Object(_) => panic!("Cannot convert JSON object to an SQL parameter"),
    }
}
