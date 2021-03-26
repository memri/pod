use crate::api_model::BulkAction;
use crate::api_model::CreateItem;
use crate::api_model::DeleteEdge;
use crate::api_model::Search;
use crate::database_api;
use crate::error::Error;
use crate::error::Result;
use crate::schema::validate_property_name;
use crate::schema::Schema;
use crate::schema::SchemaPropertyType;
use crate::sql_converters::borrow_sql_params;
use crate::sql_converters::json_value_to_sqlite;
use crate::sql_converters::sqlite_row_to_map;
use crate::triggers;
use chrono::Utc;
use log::info;
use log::warn;
use rusqlite::params;
use rusqlite::ToSql;
use rusqlite::Transaction;
use serde_json::Map;
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
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

    let mut stmt = tx.prepare_cached("SELECT name, value FROM integers WHERE item = ? ;")?;
    let mut integers = stmt.query(params![rowid])?;
    while let Some(row) = integers.next()? {
        let name: String = row.get(0)?;
        let value: i64 = row.get(1)?;
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

    let mut stmt = tx.prepare_cached("SELECT name, value FROM strings WHERE item = ? ;")?;
    let mut integers = stmt.query(params![rowid])?;
    while let Some(row) = integers.next()? {
        let name: String = row.get(0)?;
        let value: String = row.get(1)?;
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
        };
    }

    let mut stmt = tx.prepare_cached("SELECT name, value FROM reals WHERE item = ? ;")?;
    let mut integers = stmt.query(params![rowid])?;
    while let Some(row) = integers.next()? {
        let name: String = row.get(0)?;
        let value: f64 = row.get(1)?;
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

pub fn get_item_tx(tx: &Transaction, schema: &Schema, id: &str) -> Result<Vec<Value>> {
    info!("Getting item {}", id);
    let search_query = Search {
        id: Some(id.to_string()),
        _type: None,
        _date_server_modified_gte: None,
        _date_server_modified_lt: None,
        deleted: None,
        other_properties: Default::default(),
    };
    let result = search(tx, schema, search_query)?;
    Ok(result)
}

fn is_array_or_object(value: &Value) -> bool {
    matches!(value, Value::Array(_) | Value::Object(_))
}

fn execute_sql(tx: &Transaction, sql: &str, fields: &HashMap<String, Value>) -> Result<()> {
    let mut sql_params = Vec::new();
    for (key, value) in fields {
        sql_params.push((format!(":{}", key), json_value_to_sqlite(value, key)?));
    }
    let sql_params: Vec<_> = sql_params
        .iter()
        .map(|(field, value)| (field.as_str(), value as &dyn ToSql))
        .collect();
    let mut stmt = tx.prepare_cached(&sql)?;
    stmt.execute_named(&sql_params).map_err(|err| {
        let msg = format!(
            "Database rusqlite error for parameters: {:?}, {}",
            fields, err
        );
        Error {
            code: StatusCode::BAD_REQUEST,
            msg,
        }
    })?;
    Ok(())
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

pub fn create_item_tx(tx: &Transaction, schema: &Schema, item: CreateItem) -> Result<i64> {
    let default_id: String;
    let id = if let Some(id) = &item.id {
        id
    } else {
        default_id = new_uuid();
        &default_id
    };
    let time_now = Utc::now().timestamp_millis();
    triggers::trigger_before_item_create(tx, schema, &item)?;
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
    Ok(rowid)
}

pub fn update_item_tx(
    tx: &Transaction,
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
    if !fields.is_empty() {
        return Err(Error {
            code: StatusCode::NOT_IMPLEMENTED,
            msg: format!("Updating of non-base item properties not implemented yet"),
        });
    }
    Ok(())
}

pub fn update_item_tx_old(
    tx: &Transaction,
    uid: i64,
    fields: HashMap<String, Value>,
) -> Result<()> {
    let mut fields: HashMap<String, Value> = fields
        .into_iter()
        .filter(|(k, v)| !is_array_or_object(v) && validate_property_name(k).is_ok())
        .collect();
    fields.insert("uid".to_string(), uid.into());
    fields.remove("_type");
    fields.remove("dateCreated");

    let time_now = Utc::now().timestamp_millis();
    if !fields.contains_key("dateModified") {
        fields.insert("dateModified".to_string(), time_now.into());
    }
    fields.insert("dateServerModified".to_string(), time_now.into());

    let mut sql = "UPDATE items SET ".to_string();
    let mut after_first = false;
    for key in fields.keys() {
        if after_first {
            sql.push_str(", ");
        }
        after_first = true;
        sql.push_str(key);
        sql.push_str(" = :");
        sql.push_str(key);
    }
    sql.push_str(", version = version + 1 ");
    sql.push_str("WHERE uid = :uid ;");
    execute_sql(tx, &sql, &fields)
}

fn create_edge(
    _tx: &Transaction,
    _type: &str,
    _source: &str,
    _target: &str,
    _fields: HashMap<String, Value>,
) -> Result<()> {
    // let source_rowid = if let Some(rowid) = get_item_rowid(tx, source)? {
    //     rowid
    // } else {
    //     return Err(Error {
    //         code: StatusCode::BAD_REQUEST,
    //         msg: format!("Cannot create edge, source not found in edge {}->{} ({})", source, target, _type)
    //     })
    // };
    // let target_rowid = if let Some(rowid) = get_item_rowid(tx, target)? {
    //     rowid
    // } else {
    //     return Err(Error {
    //         code: StatusCode::BAD_REQUEST,
    //         msg: format!("Cannot create edge, target not found in edge {}->{} ({})", source, target, _type)
    //     })
    // };
    // fields.insert("_type".to_string(), _type.into());
    // fields.insert("_source".to_string(), source.into());
    // fields.insert("_target".to_string(), target.into());
    // let fields: HashMap<String, Value> = fields
    //     .into_iter()
    //     .filter(|(k, v)| !is_array_or_object(v) && validate_property_name(k).is_ok())
    //     .collect();
    // let mut sql = "INSERT INTO edges (".to_string();
    // let keys: Vec<_> = fields.keys().collect();
    // write_sql_body(&mut sql, &keys, ", ");
    // sql.push_str(") VALUES (:");
    // write_sql_body(&mut sql, &keys, ", :");
    // sql.push_str(");");
    // if !check_item_exists(tx, source)? {
    //     return Err(Error {
    //         code: StatusCode::NOT_FOUND,
    //         msg: format!(
    //             "Failed to create edge {} {}->{} because source uid is not found",
    //             _type, source, target
    //         ),
    //     });
    // };
    // if !check_item_exists(tx, target)? {
    //     return Err(Error {
    //         code: StatusCode::NOT_FOUND,
    //         msg: format!(
    //             "Failed to create edge {} {}->{} because target uid is not found",
    //             _type, source, target
    //         ),
    //     });
    // };
    // execute_sql(tx, &sql, &fields)
    panic!()
}

fn delete_edge_tx(tx: &Transaction, edge: DeleteEdge) -> Result<usize> {
    let sql =
        "DELETE FROM edges WHERE _source = :_source AND _target = :_target AND _type = :_type;";
    let sql_params = vec![
        (":_source".to_string(), edge._source.into()),
        (":_target".to_string(), edge._target.into()),
        (":_type".to_string(), edge._type.into()),
    ];
    let sql_params = borrow_sql_params(&sql_params);
    let mut stmt = tx.prepare_cached(&sql)?;
    let result = stmt.execute_named(sql_params.as_slice())?;
    Ok(result)
}

pub fn delete_item_tx(tx: &Transaction, uid: i64) -> Result<()> {
    let mut fields = HashMap::new();
    let time_now = Utc::now().timestamp_millis();
    fields.insert("deleted".to_string(), true.into());
    fields.insert("dateModified".to_string(), time_now.into());
    fields.insert("dateServerModified".to_string(), time_now.into());
    update_item_tx_old(tx, uid, fields)
}

pub fn bulk_action_tx(tx: &Transaction, schema: &Schema, bulk_action: BulkAction) -> Result<()> {
    info!(
        "Performing bulk action with {} new items, {} updated items, {} deleted items, {} created edges, {} deleted edges",
        bulk_action.create_items.len(),
        bulk_action.update_items.len(),
        bulk_action.delete_items.len(),
        bulk_action.create_edges.len(),
        bulk_action.delete_edges.len(),
    );
    let edges_will_be_created = !bulk_action.create_edges.is_empty();
    for item in bulk_action.create_items {
        if !item.fields.contains_key("uid") && edges_will_be_created {
            return Err(Error {
                code: StatusCode::BAD_REQUEST,
                msg: format!("Creating items without uid in bulk_action is restricted for safety reasons (creating edges might be broken), item: {:?}", item.fields)
            });
        }
        create_item_tx(tx, schema, item)?;
    }
    for item in bulk_action.update_items {
        update_item_tx_old(tx, item.uid, item.fields)?;
    }
    let sources_set: HashSet<_> = bulk_action
        .create_edges
        .iter()
        .map(|e| e._source.to_string())
        .collect();
    for edge in bulk_action.create_edges {
        create_edge(tx, &edge._type, &edge._source, &edge._target, edge.fields)?;
    }
    for edge_source in sources_set {
        println!("{}", edge_source);
        let edge_source = -1; // TODO
        update_item_tx_old(tx, edge_source, HashMap::new())?;
    }
    for edge_uid in bulk_action.delete_items {
        delete_item_tx(tx, edge_uid)?;
    }
    for del_edge in bulk_action.delete_edges {
        delete_edge_tx(tx, del_edge)?;
    }
    Ok(())
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
    let items = database_api::search_items(
        tx,
        None,
        query.id.as_deref(),
        query._type.as_deref(),
        query._date_server_modified_gte,
        query._date_server_modified_lt,
        query.deleted,
    )?;
    let mut result = Vec::new();
    for item in items {
        let mut properties = get_item_properties(tx, item.rowid, schema)?;
        properties.insert("id".to_string(), item.id.into());
        properties.insert("type".to_string(), item._type.into());
        properties.insert("dateCreated".to_string(), item.date_created.into());
        properties.insert("dateModified".to_string(), item.date_modified.into());
        properties.insert(
            "dateServerModified".to_string(),
            item.date_server_modified.into(),
        );
        properties.insert("deleted".to_string(), item.deleted.into());
        result.push(Value::Object(properties))
    }
    Ok(result)
}

pub fn get_item_with_edges_tx(tx: &Transaction, uid: i64) -> Result<Value> {
    let mut stmt_item = tx.prepare_cached("SELECT * FROM items WHERE uid = :uid")?;
    let mut item_rows = stmt_item.query_named(&[(":uid", &uid)])?;
    let mut item = match item_rows.next()? {
        Some(row) => sqlite_row_to_map(row, false)?,
        None => {
            return Err(Error {
                code: StatusCode::NOT_FOUND,
                msg: format!("Item with uid {} not found", uid),
            })
        }
    };
    assert!(
        item_rows.next()?.is_none(),
        "Impossible to get multiple results for a single uid"
    );

    let mut stmt_edge = tx.prepare_cached(
        "SELECT _type, sequence, edgeLabel, _target FROM edges WHERE _source = :_source",
    )?;
    let mut edge_rows = stmt_edge.query_named(&[(":_source", &uid)])?;
    let mut edges = Vec::new();
    while let Some(row) = edge_rows.next()? {
        edges.push(sqlite_row_to_map(row, false)?);
    }

    let mut new_edges = Vec::new();
    for mut edge in edges {
        let target = edge
            .get("_target")
            .expect("Failed to get _target")
            .as_i64()
            .expect("Failed to get value as i64");
        let mut stmt = tx.prepare_cached("SELECT * FROM items WHERE uid = :uid")?;
        let mut rows = stmt.query_named(&[(":uid", &target)])?;
        edge.remove("_target");
        while let Some(row) = rows.next()? {
            edge.insert(
                "_target".to_string(),
                Value::from(sqlite_row_to_map(row, true)?),
            );
        }
        new_edges.push(edge);
    }

    item.insert("allEdges".to_string(), Value::from(new_edges));
    Ok(item.into())
}

pub fn get_items_with_edges_tx(tx: &Transaction, uids: &[i64]) -> Result<Vec<Value>> {
    let mut result = Vec::new();
    for uid in uids {
        result.push(get_item_with_edges_tx(tx, *uid)?);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::api_model::CreateItem;
    use crate::database_migrate_refinery;
    use crate::internal_api;
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
    fn test_item_insert_schema() {
        let mut conn = new_conn();
        let minimal_schema = minimal_schema();

        {
            let tx = conn.transaction().unwrap();
            let json = json!({
                "type": "ItemPropertySchema",
                "itemType": "Person",
                "propertyName": "age",
                "valueType": "Integer",
            });
            let create_item: CreateItem = serde_json::from_value(json.clone()).unwrap();
            let result = internal_api::create_item_tx(&tx, &minimal_schema, create_item).unwrap();
            assert!(result < 10); // no more than 10 items existed before in the DB

            let bad_empty_schema = Schema {
                property_types: HashMap::new(),
            };
            let create_item: CreateItem = serde_json::from_value(json).unwrap();
            let result =
                internal_api::create_item_tx(&tx, &bad_empty_schema, create_item).unwrap_err();
            assert_eq!(result.code, StatusCode::BAD_REQUEST);
            assert!(result.msg.contains("not defined in Schema"));

            tx.commit().unwrap();
        }
    }
}
