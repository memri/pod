use crate::api_model::Bulk;
use crate::api_model::CreateEdge;
use crate::api_model::CreateItem;
use crate::api_model::EdgeDirection;
use crate::api_model::GetEdges;
use crate::api_model::Search;
use crate::api_model::SortOrder;
use crate::command_line_interface::CliOptions;
use crate::database_api;
use crate::database_api::get_incoming_edges;
use crate::database_api::get_outgoing_edges;
use crate::database_api::DatabaseSearch;
use crate::database_api::EdgePointer;
use crate::database_api::Rowid;
use crate::database_utils::add_item_edge_properties;
use crate::database_utils::check_item_has_all_properties;
use crate::database_utils::insert_property;
use crate::database_utils::item_base_to_json;
use crate::error::Error;
use crate::error::Result;
use crate::plugin_auth_crypto::DatabaseKey;
use crate::schema;
use crate::schema::validate_property_name;
use crate::schema::Schema;
use crate::triggers;
use chrono::Utc;
use log::info;
use rand::Rng;
use rusqlite::Transaction as Tx;
use serde_json::Value;
use std::collections::HashMap;
use std::str;
use warp::http::status::StatusCode;

pub fn get_project_version() -> String {
    serde_json::json!({
        "cargo": std::env!("CARGO_PKG_VERSION"),
        "git_describe": std::option_env!("GIT_DESCRIBE")
    })
    .to_string()
}

pub fn get_item_tx(tx: &Tx, schema: &Schema, id: &str) -> Result<Vec<Value>> {
    info!("Getting item {}", id);
    let search_query = Search {
        id: Some(id.to_string()),
        _type: None,
        date_server_modified_gte: None,
        date_server_modified_lt: None,
        deleted: None,
        sort_order: SortOrder::Asc,
        limit: 1,
        offset: Some(0),
        forward_edges: None,
        backward_edges: None,
        other_properties: Default::default(),
    };
    let result = search(tx, schema, search_query)?;
    Ok(result)
}

/// Generate a new random item id.
/// This implementation chooses to generate 32 random hex characters.
pub fn new_random_item_id() -> String {
    new_random_string(32)
}
pub fn new_random_string(length: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..HEX_CHARACTERS.len());
            HEX_CHARACTERS[idx] as char
        })
        .collect()
}
const HEX_CHARACTERS: &[u8] = b"0123456789abcdef";

pub fn create_item_tx(
    tx: &Tx,
    schema: &Schema,
    item: CreateItem,
    pod_owner: &str,
    cli: &CliOptions,
    database_key: &DatabaseKey,
) -> Result<String> {
    let id: String = if let Some(id) = &item.id {
        id.to_string()
    } else {
        new_random_item_id()
    };
    if let Err(err) = schema::validate_create_item_id(&id) {
        return Err(err);
    }
    let time_now = Utc::now().timestamp_millis();
    let _ignore_insertion = triggers::trigger_before_item_create(schema, &item)?;
    let rowid = database_api::insert_item_base(
        tx,
        &id,
        &item._type,
        item.date_created.unwrap_or(time_now),
        item.date_modified.unwrap_or(time_now),
        time_now,
        item.deleted,
    )?;
    for (prop_name, prop_value) in &item.fields {
        insert_property(tx, schema, rowid, prop_name, prop_value)?;
    }
    triggers::trigger_after_item_create(
        tx,
        schema,
        rowid,
        &id,
        &item,
        pod_owner,
        cli,
        database_key,
    )?;
    Ok(id)
}

pub fn update_item_tx(
    tx: &Tx,
    schema: &Schema,
    id: &str,
    mut fields: HashMap<String, Value>,
) -> Result<()> {
    log::debug!("Updating item {}", id);
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
    for k in fields.keys() {
        validate_property_name(k)?;
    }
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

pub fn delete_item_tx(tx: &Tx, schema: &Schema, id: &str) -> Result<()> {
    log::debug!("Deleting item {}", id);
    let mut fields = HashMap::new();
    fields.insert("deleted".to_string(), true.into());
    update_item_tx(tx, schema, id, fields)
}

pub fn bulk_tx(
    tx: &Tx,
    schema: &Schema,
    bulk: Bulk,
    pod_owner: &str,
    cli: &CliOptions,
    database_key: &DatabaseKey,
) -> Result<Value> {
    info!(
        "Performing bulk action with {} new items, {} updated items, {} deleted items, {} created edges",
        bulk.create_items.len(),
        bulk.update_items.len(),
        bulk.delete_items.len(),
        bulk.create_edges.len(),
    );
    let mut created_items = Vec::new();
    for item in bulk.create_items {
        let id = create_item_tx(tx, schema, item, pod_owner, cli, database_key)?;
        created_items.push(id);
    }
    for item in bulk.update_items {
        update_item_tx(tx, schema, &item.id, item.fields)?;
    }
    for item_id in bulk.delete_items {
        delete_item_tx(tx, schema, &item_id)?;
    }
    let mut created_edges = Vec::new();
    for item_id in bulk.create_edges {
        let id = create_edge(tx, item_id)?;
        created_edges.push(id);
    }
    let mut search_results = Vec::new();
    for query in bulk.search {
        let result = search(tx, schema, query)?;
        search_results.push(result);
    }
    let result = serde_json::json!({
        "createItems": created_items,
        "createEdges": created_edges,
        "search": search_results,
    });
    Ok(result)
}

pub fn create_edge(tx: &Tx, query: CreateEdge) -> Result<String> {
    let CreateEdge {
        source,
        target,
        name,
        self_id,
    } = query;
    let date = Utc::now().timestamp_millis();

    let (self_rowid, self_id) = if let Some(id) = self_id {
        let self_rowid = database_api::get_item_rowid(tx, &id)?.ok_or_else(|| Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!("Failed to create edge with _self: {}", id),
        })?;
        (self_rowid, id)
    } else {
        let self_id = new_random_item_id();
        let self_rowid =
            database_api::insert_item_base(tx, &self_id, "Edge", date, date, date, false)?;
        (self_rowid, self_id)
    };

    let source: Rowid = database_api::get_item_rowid(tx, &source)?.ok_or_else(|| Error {
        code: StatusCode::NOT_FOUND,
        msg: format!("Edge source not found: {}", source),
    })?;
    let target: Rowid = database_api::get_item_rowid(tx, &target)?.ok_or_else(|| Error {
        code: StatusCode::NOT_FOUND,
        msg: format!("Edge target not found: {}", target),
    })?;
    database_api::insert_edge(tx, self_rowid, source, &name, target)?;
    database_api::update_item_date_server_modified(tx, source, date)?;
    Ok(self_id)
}

pub fn edge_pointer_to_json(tx: &Tx, schema: &Schema, edge: &EdgePointer) -> Result<Value> {
    let target = database_api::get_item_base(tx, edge.item)?;
    let target = target.ok_or_else(|| Error {
        code: StatusCode::INTERNAL_SERVER_ERROR,
        msg: format!("Edge connects to an nonexisting item.rowid {}", edge.item),
    })?;
    let target_json = Value::Object(item_base_to_json(tx, target, schema)?);
    let edge_item = database_api::get_item_base(tx, edge.rowid)?.ok_or_else(|| Error {
        code: StatusCode::INTERNAL_SERVER_ERROR,
        msg: format!("Edge does not have an item base, rowid: {}", edge.item),
    })?;
    let mut edge_item = item_base_to_json(tx, edge_item, schema)?;
    edge_item.insert("_edge".to_string(), serde_json::json!(edge.name));
    edge_item.insert("_item".to_string(), serde_json::json!(target_json));
    Ok(serde_json::json!(edge_item))
}
pub fn edge_pointers_to_json(
    tx: &Tx,
    schema: &Schema,
    half_edges: &[EdgePointer],
) -> Result<Vec<Value>> {
    let mut result = Vec::new();
    for edge in half_edges {
        result.push(edge_pointer_to_json(tx, schema, edge)?);
    }
    Ok(result)
}

pub fn get_edges(tx: &Tx, query: GetEdges, schema: &Schema) -> Result<Vec<Value>> {
    let root_item = database_api::get_item_rowid(tx, &query.item)?.ok_or_else(|| Error {
        code: StatusCode::NOT_FOUND,
        msg: format!("Cannot find item id {}", query.item),
    })?;
    let edges = if query.direction == EdgeDirection::Outgoing {
        database_api::get_outgoing_edges(tx, root_item)?
    } else {
        database_api::get_incoming_edges(tx, root_item)?
    };
    let mut result = Vec::new();
    for edge in edges {
        let base = database_api::get_item_base(tx, edge.item)?;
        let base = base.ok_or_else(|| Error {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: format!("Edge connects to an nonexisting item.rowid {}", edge.item),
        })?;
        if query.expand_items {
            let item_json = Value::Object(item_base_to_json(tx, base, schema)?);
            result.push(serde_json::json!({
                "name": edge.name,
                "item": item_json,
            }));
        } else {
            result.push(serde_json::json!({
                "name": edge.name,
                "item": { "id": base.id },
            }));
        }
    }
    Ok(result)
}

pub fn search(tx: &Tx, schema: &Schema, query: Search) -> Result<Vec<Value>> {
    info!("Searching by fields {:?}", query);
    if !query.other_properties.is_empty() {
        log::trace!(
            "Doing a slow search request using non-base properties {:?}",
            query.other_properties.keys()
        );
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
        _offset: query.offset,
    };
    let items = database_api::search_items(tx, &database_search)?;
    let mut result = Vec::new();
    for item in items {
        let rowid = item.rowid;
        if check_item_has_all_properties(tx, schema, rowid, &query.other_properties)? {
            let mut object_map = item_base_to_json(tx, item, schema)?;
            add_item_edge_properties(tx, &mut object_map, rowid)?;
            if query.forward_edges.is_some() {
                let edges = get_outgoing_edges(tx, rowid)?;
                let edges = edge_pointers_to_json(tx, schema, &edges)?;
                object_map.insert("[[edges]]".to_string(), Value::Array(edges));
            }
            if query.backward_edges.is_some() {
                let edges = get_incoming_edges(tx, rowid)?;
                let edges = edge_pointers_to_json(tx, schema, &edges)?;
                object_map.insert("~[[edges]]".to_string(), Value::Array(edges));
            }
            result.push(Value::Object(object_map));
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::api_model::CreateItem;
    use crate::command_line_interface;
    use crate::database_api;
    use crate::database_api::tests::new_conn;
    use crate::error::Result;
    use crate::internal_api;
    use crate::internal_api::*;
    use crate::plugin_auth_crypto::DatabaseKey;
    use crate::schema::Schema;
    use serde_json::json;
    use std::collections::HashMap;
    use warp::hyper::StatusCode;

    #[test]
    fn test_schema_checking() -> Result<()> {
        let mut conn = new_conn();
        let tx = conn.transaction().unwrap();
        let database_key = DatabaseKey::from("".to_string()).unwrap();
        let cli = command_line_interface::tests::test_cli();

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
        let cli = command_line_interface::tests::test_cli();
        let database_key = DatabaseKey::from("".to_string()).unwrap();

        let tx = conn.transaction().unwrap();
        let minimal_schema = database_api::get_schema(&tx).unwrap();

        let json = json!({
            "type": "ItemPropertySchema",
            "itemType": "Person",
            "propertyName": "age",
            "valueType": "Integer",
        });

        {
            // insert "age" successfully
            let create_item: CreateItem = serde_json::from_value(json.clone()).unwrap();
            internal_api::create_item_tx(
                &tx,
                &minimal_schema,
                create_item,
                "",
                &cli,
                &database_key,
            )
            .unwrap();
        }

        {
            let json = json!({
                "type": "ItemPropertySchema",
                "itemType": "Person",
                "propertyName": "dateCreated",
                "valueType": "Bool",
            });
            let create_item: CreateItem = serde_json::from_value(json).unwrap();
            let expected_error = "Schema for property dateCreated is already defined to type DateTime, cannot override to type Bool";
            assert!(internal_api::create_item_tx(
                &tx,
                &minimal_schema,
                create_item,
                "",
                &cli,
                &database_key
            )
            .unwrap_err()
            .msg
            .contains(expected_error));
        }

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

    #[test]
    fn test_edge_search() {
        let mut conn = new_conn();
        let cli = command_line_interface::tests::test_cli();
        let db_key = DatabaseKey::from("".to_string()).unwrap();
        let tx = conn.transaction().unwrap();
        let schema = database_api::get_schema(&tx).unwrap();

        let person1 = {
            let person = json!({
                "type": "Person",
            });
            let person: CreateItem = serde_json::from_value(person).unwrap();
            create_item_tx(&tx, &schema, person, "", &cli, &db_key).unwrap()
        };
        let person2 = {
            let person = json!({
                "type": "Person",
            });
            let person: CreateItem = serde_json::from_value(person).unwrap();
            create_item_tx(&tx, &schema, person, "", &cli, &db_key).unwrap()
        };
        let _edge = {
            let json = json!({
                "_source": person1,
                "_target": person2,
                "_name": "friend",
            });
            let parsed = serde_json::from_value(json).unwrap();
            create_edge(&tx, parsed).unwrap()
        };

        let result = {
            let json = json!({
                "id": person1,
                "[[edges]]": {},
            });
            let parsed = serde_json::from_value(json).unwrap();
            search(&tx, &schema, parsed).unwrap()
        };
        assert_eq!(result.len(), 1);
        let result = result.into_iter().next().unwrap();
        let result = match result {
            Value::Object(r) => r,
            _ => panic!("Result is not JSON Object"),
        };

        // {
        //   "[[edges]]": [
        //     {
        //       "_edge": "friend",
        //       "_item": {
        //         "dateCreated": 1624363823546,
        //         "dateModified": 1624363823546,
        //         "dateServerModified": 1624363823546,
        //         "deleted": false,
        //         "id": "1539da9dfc14a0467694e79730135df6",
        //         "type": "Person"
        //       },
        //       "dateCreated": 1624363823546,
        //       "dateModified": 1624363823546,
        //       "dateServerModified": 1624363823546,
        //       "deleted": false,
        //       "id": "4ea088d17a670cbb26f4154bce809950",
        //       "type": "Edge"
        //     }
        //   ],
        //   "dateCreated": 1624363823545,
        //   "dateModified": 1624363823545,
        //   "dateServerModified": 1624363823545,
        //   "deleted": false,
        //   "id": "59dd635884bff1de1c8447ac3763543b",
        //   "type": "Person"
        // }

        let result_id = result.get("id").unwrap().as_str().unwrap();
        assert_eq!(result_id, person1);

        let result_edge = result.get("[[edges]]").unwrap().as_array().unwrap();
        let result_edge = result_edge.iter().next().unwrap();

        assert_eq!(
            result_edge.get("_edge").unwrap().as_str().unwrap(),
            "friend"
        );

        let result_edge_item = result_edge.get("_item").unwrap();
        let result_edge_item = match result_edge_item {
            Value::Object(i) => i,
            _ => panic!("edge item is not an JSON Object"),
        };
        assert_eq!(
            result_edge_item.get("id").unwrap().as_str().unwrap(),
            person2
        );
    }
}
