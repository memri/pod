//
// Pod triggers that should "run" on particular DB/data changes (e.g. item insertion)
//

use crate::api_model::CreateItem;
use crate::command_line_interface::CliOptions;
use crate::database_api;
use crate::database_api::Rowid;
use crate::error::ErrorContext;
use crate::error::Result;
use crate::internal_api;
use crate::plugin_auth_crypto::DatabaseKey;
use crate::plugin_run;
use crate::schema;
use crate::schema::Schema;
use crate::schema::SchemaPropertyType;
use rusqlite::Transaction as Tx;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SchemaItem {
    pub item_type: String,
    pub property_name: String,
    pub value_type: SchemaPropertyType,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartPluginItem {
    pub container: String,
    pub target_item_id: String,
}

pub fn trigger_before_item_create(tx: &Tx, item: &CreateItem) -> Result<()> {
    // We'll do something ugly here.
    // We'll convert the item into JSON and back into the desired type for type check and parsing.
    // This is easier code-wise than to do manual conversions.
    // It only triggers for specific, rarely used items. This implementation might change later.
    if item._type == "ItemPropertySchema" {
        let json = serde_json::to_value(item)?;
        let parsed: SchemaItem = serde_json::from_value(json)
            .context(|| format!("Parsing of Schema item {:?}, {}:{}", item, file!(), line!()))?;
        schema::validate_property_name(&parsed.property_name)
            .context_str("Failed to add Schema property, name invalid")?;
        database_api::delete_schema_items_by_item_type_and_prop(
            tx,
            &parsed.item_type,
            &parsed.property_name,
        )?;
    }
    Ok(())
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
    if item._type == "StartPlugin" {
        let json = internal_api::get_item_from_rowid(tx, schema, source_rowid)?;
        let parsed: StartPluginItem = serde_json::from_value(json)
            .context(|| format!("Parsing of item {:?}, {}:{}", item, file!(), line!()))?;
        plugin_run::run_plugin_container(
            tx,
            schema,
            parsed.container,
            &parsed.target_item_id,
            source_id,
            pod_owner,
            database_key,
            cli,
        )?;
    }
    Ok(())
}
