use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

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
