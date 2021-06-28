use crate::error::Error;
use crate::error::Result;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use warp::http::StatusCode;

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

impl std::fmt::Display for SchemaPropertyType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl SchemaPropertyType {
    pub fn from_string(str: &str) -> std::result::Result<SchemaPropertyType, String> {
        match str {
            "Text" => Ok(SchemaPropertyType::Text),
            "Integer" => Ok(SchemaPropertyType::Integer),
            "Real" => Ok(SchemaPropertyType::Real),
            "Bool" => Ok(SchemaPropertyType::Bool),
            "DateTime" => Ok(SchemaPropertyType::DateTime),
            _ => Err(format!(
                "Failed to parse {} into {}",
                str,
                std::any::type_name::<SchemaPropertyType>()
            )),
        }
    }
}

#[derive(Debug)]
pub struct Schema {
    pub property_types: HashMap<String, SchemaPropertyType>,
}

/// Validation of _new_ item ids. Note that it is not applied to already existing
/// ids or endpoints that access already existing ids.
pub fn validate_create_item_id(item_id: &str) -> Result<()> {
    lazy_static! {
        static ref REGEXP: Regex =
            Regex::new(r"^[a-zA-Z0-9_-]{6,36}$").expect("Cannot create regex");
    }
    if !REGEXP.is_match(item_id) {
        Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!(
                "Item id '{}' does not satisfy the format {} (use 32 random hex characters if in doubt on item id creation)",
                item_id,
                REGEXP.as_str()
            ),
        })
    } else {
        Ok(())
    }
}

fn validate_property_name_syntax(property: &str) -> Result<()> {
    lazy_static! {
        static ref REGEXP: Regex =
            Regex::new(r"^[a-zA-Z][_a-zA-Z0-9]{0,30}[a-zA-Z0-9]$").expect("Cannot create regex");
    }
    if !REGEXP.is_match(property) {
        Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!(
                "Property name {} does not satisfy the format {}",
                property,
                REGEXP.as_str()
            ),
        })
    } else {
        Ok(())
    }
}

/// All item properties should be of this format
pub fn validate_property_name(property: &str) -> Result<()> {
    if let Err(err) = validate_property_name_syntax(property) {
        Err(err)
    } else if BLOCKLIST_COLUMN_NAMES.contains(&property.to_lowercase()) {
        Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!("Blocklisted item property {}", property),
        })
    } else {
        Ok(())
    }
}

lazy_static! {
    static ref BLOCKLIST_COLUMN_NAMES: HashSet<String> = {
        SQLITE_RESERVED_KEYWORDS
            .iter()
            .chain(POD_ITEM_MANDATORY_PROPERTIES)
            .map(|w| w.to_lowercase())
            .collect()
    };
}

const POD_ITEM_MANDATORY_PROPERTIES: &[&str] = &[
    "rowid",
    "id",
    "type",
    "dateCreated",
    "dateModified",
    "dateServerModified",
    "deleted",
    "source",
    "target",
    "edges",
    "allEdges",
    "forwardEdges",
    "reverseEdges",
    "backwardEdges",
];

/// SQLite keywords taken from https://www.sqlite.org/lang_keywords.html
const SQLITE_RESERVED_KEYWORDS: &[&str] = &[
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
