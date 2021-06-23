use crate::database_api;
use crate::database_api::IntegersNameValue;
use crate::database_api::ItemBase;
use crate::database_api::RealsNameValue;
use crate::database_api::Rowid;
use crate::database_api::StringsNameValue;
use crate::error::Error;
use crate::error::Result;
use crate::schema::Schema;
use crate::schema::SchemaPropertyType;
use log::warn;
use rusqlite::Transaction as Tx;
use serde_json::Map;
use serde_json::Value;
use std::collections::HashMap;
use std::str;
use warp::http::status::StatusCode;

/// Get all properties that the item has, ignoring those
/// that exist in the DB but are not defined in the Schema
pub fn get_item_properties(tx: &Tx, rowid: i64, schema: &Schema) -> Result<Map<String, Value>> {
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

pub fn get_item_from_rowid(tx: &Tx, schema: &Schema, rowid: Rowid) -> Result<Value> {
    let item = database_api::get_item_base(tx, rowid)?;
    let item = item.ok_or_else(|| Error {
        code: StatusCode::INTERNAL_SERVER_ERROR,
        msg: format!("Item rowid {} not found right after inserting", rowid),
    })?;
    let mut props = get_item_properties(tx, rowid, schema)?;
    add_item_base_properties(&mut props, item);
    Ok(Value::Object(props))
}

pub fn check_item_has_all_properties(
    tx: &Tx,
    schema: &Schema,
    rowid: i64,
    props: &HashMap<String, Value>,
) -> Result<bool> {
    for (name, value) in props {
        if !check_item_has_property(tx, schema, rowid, name, value)? {
            return Ok(false);
        }
    }
    Ok(true)
}

pub fn check_item_has_property(
    tx: &Tx,
    schema: &Schema,
    rowid: i64,
    name: &str,
    value: &Value,
) -> Result<bool> {
    let dbtype = if let Some(t) = schema.property_types.get(name) {
        t
    } else {
        return Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!(
                "Property {} not defined in Schema (attempted to use it for json value {})",
                name, value,
            ),
        });
    };

    match value {
        Value::Null => Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!(
                "Searching for undefined (null) properties is not supported yet. Attempted for {} ({})",
                name, dbtype
            ),
        }),
        Value::String(value) if dbtype == &SchemaPropertyType::Text => {
            database_api::check_string_exists(tx, rowid, name, value)
        }
        Value::Number(n) if dbtype == &SchemaPropertyType::Integer => {
            if let Some(value) = n.as_i64() {
                database_api::check_integer_exists(tx, rowid, name, value)
            } else {
                Err(Error {
                    code: StatusCode::BAD_REQUEST,
                    msg: format!("Failed to parse JSON number {} to i64 ({})", n, name),
                })
            }
        }
        Value::Number(n) if dbtype == &SchemaPropertyType::Real => {
            if let Some(value) = n.as_f64() {
                database_api::check_real_exists(tx, rowid, name, value)
            } else {
                Err(Error {
                    code: StatusCode::BAD_REQUEST,
                    msg: format!("Failed to parse JSON number {} to f64 ({})", n, name),
                })
            }
        }
        Value::Bool(b) if dbtype == &SchemaPropertyType::Bool => {
            database_api::check_integer_exists(tx, rowid, name, if *b { 1 } else { 0 })
        }
        Value::Number(n) if dbtype == &SchemaPropertyType::DateTime => {
            if let Some(value) = n.as_i64() {
                database_api::check_integer_exists(tx, rowid, name, value)
            } else if let Some(float) = n.as_f64() {
                warn!("Using float-to-integer conversion property {}, value {}. This might not be supported in the future, please use a compatible DateTime format https://gitlab.memri.io/memri/pod#understanding-the-schema", float, name);
                database_api::check_integer_exists(tx, rowid, name, float.round() as i64)
            } else {
                Err(Error {
                    code: StatusCode::BAD_REQUEST,
                    msg: format!(
                        "Failed to parse JSON number {} to DateTime ({}), use i64 number instead",
                        n, name
                    ),
                })
            }
        }
        _ => {
            Err(Error {
                code: StatusCode::BAD_REQUEST,
                msg: format!(
                    "Failed to parse json value {} to {:?} ({})",
                    value, dbtype, name
                ),
            })
        }
    }
}

pub fn insert_property(
    tx: &Tx,
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

pub fn item_base_to_json(tx: &Tx, item: ItemBase, schema: &Schema) -> Result<Map<String, Value>> {
    let mut props = get_item_properties(tx, item.rowid, schema)?;
    add_item_base_properties(&mut props, item);
    Ok(props)
}

pub fn add_item_edge_properties(
    tx: &Tx,
    props: &mut Map<String, Value>,
    rowid: Rowid,
) -> Result<()> {
    if let Some(edge_self) = database_api::get_self_edge(tx, rowid)? {
        let source_base =
            database_api::get_item_base(tx, edge_self.source)?.ok_or_else(|| Error {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                msg: format!(
                    "Edge {} connects to non-existing source {}",
                    rowid, edge_self.source
                ),
            })?;
        let target_base =
            database_api::get_item_base(tx, edge_self.target)?.ok_or_else(|| Error {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                msg: format!(
                    "Edge {} connects to non-existing target {}",
                    rowid, edge_self.target
                ),
            })?;
        props.insert("_source".to_string(), Value::String(source_base.id));
        props.insert("_target".to_string(), Value::String(target_base.id));
        props.insert("_edge".to_string(), Value::String(edge_self.name));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database_api::tests::new_conn;
    use crate::database_api::tests::random_id;
    use chrono::Utc;
    use serde_json::json;
    use std::ops::Not;

    #[test]
    fn test_one_property() -> Result<()> {
        let mut conn = new_conn();
        let tx = conn.transaction()?;
        let mut schema = database_api::get_schema(&tx).unwrap();
        schema
            .property_types
            .insert("age".to_string(), SchemaPropertyType::Integer);
        schema
            .property_types
            .insert("strength".to_string(), SchemaPropertyType::Real);
        schema
            .property_types
            .insert("myDescription".to_string(), SchemaPropertyType::Text);

        let date = Utc::now().timestamp_millis();
        let item: Rowid =
            database_api::insert_item_base(&tx, &random_id(), "Person", date, date, date, false)?;

        assert!(check_item_has_property(&tx, &schema, item, "age", &json!(20))?.not());

        insert_property(&tx, &schema, item, "age", &json!(20))?;
        assert!(check_item_has_property(
            &tx,
            &schema,
            item,
            "age",
            &json!(20)
        )?);
        assert!(check_item_has_property(&tx, &schema, item, "age", &json!(99))?.not());

        // Checking non-existing property should yield an error, not a successful "no" response
        assert!(check_item_has_property(&tx, &schema, item, "antiAge", &json!(99)).is_err());

        insert_property(&tx, &schema, item, "strength", &json!(13.5))?;
        assert!(check_item_has_property(
            &tx,
            &schema,
            item,
            "strength",
            &json!(13.5)
        )?);

        insert_property(
            &tx,
            &schema,
            item,
            "myDescription",
            &json!("Wow such person"),
        )?;
        assert!(check_item_has_property(
            &tx,
            &schema,
            item,
            "myDescription",
            &json!("Wow such person")
        )?);

        Ok(())
    }

    #[test]
    fn test_all_properties() -> Result<()> {
        let mut conn = new_conn();
        let tx = conn.transaction()?;
        let mut schema = database_api::get_schema(&tx).unwrap();
        schema
            .property_types
            .insert("age".to_string(), SchemaPropertyType::Integer);
        schema
            .property_types
            .insert("strength".to_string(), SchemaPropertyType::Real);
        schema
            .property_types
            .insert("myDescription".to_string(), SchemaPropertyType::Text);

        let date = Utc::now().timestamp_millis();
        let item: Rowid =
            database_api::insert_item_base(&tx, &random_id(), "Person", date, date, date, false)?;

        insert_property(&tx, &schema, item, "age", &json!(20))?;
        insert_property(&tx, &schema, item, "strength", &json!(13.5))?;
        insert_property(
            &tx,
            &schema,
            item,
            "myDescription",
            &json!("Wow such person"),
        )?;

        {
            let mut props = HashMap::new();
            props.insert("age".to_string(), json!(20));
            props.insert("strength".to_string(), json!(13.5));
            props.insert("myDescription".to_string(), json!("Wow such person"));
            assert!(check_item_has_all_properties(&tx, &schema, item, &props)?);
        }

        {
            let mut props = HashMap::new();
            props.insert("age".to_string(), json!(20));
            assert!(check_item_has_all_properties(&tx, &schema, item, &props)?);
        }

        {
            let mut props = HashMap::new();
            props.insert("age".to_string(), json!(99999999));
            props.insert("strength".to_string(), json!(13.5));
            assert!(check_item_has_all_properties(&tx, &schema, item, &props)?.not());
        }

        {
            let mut props = HashMap::new();
            props.insert("antiAge".to_string(), json!(-200000000));
            assert!(check_item_has_all_properties(&tx, &schema, item, &props).is_err());
        }

        {
            let props = HashMap::new();
            assert!(check_item_has_all_properties(&tx, &schema, item, &props)?);
        }

        Ok(())
    }
}
