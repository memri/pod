//
// Pod triggers that should "run" on particular DB/data changes (e.g. item insertion)
//

use crate::api_model::CreateItem;
use crate::command_line_interface::CliOptions;
use crate::database_api::Rowid;
use crate::database_utils::get_item_from_rowid;
use crate::error::Error;
use crate::error::ErrorContext;
use crate::error::Result;
use crate::plugin_auth_crypto::DatabaseKey;
use crate::plugin_run;
use crate::schema::Schema;
use crate::schema::SchemaPropertyType;
use crate::triggers::SchemaAdditionChange::*;
use rusqlite::Transaction as Tx;
use serde::Deserialize;
use serde::Serialize;
use warp::http::StatusCode;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SchemaItem {
    pub item_type: String,
    pub property_name: String,
    pub value_type: SchemaPropertyType,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PluginRunItem {
    pub container_image: String,
    pub target_item_id: String,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SchemaAdditionChange {
    NotASchema,
    NewSchemaAdded,
    OldSchemaIgnored,
}

/// If an item is a Schema, add it to the schema. Return the change.
/// Fail if an incompatible schema is attempted to be inserted.
pub fn add_item_as_schema_opt(
    schema: &mut Schema,
    item: &CreateItem,
) -> Result<SchemaAdditionChange> {
    // We'll do something ugly here.
    // We'll convert the item into JSON and back into the desired type for type check and parsing.
    // This is easier code-wise than to do manual conversions.
    // It only triggers for specific, rarely used items. This implementation might change later.
    if item._type == "ItemPropertySchema" {
        let json = serde_json::to_value(item)?;
        let parsed: SchemaItem = serde_json::from_value(json)
            .context(|| format!("Parsing of Schema item {:?}, {}:{}", item, file!(), line!()))?;
        if let Some(old) = schema.property_types.get(&parsed.property_name) {
            if old == &parsed.value_type {
                Ok(OldSchemaIgnored)
            } else {
                Err(Error {
                    code: StatusCode::BAD_REQUEST,
                    msg: format!("Schema for property {} is already defined to type {}, cannot override to type {}", parsed.property_name, old, parsed.value_type)
                })
            }
        } else {
            schema
                .property_types
                .insert(parsed.property_name, parsed.value_type);
            Ok(NewSchemaAdded)
        }
    } else {
        Ok(NotASchema)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn trigger_after_item_create(
    tx: &Tx,
    schema: &Schema,
    source_rowid: Rowid,
    source_id: &str,
    item: &CreateItem,
    pod_owner: &str,
    cli: &CliOptions,
    database_key: &DatabaseKey,
) -> Result<()> {
    if item._type == "PluginRun" {
        let json = get_item_from_rowid(tx, schema, source_rowid)?;
        let parsed: PluginRunItem = serde_json::from_value(json)
            .context(|| format!("Parsing of item {:?}, {}:{}", item, file!(), line!()))?;
        plugin_run::run_plugin_container(
            tx,
            schema,
            parsed.container_image,
            &parsed.target_item_id,
            source_id,
            pod_owner,
            database_key,
            cli,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::add_item_as_schema_opt;
    use crate::api_model::CreateItem;
    use crate::error::Result;
    use crate::schema::Schema;
    use crate::schema::SchemaPropertyType;
    use crate::triggers::SchemaAdditionChange;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn my_test() -> Result<()> {
        // let mut minimal_schema = database_api::get_schema(&tx).unwrap();
        let mut schema = Schema {
            property_types: HashMap::new(),
        };
        schema
            .property_types
            .insert("age".to_string(), SchemaPropertyType::Integer);

        let json = json!({
            "type": "Something"
        });
        let create_item: CreateItem = serde_json::from_value(json).unwrap();
        let result = add_item_as_schema_opt(&mut schema, &create_item);
        assert_eq!(result, Ok(SchemaAdditionChange::NotASchema));

        let json = json!({
            "type": "ItemPropertySchema",
            "itemType": "Person",
            "propertyName": "age",
            "valueType": "Integer",
        });
        let create_item: CreateItem = serde_json::from_value(json).unwrap();
        let result = add_item_as_schema_opt(&mut schema, &create_item);
        assert_eq!(result, Ok(SchemaAdditionChange::OldSchemaIgnored));

        let json = json!({
            "type": "ItemPropertySchema",
            "itemType": "Person",
            "propertyName": "agility",
            "valueType": "Integer",
        });
        let create_item: CreateItem = serde_json::from_value(json).unwrap();
        assert_eq!(schema.property_types.len(), 1);
        let result = add_item_as_schema_opt(&mut schema, &create_item);
        assert_eq!(schema.property_types.len(), 2);
        assert_eq!(
            schema.property_types.get("agility"),
            Some(SchemaPropertyType::Integer).as_ref()
        );
        assert_eq!(result, Ok(SchemaAdditionChange::NewSchemaAdded));

        let json = json!({
            "type": "ItemPropertySchema",
            "itemType": "Person",
            "propertyName": "age",
            "valueType": "Text",
        });
        let create_item: CreateItem = serde_json::from_value(json).unwrap();
        let result = add_item_as_schema_opt(&mut schema, &create_item);
        assert!(result.is_err(), "result should be an error {:?}", result);

        Ok(())
    }
}
