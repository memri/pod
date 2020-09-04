use crate::sql_converters::validate_property_name;
use lazy_static::lazy_static;
use log::info;
use rusqlite::Connection;
use rusqlite::NO_PARAMS;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct DatabaseSchema {
    types: Vec<DatabaseType>,
}

#[derive(Serialize, Deserialize)]
struct DatabaseType {
    name: String,
    properties: Vec<SchemaProperty>,
}

#[derive(Serialize, Deserialize)]
struct SchemaProperty {
    name: String,
    indexed: bool,
    dbtype: SchemaPropertyType,
}

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

lazy_static! {
    static ref SCHEMA_STRUCT: DatabaseSchema = {
        let schema: DatabaseSchema = serde_json::from_slice(SCHEMA_JSON_BYTES)
            .expect("Failed to parse autogenerated_database_schema to JSON");
        validate_schema(&schema).unwrap_or_else(|err| panic!("Schema validation failed, {}", err));
        schema
    };
}
fn get_columns_of_type(dbtype: SchemaPropertyType) -> HashSet<String> {
    let schema: &DatabaseSchema = &SCHEMA_STRUCT;
    let columns = schema.types.iter().flat_map(|t| &t.properties);
    let mut result = HashSet::new();
    for column in columns {
        if column.dbtype == dbtype {
            result.insert(column.name.to_string());
        }
    }
    result
}
lazy_static! {
    pub static ref TEXT_COLUMNS: HashSet<String> = {
        let mut result = get_columns_of_type(SchemaPropertyType::Text);
        result.insert("_type".to_string());
        result.insert("edgeLabel".to_string());
        result
    };
}
lazy_static! {
    pub static ref INTEGER_COLUMNS: HashSet<String> = {
        let mut result = get_columns_of_type(SchemaPropertyType::Integer);
        result.insert("uid".to_string());
        result.insert("version".to_string());
        result.insert("_source".to_string());
        result.insert("_target".to_string());
        result.insert("sequence".to_string());
        result
    };
}
lazy_static! {
    pub static ref REAL_COLUMNS: HashSet<String> = get_columns_of_type(SchemaPropertyType::Real);
}
lazy_static! {
    pub static ref BOOL_COLUMNS: HashSet<String> = {
        let mut result = get_columns_of_type(SchemaPropertyType::Bool);
        result.insert("deleted".to_string());
        result
    };
}
lazy_static! {
    pub static ref DATE_TIME_COLUMNS: HashSet<String> = {
        let mut result = get_columns_of_type(SchemaPropertyType::DateTime);
        result.insert("dateCreated".to_string());
        result.insert("dateModified".to_string());
        result
    };
}
lazy_static! {
    pub static ref ALL_COLUMN_TYPES: HashMap<String, SchemaPropertyType> = {
        let schema: &DatabaseSchema = &SCHEMA_STRUCT;
        let columns = schema.types.iter().flat_map(|t| &t.properties);
        columns.map(|c| (c.name.to_string(), c.dbtype)).collect()
    };
}

const SCHEMA_JSON_BYTES: &[u8] = include_bytes!("../res/autogenerated_database_schema.json");

pub fn migrate(conn: &Connection) -> Result<(), String> {
    info!("Initializing database schema (additional columns)");
    let schema: &DatabaseSchema = &SCHEMA_STRUCT;
    let (column_indexes, declared_columns) = get_column_info(schema, &conn)?;
    let sql = generate_sql(column_indexes, declared_columns);
    if !sql.is_empty() {
        info!("Updating database schema with:\n{}", sql);
    }
    conn.execute_batch(&sql)
        .map_err(|err| format!("Failed to execute SQL:\n{}\n{}", sql, err))
}

pub fn validate_schema_file(file: &Path) -> Result<(), String> {
    let file = std::fs::read(file)
        .map_err(|err| format!("Failed to read data from target file, {}", err))?;
    let schema: DatabaseSchema = serde_json::from_slice(&file)
        .map_err(|err| format!("Failed to parse schema file, {}", err))?;
    validate_schema(&schema)
}

fn validate_schema(schema: &DatabaseSchema) -> Result<(), String> {
    for typ in &schema.types {
        validate_property_name(&typ.name)
            .map_err(|err| format!("Schema type {} is invalid, {}", typ.name, err))?;
        if typ.name.starts_with("_") {
            return Err(format!("Schema type {} starts with underscore", typ.name));
        }
    }
    let mut properties: HashMap<String, SchemaPropertyType> = HashMap::new();
    for typ in &schema.types {
        for prop in &typ.properties {
            validate_property_name(&prop.name).map_err(|err| {
                format!(
                    "Schema property {} for type {} is invalid, {}",
                    prop.name, typ.name, err
                )
            })?;
            if prop.name.starts_with("_") {
                return Err(format!(
                    "Schema property {} of type {} starts with underscore",
                    prop.name, typ.name
                ));
            }
            let prop_name = prop.name.to_lowercase();
            match properties.get(&prop_name) {
                None => {
                    properties.insert(prop_name, prop.dbtype);
                }
                Some(old_dbtype) => {
                    if old_dbtype != &prop.dbtype {
                        return Err(format!("Schema property {} (lowercase {}) is defined differently for different types", prop.name, prop_name));
                    };
                }
            };
        }
    }
    Ok(())
}

type IndexedAndDeclaredProperties = (HashMap<String, bool>, HashMap<String, SchemaPropertyType>);
fn get_column_info(
    schema: &DatabaseSchema,
    conn: &Connection,
) -> Result<IndexedAndDeclaredProperties, String> {
    validate_schema(&schema)?;
    let mut column_indexes = HashMap::new();
    let mut declared_columns = HashMap::new();

    let all_items_columns: HashSet<String> =
        get_all_columns_pragma("items", conn).map_err(|err| {
            format!(
                "Failed to get items column information using PRAGMA, {}",
                err
            )
        })?;
    let all_items_columns: HashSet<_> =
        all_items_columns.iter().map(|c| c.to_lowercase()).collect();

    for typ in &schema.types {
        for prop in &typ.properties {
            let prop_name = prop.name.to_lowercase();
            let old = *column_indexes.get(&prop_name).unwrap_or(&false);
            column_indexes.insert(prop_name.to_string(), prop.indexed || old);

            if !all_items_columns.contains(&prop_name) {
                declared_columns.insert(prop.name.to_string(), prop.dbtype);
            }
        }
    }
    Ok((column_indexes, declared_columns))
}

// Solution taken from here:
// https://stackoverflow.com/questions/18920136/check-if-a-column-exists-in-sqlite
//
// Note that the approach of querying `pragma_table_info`
// does not work on older sqlcipher versions (ubuntu 20.04 still uses sqlcipher 3.4).
fn get_all_columns_pragma(table: &str, conn: &Connection) -> rusqlite::Result<HashSet<String>> {
    let sql = format!("PRAGMA table_info('{}');", table);
    let mut stmt = conn.prepare_cached(&sql)?;
    let mut rows = stmt.query(NO_PARAMS)?;
    let mut result = HashSet::new();
    while let Some(row) = rows.next()? {
        let column_name: String = row
            .get(1)
            .expect("Column 1 of PRAGMA table_info code is not a table column name");
        result.insert(column_name);
    }
    Ok(result)
}

fn generate_sql(
    column_indexes: HashMap<String, bool>,
    declared_columns: HashMap<String, SchemaPropertyType>,
) -> String {
    let mut result = String::new();

    for (column, db_type) in declared_columns {
        let db_type = match db_type {
            SchemaPropertyType::Bool => "INTEGER /* boolean */".to_string(),
            SchemaPropertyType::DateTime => "INTEGER /* datetime */".to_string(),
            db_type => format!("{:?}", db_type).to_uppercase(),
        };
        let creation = format!("ALTER TABLE items ADD {} {};", column, db_type);
        result.push_str(&creation);
        result.push_str("\n");
    }

    for (column, is_indexed) in column_indexes {
        let index = if is_indexed {
            format!(
                "CREATE INDEX IF NOT EXISTS idx_schema_items_{} ON items({}) WHERE {} IS NOT NULL;",
                column, column, column
            )
        } else {
            format!("DROP INDEX IF EXISTS idx_schema_items_{};", column)
        };
        result.push_str(&index);
        result.push_str("\n");
    }

    result
}
