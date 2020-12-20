use crate::api_model::BulkAction;
use crate::api_model::DeleteEdge;
use crate::api_model::InsertTreeItem;
use crate::api_model::SearchByFields;
use crate::error::Error;
use crate::error::Result;
use crate::sql_converters::borrow_sql_params;
use crate::sql_converters::json_value_to_sqlite;
use crate::sql_converters::sqlite_row_to_map;
use crate::sql_converters::sqlite_rows_to_json;
use crate::sql_converters::validate_property_name;
use chrono::Utc;
use log::debug;
use rusqlite::Connection;
use rusqlite::ToSql;
use rusqlite::Transaction;
use rusqlite::NO_PARAMS;
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::str;
use warp::http::status::StatusCode;

pub fn get_project_version() -> String {
    crate::command_line_interface::VERSION.to_string()
}

pub fn get_item(conn: &Connection, uid: &str) -> Result<Vec<Value>> {
    debug!("Getting item {}", uid);

    let mut stmt = conn.prepare_cached("SELECT * FROM items WHERE uid = :uid")?;
    let rows = stmt.query_named(&[(":uid", &uid)])?;

    let json = sqlite_rows_to_json(rows, true)?;
    Ok(json)
}

fn check_item_exists(tx: &Transaction, uid: &str) -> Result<bool> {
    let mut stmt = tx.prepare_cached("SELECT 1 FROM items WHERE uid = :uid")?;
    let mut rows = stmt.query_named(&[(":uid", &uid)])?;
    let result = match rows.next()? {
        None => false,
        Some(row) => {
            let count: isize = row.get(0)?;
            count > 0
        }
    };
    Ok(result)
}

pub fn get_all_items(conn: &Connection) -> Result<Vec<Value>> {
    debug!("Getting all items");
    let mut stmt = conn.prepare_cached("SELECT * FROM items")?;
    let rows = stmt.query(NO_PARAMS)?;
    let json = sqlite_rows_to_json(rows, true)?;
    Ok(json)
}

fn is_array_or_object(value: &Value) -> bool {
    match value {
        Value::Array(_) => true,
        Value::Object(_) => true,
        _ => false,
    }
}

fn write_sql_body(sql: &mut String, keys: &[&String], separator: &str) {
    let mut first = true;
    for key in keys {
        if !first {
            sql.push_str(separator);
        }
        sql.push_str(key);
        first = false;
    }
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

pub fn create_item_tx(tx: &Transaction, fields: HashMap<String, Value>) -> Result<String> {
    let mut fields: HashMap<String, Value> = fields
        .into_iter()
        .filter(|(k, v)| !is_array_or_object(v) && validate_property_name(k).is_ok())
        .collect();
    let time_now = Utc::now().timestamp_millis();
    if !fields.contains_key("dateCreated") {
        fields.insert("dateCreated".to_string(), time_now.into());
    }
    if !fields.contains_key("dateModified") {
        fields.insert("dateModified".to_string(), time_now.into());
    }
    fields.insert("_dateServerModified".to_string(), time_now.into());
    fields.insert("version".to_string(), Value::from(1));

    let mut sql = "INSERT INTO items (".to_string();
    let keys: Vec<_> = fields.keys().collect();
    write_sql_body(&mut sql, &keys, ", ");
    sql.push_str(") VALUES (:");
    write_sql_body(&mut sql, &keys, ", :");
    sql.push_str(");");
    execute_sql(tx, &sql, &fields)?;
    Ok(fields["uid"].to_string())
}

pub fn update_item_tx(tx: &Transaction, uid: &str, fields: HashMap<String, Value>) -> Result<()> {
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
    fields.insert("_dateServerModified".to_string(), time_now.into());

    fields.remove("version");
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
    tx: &Transaction,
    _type: &str,
    source: &str,
    target: &str,
    mut fields: HashMap<String, Value>,
) -> Result<()> {
    fields.insert("_type".to_string(), _type.into());
    fields.insert("_source".to_string(), source.into());
    fields.insert("_target".to_string(), target.into());
    let fields: HashMap<String, Value> = fields
        .into_iter()
        .filter(|(k, v)| !is_array_or_object(v) && validate_property_name(k).is_ok())
        .collect();
    let mut sql = "INSERT INTO edges (".to_string();
    let keys: Vec<_> = fields.keys().collect();
    write_sql_body(&mut sql, &keys, ", ");
    sql.push_str(") VALUES (:");
    write_sql_body(&mut sql, &keys, ", :");
    sql.push_str(");");
    if !check_item_exists(tx, source)? {
        return Err(Error {
            code: StatusCode::NOT_FOUND,
            msg: format!(
                "Failed to create edge {} {}->{} because source uid is not found",
                _type, source, target
            ),
        });
    };
    if !check_item_exists(tx, target)? {
        return Err(Error {
            code: StatusCode::NOT_FOUND,
            msg: format!(
                "Failed to create edge {} {}->{} because target uid is not found",
                _type, source, target
            ),
        });
    };
    execute_sql(tx, &sql, &fields)
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

pub fn delete_item_tx(tx: &Transaction, uid: &str) -> Result<()> {
    let mut fields = HashMap::new();
    let time_now = Utc::now().timestamp_millis();
    fields.insert("deleted".to_string(), true.into());
    fields.insert("dateModified".to_string(), time_now.into());
    fields.insert("_dateServerModified".to_string(), time_now.into());
    update_item_tx(tx, uid, fields)
}

pub fn bulk_action_tx(tx: &Transaction, bulk_action: BulkAction) -> Result<()> {
    debug!("Performing bulk action {:#?}", bulk_action);
    let edges_will_be_created = !bulk_action.create_edges.is_empty();
    for item in bulk_action.create_items {
        if !item.fields.contains_key("uid") && edges_will_be_created {
            return Err(Error {
                code: StatusCode::BAD_REQUEST,
                msg: format!("Creating items without uid in bulk_action is restricted for safety reasons (creating edges might be broken), item: {:?}", item.fields)
            });
        }
        create_item_tx(tx, item.fields)?;
    }
    for item in bulk_action.update_items {
        update_item_tx(tx, &item.uid, item.fields)?;
    }
    
    let sources_set: HashSet<String> = bulk_action.create_edges.iter().map(|e| e._source.clone()).collect();
    for edge in bulk_action.create_edges {
        create_edge(tx, &edge._type, &edge._source, &edge._target, edge.fields)?;
    }
    
    for edge_source in sources_set {
        update_item_tx(tx, &edge_source, HashMap::new())?;
    }
    for edge_uid in bulk_action.delete_items {
        delete_item_tx(tx, &edge_uid)?;
    }
    for del_edge in bulk_action.delete_edges {
        delete_edge_tx(tx, del_edge)?;
    }
    Ok(())
}

pub fn insert_tree(tx: &Transaction, item: InsertTreeItem, shared_server: bool) -> Result<String> {
    let source_uid: String = if item.fields.len() > 1 {
        create_item_tx(tx, item.fields)?
    } else if let Some(uid) = item.fields.get("uid").map(|v| v.to_string()) {
        if item._edges.is_empty() {
        } else if shared_server {
            return Err(Error {
                code: StatusCode::BAD_REQUEST,
                msg: format!(
                    "Cannot create edges from already existing items in a shared server {:?}",
                    item
                ),
            });
        } else {
            update_item_tx(tx, &uid, HashMap::new())?;
        }
        uid
    } else {
        return Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!("Cannot create item: {:?}", item),
        });
    };
    for edge in item._edges {
        let target_item = insert_tree(tx, edge._target, shared_server)?;
        create_edge(tx, &edge._type, &source_uid, &target_item, edge.fields)?;
    }
    Ok(source_uid)
}

pub fn search_by_fields(tx: &Transaction, query: SearchByFields) -> Result<Vec<Value>> {
    debug!("Searching by fields {:?}", query);
    let mut sql_body = "SELECT * FROM items WHERE ".to_string();
    let mut first_parameter = true;
    for (field, value) in &query.fields {
        validate_property_name(field)?;
        match value {
            Value::Array(_) => continue,
            Value::Object(_) => continue,
            _ => (),
        };
        if !first_parameter {
            sql_body.push_str(" AND ")
        };
        first_parameter = false;
        sql_body.push_str(field);
        sql_body.push_str(" = :");
        sql_body.push_str(field);
    }
    if query._date_server_modified_after.is_some() {
        sql_body.push_str(" AND _dateServerModified > :_dateServerModified");
    };
    sql_body.push_str(";");

    let mut sql_params = Vec::new();
    for (key, value) in &query.fields {
        sql_params.push((format!(":{}", key), json_value_to_sqlite(&value, &key)?));
    }
    if let Some(date) = query._date_server_modified_after {
        let key = ":_dateServerModified".to_string();
        sql_params.push((key, date.into()));
    };
    let sql_params = borrow_sql_params(sql_params.as_slice());
    let mut stmt = tx.prepare_cached(&sql_body)?;
    let rows = stmt.query_named(sql_params.as_slice())?;
    let json = sqlite_rows_to_json(rows, true)?;
    Ok(json)
}

pub fn get_item_with_edges_tx(tx: &Transaction, uid: &str) -> Result<Value> {
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
            .get("_target").map(|v| v.to_string())
            .expect("Failed to get _target");
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

pub fn get_items_with_edges_tx(tx: &Transaction, uids: &[String]) -> Result<Vec<Value>> {
    let mut result = Vec::new();
    for uid in uids {
        result.push(get_item_with_edges_tx(tx, uid)?);
    }
    Ok(result)
}
