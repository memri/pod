use crate::api_model::Bulk;
use crate::api_model::CreateItem;
use crate::api_model::Search;
use crate::api_model::SortOrder;
use crate::command_line_interface::CliOptions;
use crate::database_api;
use crate::database_api::DatabaseSearch;
use crate::database_api::IntegersNameValue;
use crate::database_api::ItemBase;
use crate::database_api::RealsNameValue;
use crate::database_api::Rowid;
use crate::database_api::StringsNameValue;
use crate::error::Error;
use crate::error::Result;
use crate::plugin_auth_crypto::DatabaseKey;
use crate::schema;
use crate::schema::validate_property_name;
use crate::schema::Schema;
use crate::schema::SchemaPropertyType;
use crate::triggers;
use chrono::Utc;
use log::info;
use log::warn;
use rusqlite::Transaction;
use serde_json::Map;
use serde_json::Value;
use std::collections::HashMap;
use std::str;
use warp::http::status::StatusCode;

pub fn get_project_version() -> String {
    crate::command_line_interface::VERSION.to_string()
}

/// Get all properties that the item has, ignoring those
/// that exist in the DB but are not defined in the Schema
pub fn get_item_properties(
    tx: &Transaction,
    rowid: i64,
    schema: &Schema,
) -> Result<Map<String, Value>> {
    let mut json = serde_json::Map::new();

    for IntegersNameValue { name, value } in database_api::get_integers_records_for_item(tx, rowid)?
    {
        match schema.property_types.get(&name) {
            Some(SchemaPropertyType::Bool) => {
                json.insert(name, (value == 1).into());
            }
            Some(SchemaPropertyType::DateTime) | Some(SchemaPropertyType::Integer) => {
                json.insert(name, value.into());
            }
            other => {
                log::warn!(
                    "Ignoring item property {}: {} which according to Schema should be a {:?}",
                    name,
                    value,
                    other
                );
            }
        };
    }

    for StringsNameValue { name, value } in database_api::get_strings_records_for_item(tx, rowid)? {
        match schema.property_types.get(&name) {
            Some(SchemaPropertyType::Text) => {
                json.insert(name, value.into());
            }
            other => {
                log::warn!(
                    "Ignoring item property {}: {} which according to Schema should be a {:?}",
                    name,
                    value,
                    other
                );
            }
        }
    }

    for RealsNameValue { name, value } in database_api::get_reals_records_for_item(tx, rowid)? {
        match schema.property_types.get(&name) {
            Some(SchemaPropertyType::Real) => {
                json.insert(name, value.into());
            }
            other => {
                log::warn!(
                    "Ignoring item property {}: {} which according to Schema should be a {:?}",
                    name,
                    value,
                    other
                );
            }
        };
    }

    Ok(json)
}

pub fn get_item_from_rowid(tx: &Transaction, schema: &Schema, rowid: Rowid) -> Result<Value> {
    let database_search = DatabaseSearch {
        rowid: Some(rowid),
        id: None,
        _type: None,
        date_server_modified_gte: None,
        date_server_modified_lt: None,
        deleted: None,
        sort_order: SortOrder::Asc,
        _limit: 1,
    };
    let item = database_api::search_items(tx, &database_search)?;
    let item = if let Some(item) = item.into_iter().next() {
        item
    } else {
        return Err(Error {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: format!("Item rowid {} not found right after inserting", rowid),
        });
    };
    let mut props = get_item_properties(tx, rowid, schema)?;
    add_item_base_properties(&mut props, item);
    Ok(Value::Object(props))
}

pub fn get_item_tx(tx: &Transaction, schema: &Schema, id: &str) -> Result<Vec<Value>> {
    info!("Getting item {}", id);
    let search_query = Search {
        id: Some(id.to_string()),
        _type: None,
        date_server_modified_gte: None,
        date_server_modified_lt: None,
        deleted: None,
        sort_order: SortOrder::Asc,
        limit: 1,
        other_properties: Default::default(),
    };
    let result = search(tx, schema, search_query)?;
    Ok(result)
}

fn new_uuid() -> String {
    uuid::Uuid::new_v4().to_simple().to_string()
}

fn insert_property(
    tx: &Transaction,
    schema: &Schema,
    rowid: i64,
    name: &str,
    json: &Value,
) -> Result<()> {
    let dbtype = if let Some(t) = schema.property_types.get(name) {
        t
    } else {
        return Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!(
                "Property {} not defined in Schema (attempted to use it for json value {})",
                name, json,
            ),
        });
    };
    database_api::delete_property(tx, rowid, name)?;

    match json {
        Value::Null => (),
        Value::String(value) if dbtype == &SchemaPropertyType::Text => {
            database_api::insert_string(tx, rowid, name, value)?
        }
        Value::Number(n) if dbtype == &SchemaPropertyType::Integer => {
            if let Some(value) = n.as_i64() {
                database_api::insert_integer(tx, rowid, name, value)?
            } else {
                return Err(Error {
                    code: StatusCode::BAD_REQUEST,
                    msg: format!("Failed to parse JSON number {} to i64 ({})", n, name),
                });
            }
        }
        Value::Number(n) if dbtype == &SchemaPropertyType::Real => {
            if let Some(value) = n.as_f64() {
                database_api::insert_real(tx, rowid, name, value)?
            } else {
                return Err(Error {
                    code: StatusCode::BAD_REQUEST,
                    msg: format!("Failed to parse JSON number {} to f64 ({})", n, name),
                });
            }
        }
        Value::Bool(b) if dbtype == &SchemaPropertyType::Bool => {
            database_api::insert_integer(tx, rowid, name, if *b { 1 } else { 0 })?
        }
        Value::Number(n) if dbtype == &SchemaPropertyType::DateTime => {
            if let Some(value) = n.as_i64() {
                database_api::insert_integer(tx, rowid, name, value)?
            } else if let Some(float) = n.as_f64() {
                warn!("Using float-to-integer conversion property {}, value {}. This might not be supported in the future, please use a compatible DateTime format https://gitlab.memri.io/memri/pod#understanding-the-schema", float, name);
                database_api::insert_integer(tx, rowid, name, float.round() as i64)?
            } else {
                return Err(Error {
                    code: StatusCode::BAD_REQUEST,
                    msg: format!(
                        "Failed to parse JSON number {} to DateTime ({}), use i64 number instead",
                        n, name
                    ),
                });
            }
        }
        _ => {
            return Err(Error {
                code: StatusCode::BAD_REQUEST,
                msg: format!(
                    "Failed to parse json value {} to {:?} ({})",
                    json, dbtype, name
                ),
            })
        }
    };
    Ok(())
}

pub fn create_item_tx(
    tx: &Transaction,
    schema: &Schema,
    item: CreateItem,
    pod_owner: &str,
    cli: &CliOptions,
    database_key: &DatabaseKey,
) -> Result<i64> {
    let default_id: String;
    let id = if let Some(id) = &item.id {
        id
    } else {
        default_id = new_uuid();
        &default_id
    };
    if let Err(err) = schema::validate_create_item_id(id) {
        return Err(err);
    }
    let time_now = Utc::now().timestamp_millis();
    triggers::trigger_before_item_create(tx, &item)?;
    let rowid = database_api::insert_item_base(
        tx,
        id,
        &item._type,
        item.date_created.unwrap_or(time_now),
        item.date_modified.unwrap_or(time_now),
        time_now,
        item.deleted,
    )?;
    for (prop_name, prop_value) in &item.fields {
        insert_property(tx, schema, rowid, &prop_name, &prop_value)?;
    }
    triggers::trigger_after_item_create(
        tx,
        schema,
        rowid,
        id,
        &item,
        pod_owner,
        cli,
        database_key,
    )?;
    Ok(rowid)
}

pub fn update_item_tx(
    tx: &Transaction,
    schema: &Schema,
    id: &str,
    mut fields: HashMap<String, Value>,
) -> Result<()> {
    log::debug!("Updating item {}", id);
    for k in fields.keys() {
        validate_property_name(k)?;
    }
    fields.remove("type");
    fields.remove("dateCreated");

    let time_now = Utc::now().timestamp_millis();
    let date_modified = if let Some(dm) = fields.remove("dateModified") {
        if let Some(dm) = dm.as_i64() {
            dm
        } else {
            return Err(Error {
                code: StatusCode::BAD_REQUEST,
                msg: format!("Cannot parse dateModified {} to i64", dm),
            });
        }
    } else {
        time_now
    };
    fields.remove("dateServerModified");
    let deleted = if let Some(d) = fields.remove("deleted") {
        match d {
            Value::Null => None,
            Value::Bool(d) => Some(d),
            _ => {
                return Err(Error {
                    code: StatusCode::BAD_REQUEST,
                    msg: format!("Failed to parse deleted {} to bool", d),
                })
            }
        }
    } else {
        None
    };
    let rowid = database_api::get_item_rowid(tx, id)?.ok_or_else(|| Error {
        code: StatusCode::NOT_FOUND,
        msg: format!("Item with id {} not found", id),
    })?;
    database_api::update_item_base(tx, rowid, date_modified, time_now, deleted)?;
    for (k, v) in &fields {
        insert_property(tx, schema, rowid, k, v)?;
    }
    Ok(())
}

pub fn delete_item_tx(tx: &Transaction, schema: &Schema, id: &str) -> Result<()> {
    log::debug!("Deleting item {}", id);
    let mut fields = HashMap::new();
    fields.insert("deleted".to_string(), true.into());
    update_item_tx(tx, schema, id, fields)
}

pub fn bulk_tx(
    tx: &Transaction,
    schema: &Schema,
    bulk: Bulk,
    pod_owner: &str,
    cli: &CliOptions,
    database_key: &DatabaseKey,
) -> Result<()> {
    info!(
        "Performing bulk action with {} new items, {} updated items, {} deleted items",
        bulk.create_items.len(),
        bulk.update_items.len(),
        bulk.delete_items.len(),
    );
    for item in bulk.create_items {
        create_item_tx(tx, schema, item, pod_owner, cli, database_key)?;
    }
    for item in bulk.update_items {
        update_item_tx(tx, schema, &item.id, item.fields)?;
    }
    for item_id in bulk.delete_items {
        delete_item_tx(tx, schema, &item_id)?;
    }
    Ok(())
}

fn add_item_base_properties(props: &mut Map<String, Value>, item: ItemBase) {
    props.insert("id".to_string(), item.id.into());
    props.insert("type".to_string(), item._type.into());
    props.insert("dateCreated".to_string(), item.date_created.into());
    props.insert("dateModified".to_string(), item.date_modified.into());
    props.insert(
        "dateServerModified".to_string(),
        item.date_server_modified.into(),
    );
    props.insert("deleted".to_string(), item.deleted.into());
}

pub fn search(tx: &Transaction, schema: &Schema, query: Search) -> Result<Vec<Value>> {
    info!("Searching by fields {:?}", query);
    if !query.other_properties.is_empty() {
        return Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!(
                "Cannot search by non-base properties: {:?}. \
                    In the future we will allow searching for other properties. \
                    See HTTP_API.md for details.",
                query.other_properties.keys()
            ),
        });
    }
    let database_search = DatabaseSearch {
        rowid: None,
        id: query.id.as_deref(),
        _type: query._type.as_deref(),
        date_server_modified_gte: query.date_server_modified_gte,
        date_server_modified_lt: query.date_server_modified_lt,
        deleted: query.deleted,
        sort_order: query.sort_order,
        _limit: query.limit,
    };
    let items = database_api::search_items(tx, &database_search)?;
    let mut result = Vec::new();
    for item in items {
        let mut properties = get_item_properties(tx, item.rowid, schema)?;
        add_item_base_properties(&mut properties, item);
        result.push(Value::Object(properties))
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::api_model::CreateItem;
    use crate::command_line_interface;
    use crate::database_api;
    use crate::database_migrate_refinery;
    use crate::error::Result;
    use crate::internal_api;
    use crate::plugin_auth_crypto::DatabaseKey;
    use crate::schema::Schema;
    use crate::schema::SchemaPropertyType;
    use rusqlite::Connection;
    use serde_json::json;
    use std::collections::HashMap;
    use warp::hyper::StatusCode;

    fn new_conn() -> Connection {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        database_migrate_refinery::embedded::migrations::runner()
            .run(&mut conn)
            .expect("Failed to run refinery migrations");
        conn
    }

    fn minimal_schema() -> Schema {
        let mut schema = HashMap::new();
        schema.insert("itemType".to_string(), SchemaPropertyType::Text);
        schema.insert("propertyName".to_string(), SchemaPropertyType::Text);
        schema.insert("valueType".to_string(), SchemaPropertyType::Text);
        Schema {
            property_types: schema,
        }
    }

    #[test]
    fn test_schema_checking() -> Result<()> {
        let mut conn = new_conn();
        let tx = conn.transaction().unwrap();
        let database_key = DatabaseKey::from("".to_string()).unwrap();
        let cli = command_line_interface::tests::new_cli();

        // first try to insert the Person without Schema
        let item_json = json!({
            "type": "Person",
            "age": 20,
        });
        let item_struct: CreateItem = serde_json::from_value(item_json).unwrap();
        let result = internal_api::create_item_tx(
            &tx,
            &database_api::get_schema(&tx)?,
            item_struct.clone(),
            "",
            &cli,
            &database_key,
        );
        assert_eq!(result.unwrap_err().code, StatusCode::BAD_REQUEST);

        // Then insert the Schema, it should succeed
        let schema_json = json!({
            "type": "ItemPropertySchema",
            "itemType": "Person",
            "propertyName": "age",
            "valueType": "Integer",
        });
        let schema_struct: CreateItem = serde_json::from_value(schema_json).unwrap();
        let result = internal_api::create_item_tx(
            &tx,
            &database_api::get_schema(&tx)?,
            schema_struct,
            "",
            &cli,
            &database_key,
        );
        assert!(result.is_ok());

        // Now insert the Person with already in place
        let result = internal_api::create_item_tx(
            &tx,
            &database_api::get_schema(&tx)?,
            item_struct,
            "",
            &cli,
            &database_key,
        );
        assert!(result.is_ok());

        Ok(())
    }
    #[test]
    fn test_item_insert_schema() {
        let mut conn = new_conn();
        let minimal_schema = minimal_schema();
        let cli = command_line_interface::tests::new_cli();
        let database_key = DatabaseKey::from("".to_string()).unwrap();

        let tx = conn.transaction().unwrap();
        let json = json!({
            "type": "ItemPropertySchema",
            "itemType": "Person",
            "propertyName": "age",
            "valueType": "Integer",
        });
        let create_item: CreateItem = serde_json::from_value(json.clone()).unwrap();
        internal_api::create_item_tx(&tx, &minimal_schema, create_item, "", &cli, &database_key)
            .unwrap();

        let bad_empty_schema = Schema {
            property_types: HashMap::new(),
        };
        let create_item: CreateItem = serde_json::from_value(json).unwrap();
        let result = internal_api::create_item_tx(
            &tx,
            &bad_empty_schema,
            create_item,
            "",
            &cli,
            &database_key,
        )
        .unwrap_err();
        assert_eq!(result.code, StatusCode::BAD_REQUEST);
        assert!(result.msg.contains("not defined in Schema"));

        tx.commit().unwrap();
    }
}
