use crate::error::{Error, Result};
use serde::Deserialize;
use serde::Serialize;
use std::any::type_name;
use std::collections::HashMap;
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

impl SchemaPropertyType {
    pub fn from_string(str: &str) -> Result<SchemaPropertyType> {
        match str {
            "text" => Ok(SchemaPropertyType::Text),
            "integer" => Ok(SchemaPropertyType::Integer),
            "real" => Ok(SchemaPropertyType::Real),
            "bool" => Ok(SchemaPropertyType::Bool),
            "datetime" => Ok(SchemaPropertyType::DateTime),
            _ => Err(Error {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                msg: format!(
                    "Failed to parse {} into {}",
                    str,
                    type_name::<SchemaPropertyType>()
                ),
            }),
        }
    }
}

pub struct Schema {
    pub property_types: HashMap<String, SchemaPropertyType>,
}

// impl Schema {
//     pub fn new_empty() -> Schema {
//         Schema {
//             property_types: HashMap::new(),
//         }
//     }
// }
