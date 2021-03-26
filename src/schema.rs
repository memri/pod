use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use warp::http::StatusCode;
use lazy_static::lazy_static;
use std::collections::HashSet;
use regex::Regex;

/// See `README.md#understanding-the-schema` to understand possible
/// property types and their meaning
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum SchemaPropertyType {
    Text,
    Integer,
    Real,
    Bool,
    DateTime,
}

impl SchemaPropertyType {
    pub fn from_string(str: &str) -> std::result::Result<SchemaPropertyType, String> {
        let str = str.to_lowercase();
        match str.as_ref() {
            "text" => Ok(SchemaPropertyType::Text),
            "integer" => Ok(SchemaPropertyType::Integer),
            "real" => Ok(SchemaPropertyType::Real),
            "bool" => Ok(SchemaPropertyType::Bool),
            "datetime" => Ok(SchemaPropertyType::DateTime),
            _ => Err(format!(
                "Failed to parse {} into {}",
                str,
                std::any::type_name::<SchemaPropertyType>()
            )),
        }
    }
}

pub struct Schema {
    pub property_types: HashMap<String, SchemaPropertyType>,
}

pub fn validate_property_name(property: &str) -> crate::error::Result<()> {
    lazy_static! {
        static ref REGEXP: Regex =
            Regex::new(r"^[_a-zA-Z][_a-zA-Z0-9]{1,30}$").expect("Cannot create regex");
    }
    if !REGEXP.is_match(property) {
        Err(crate::error::Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!(
                "Property name {} does not satisfy the format {}",
                property,
                REGEXP.as_str()
            ),
        })
    } else if BLOCKLIST_COLUMN_NAMES.contains(&property.to_lowercase()) {
        Err(crate::error::Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!("Blocklisted item property {}", property),
        })
    } else {
        Ok(())
    }
}

// SQLite keywords taken from https://www.sqlite.org/lang_keywords.html
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
            .map(|w| w.to_lowercase())
            .collect()
    };
}
