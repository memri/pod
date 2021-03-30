use crate::schema::validate_property_name;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
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
        if typ.name.starts_with('_') {
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
            if prop.name.starts_with('_') {
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
