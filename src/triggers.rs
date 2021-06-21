//
// Pod triggers that should "run" on particular DB/data changes (e.g. item insertion)
//

use crate::api_model::CreateItem;
use crate::command_line_interface::CliOptions;
use crate::database_api::Rowid;
use crate::error::Error;
use crate::error::ErrorContext;
use crate::error::Result;
use crate::internal_api;
use crate::plugin_auth_crypto::DatabaseKey;
use crate::plugin_run;
use crate::schema::Schema;
use crate::schema::SchemaPropertyType;
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

/// Returns whether the item insertion should be ignored
pub fn trigger_before_item_create(schema: &Schema, item: &CreateItem) -> Result<bool> {
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
                Ok(true)
            } else {
                Err(Error {
                    code: StatusCode::BAD_REQUEST,
                    msg: format!("Schema for property {} is already defined to type {}, cannot override to type {}", parsed.property_name, old, parsed.value_type)
                })
            }
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
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
        let json = internal_api::get_item_from_rowid(tx, schema, source_rowid)?;
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
