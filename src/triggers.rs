//
// Pod triggers that should "run" on particular DB/data changes (e.g. item insertion)
//

use crate::api_model::CreateItem;
use crate::database_api;
use crate::error::ErrorContext;
use crate::error::Result;
use crate::schema::Schema;
use crate::schema::SchemaPropertyType;
use crate::schema;
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

pub fn trigger_before_item_create(tx: &Tx, _schema: &Schema, item: &CreateItem) -> Result<()> {
    if item._type == "ItemPropertySchema" {
        // We'll do something ugly here.
        // We'll convert the item into JSON and back into a better type: SchemaItem.
        // This is easier code-wise than to do manual conversions. It only triggers
        // for Schema items. This implementation might change later.
        let json = serde_json::to_value(&item).context(|| format!("item {:?}", item))?;
        let parsed: SchemaItem = serde_json::from_value(json)
            .context(|| format!("Parsing of Schema item {:?}", item))?;
        schema::validate_property_name(&parsed.property_name)?;
        database_api::delete_schema_items_by_item_type_and_prop(
            tx,
            &parsed.item_type,
            &parsed.property_name,
        )?;
    }
    Ok(())
}
