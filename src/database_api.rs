use crate::error::Error;
use crate::error::ErrorContext;
use crate::error::Result;
use crate::schema::Schema;
use crate::schema::SchemaPropertyType;
use log::debug;
use rusqlite::params;
use rusqlite::types::ToSqlOutput;
use rusqlite::Transaction as Tx;
use rusqlite::NO_PARAMS;
use std::collections::HashMap;
use warp::http::StatusCode;

pub type Rowid = i64;
pub type DBTime = i64;

pub struct ItemBase {
    pub rowid: Rowid,
    pub id: String,
    pub _type: String,
    pub date_created: DBTime,
    pub date_modified: DBTime,
    pub date_server_modified: DBTime,
    pub deleted: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn insert_item_base(
    tx: &Tx,
    id: &str,
    _type: &str,
    date_created_millis: DBTime,
    date_modified_millis: DBTime,
    date_server_modified_millis: DBTime,
    deleted: bool,
) -> Result<Rowid> {
    let mut stmt = tx
        .prepare_cached(
            "INSERT INTO items (\
            id, \
            type, \
            dateCreated, \
            dateModified, \
            dateServerModified, \
            deleted \
        ) VALUES (?, ?, ?, ?, ?, ?);",
        )
        .context_str("Failed to prepare/compile INSERT statement")?;
    stmt.insert(params![
        id,
        _type,
        date_created_millis,
        date_modified_millis,
        date_server_modified_millis,
        deleted,
    ])
    .context_str("Failed to execute insert_item with parameters")
}

pub fn get_item_rowid(tx: &Tx, id: &str) -> Result<Option<Rowid>> {
    let mut stmt = tx.prepare_cached("SELECT rowid FROM items WHERE id = ?;")?;
    let mut rows = stmt.query_map(params![id], |row| row.get(0))?;
    if let Some(row) = rows.next() {
        let rowid: i64 = row?;
        Ok(Some(rowid))
    } else {
        Ok(None)
    }
}

pub fn search_items(
    tx: &Tx,
    rowid: Option<Rowid>,
    id: Option<&str>,
    _type: Option<&str>,
    date_server_modified_gte: Option<DBTime>,
    date_server_modified_lt: Option<DBTime>,
    deleted: Option<bool>,
) -> Result<Vec<ItemBase>> {
    let mut sql_query = "\
        SELECT \
            rowid, \
            id, \
            type, \
            dateCreated, \
            dateModified, \
            dateServerModified, \
            deleted \
        FROM \
            items \
        WHERE "
        .to_string();
    let mut params_vec: Vec<ToSqlOutput> = Vec::new();
    if let Some(r) = rowid {
        add_sql_param(&mut sql_query, "rowid", &Comparison::Equals);
        params_vec.push(r.into());
    }
    if let Some(id) = id {
        add_sql_param(&mut sql_query, "id", &Comparison::Equals);
        params_vec.push(id.into());
    }
    if let Some(typ) = _type {
        add_sql_param(&mut sql_query, "type", &Comparison::Equals);
        params_vec.push(typ.into());
    }
    if let Some(dt) = date_server_modified_gte {
        add_sql_param(
            &mut sql_query,
            "dateServerModified",
            &Comparison::GreaterOrEquals,
        );
        params_vec.push(dt.into());
    }
    if let Some(dt) = date_server_modified_lt {
        add_sql_param(&mut sql_query, "dateServerModified", &Comparison::LessThan);
        params_vec.push(dt.into());
    }
    if let Some(deleted) = deleted {
        add_sql_param(&mut sql_query, "deleted", &Comparison::Equals);
        params_vec.push(deleted.into());
    }
    sql_query.push_str("1 ;"); // older sqlite versions do not support `true`
    debug!("Executing search SQL: {}", sql_query);

    let mut stmt = tx
        .prepare_cached(&sql_query)
        .context(|| format!("SQL query: {}", sql_query))?;
    let mut rows = stmt.query(params_vec)?;
    let mut result = Vec::new();
    while let Some(row) = rows.next()? {
        result.push(ItemBase {
            rowid: row.get(0)?,
            id: row.get(1)?,
            _type: row.get(2)?,
            date_created: row.get(3)?,
            date_modified: row.get(4)?,
            date_server_modified: row.get(5)?,
            deleted: row.get(6)?,
        });
    }
    Ok(result)
}

pub fn dangerous_permament_remove_item(tx: &Tx, rowid: Rowid) -> Result<()> {
    let mut stmt = tx.prepare_cached("DELETE FROM integers WHERE item = ?;")?;
    stmt.execute(params![rowid])?;
    let mut stmt = tx.prepare_cached("DELETE FROM reals WHERE item = ?;")?;
    stmt.execute(params![rowid])?;
    let mut stmt = tx.prepare_cached("DELETE FROM strings WHERE item = ?;")?;
    stmt.execute(params![rowid])?;
    let mut stmt = tx.prepare_cached("DELETE FROM items WHERE rowid = ?;")?;
    stmt.execute(params![rowid])?;
    Ok(())
}

pub fn insert_integer(tx: &Tx, item: Rowid, name: &str, value: i64) -> Result<()> {
    let mut stmt = tx.prepare_cached("INSERT INTO integers VALUES(?, ?, ?);")?;
    stmt.execute(params![item, name, value])?;
    Ok(())
}

pub fn insert_real(tx: &Tx, item: Rowid, name: &str, value: f64) -> Result<()> {
    let mut stmt = tx.prepare_cached("INSERT INTO reals VALUES(?, ?, ?);")?;
    stmt.execute(params![item, name, value])?;
    Ok(())
}

pub fn insert_string(tx: &Tx, item: Rowid, name: &str, value: &str) -> Result<()> {
    let mut stmt = tx.prepare_cached("INSERT INTO strings VALUES(?, ?, ?);")?;
    stmt.execute(params![item, name, value])?;
    Ok(())
}

pub fn delete_property(tx: &Tx, item: Rowid, name: &str) -> Result<()> {
    let mut stmt = tx.prepare_cached("DELETE FROM integers WHERE item = ? AND name = ?;")?;
    stmt.execute(params![item, name])?;
    let mut stmt = tx.prepare_cached("DELETE FROM strings WHERE item = ? AND name = ?;")?;
    stmt.execute(params![item, name])?;
    let mut stmt = tx.prepare_cached("DELETE FROM reals WHERE item = ? AND name = ?;")?;
    stmt.execute(params![item, name])?;
    Ok(())
}

/// Low-level function to insert an edge.
/// No Schema/type checks are done. Use other functions around instead.
#[allow(dead_code)]
fn insert_edge_unchecked(
    tx: &Tx,
    source: Rowid,
    name: &str,
    target: Rowid,
    id: &str,
    date: DBTime,
) -> Result<Rowid> {
    let item = insert_item_base(tx, id, name, date, date, date, false)?;
    let mut stmt =
        tx.prepare_cached("INSERT INTO edges(self, source, name, target) VALUES(?, ?, ?, ?);")?;
    stmt.execute(params![item, source, name, target])?;
    Ok(item)
}

pub fn get_schema(tx: &Tx) -> Result<Schema> {
    let mut stmt = tx
        .prepare_cached(
            "SELECT thisProperty.value, thisType.value \
        FROM \
            items as item, \
            strings as thisProperty, \
            strings as thisType \
        WHERE item.type = 'ItemPropertySchema' \
        AND thisProperty.item = item.rowid \
        AND thisType.item = item.rowid \
        AND thisProperty.name = 'propertyName' \
        AND thisType.name = 'valueType';",
        )
        .context_str("Failed to prepare SQL get_schema query")?;
    let mut rows = stmt.query(NO_PARAMS)?;
    let mut property_types: HashMap<String, SchemaPropertyType> = HashMap::new();
    while let Some(row) = rows.next()? {
        let this_property: String = row.get(0)?;
        let this_type: String = row.get(1)?;
        let value_type = SchemaPropertyType::from_string(&this_type).map_err(|e| Error {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: e,
        })?;
        property_types.insert(this_property, value_type);
    }
    Ok(Schema { property_types })
}

pub fn delete_schema_items_by_item_type_and_prop(
    tx: &Tx,
    item_type: &str,
    property_name: &str,
) -> Result<()> {
    let sql = "SELECT rowid FROM items as item, strings as itemTypeStr, strings as propNameStr \
        WHERE item.type = 'ItemPropertySchema' \
        AND item.rowid = itemTypeStr.item \
        AND item.rowid = propNameStr.item \
        AND itemTypeStr.name = 'itemType' \
        AND itemTypeStr.value = ? \
        AND propNameStr.name = 'propertyName' \
        AND propNameStr.value = ? \
        ;";
    let mut stmt = tx.prepare_cached(sql)?;
    let mut rows = stmt.query(params![item_type, property_name])?;
    while let Some(row) = rows.next()? {
        let rowid: Rowid = row.get(0)?;
        dangerous_permament_remove_item(tx, rowid)?;
    }
    Ok(())
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Comparison {
    Equals,
    GreaterThan,
    GreaterOrEquals,
    LessThan,
    LessOrEquals,
}

fn add_sql_param(query: &mut String, column: &str, operation: &Comparison) {
    query.push_str(column);
    match operation {
        Comparison::Equals => query.push_str(" = "),
        Comparison::GreaterThan => query.push_str(" > "),
        Comparison::GreaterOrEquals => query.push_str(" >= "),
        Comparison::LessThan => query.push_str(" < "),
        Comparison::LessOrEquals => query.push_str(" <= "),
    };
    query.push_str("? AND ");
}

#[cfg(test)]
mod tests {
    use super::super::database_migrate_refinery;
    use super::super::error::Result;
    use super::*;
    use chrono::Utc;
    use rand::Rng;
    use rusqlite::Connection;

    fn new_conn() -> Connection {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        database_migrate_refinery::embedded::migrations::runner()
            .run(&mut conn)
            .expect("Failed to run refinery migrations");
        conn
    }

    fn random_id() -> String {
        rand::thread_rng().gen::<i64>().to_string()
    }

    #[test]
    fn test_insert_item() -> Result<()> {
        let mut conn = new_conn();
        let tx = conn.transaction()?;
        let date = Utc::now().timestamp_millis();
        let item1 = insert_item_base(&tx, &random_id(), "Person", date, date, date, false)?;
        let item2 = insert_item_base(&tx, &random_id(), "Book", date, date, date, false)?;
        assert_eq!(item2 - item1, 1);
        Ok(())
    }

    #[test]
    fn test_insert_properties() -> Result<()> {
        let mut conn = new_conn();
        let tx = conn.transaction()?;
        let date = Utc::now().timestamp_millis();
        let item = insert_item_base(&tx, &random_id(), "Person", date, date, date, false)?;
        insert_integer(&tx, item, "age", 20)?;
        insert_real(&tx, item, "attack", 13.5)?;
        insert_string(&tx, item, "trait", "resilient")?;
        Ok(())
    }

    #[test]
    fn test_insert_edge() -> Result<()> {
        let mut conn = new_conn();
        let tx = conn.transaction()?;
        let date = Utc::now().timestamp_millis();
        let source = insert_item_base(&tx, &random_id(), "Person", date, date, date, false)?;
        let target = insert_item_base(&tx, &random_id(), "Person", date, date, date, false)?;
        assert_eq!(target - source, 1);
        let edge = insert_edge_unchecked(&tx, source, "friend", target, &random_id(), date)?;
        assert_eq!(edge - target, 1);
        Ok(())
    }

    #[test]
    fn test_default_schema() -> Result<()> {
        let mut conn = new_conn();
        let tx = conn.transaction()?;
        let schema = get_schema(&tx)?;
        assert_eq!(schema.property_types.len(), 3);
        Ok(())
    }

    #[test]
    fn test_full_schema() -> Result<()> {
        let mut conn = new_conn();
        let tx = conn.transaction()?;
        let date = Utc::now().timestamp_millis();
        let item = insert_item_base(
            &tx,
            &random_id(),
            "ItemPropertySchema",
            date,
            date,
            date,
            false,
        )?;
        insert_string(&tx, item, "itemType", "Person")?;
        insert_string(&tx, item, "propertyName", "age")?;
        insert_string(&tx, item, "valueType", "integer")?;

        let item = insert_item_base(
            &tx,
            &random_id(),
            "ItemPropertySchema",
            date,
            date,
            date,
            false,
        )?;
        insert_string(&tx, item, "itemType", "Person")?;
        insert_string(&tx, item, "propertyName", "name")?;
        insert_string(&tx, item, "valueType", "text")?;

        let schema = get_schema(&tx)?;
        assert_eq!(
            schema.property_types.get("age"),
            Some(&SchemaPropertyType::Integer)
        );
        assert_eq!(
            schema.property_types.get("name"),
            Some(&SchemaPropertyType::Text)
        );
        assert_eq!(schema.property_types.len(), 5);
        Ok(())
    }

    #[test]
    fn test_delete_schema_items() -> Result<()> {
        let mut conn = new_conn();
        let tx = conn.transaction()?;
        let date = Utc::now().timestamp_millis();
        let item: Rowid = insert_item_base(
            &tx,
            &random_id(),
            "ItemPropertySchema",
            date,
            date,
            date,
            false,
        )?;
        insert_string(&tx, item, "itemType", "Person")?;
        insert_string(&tx, item, "propertyName", "age")?;
        insert_string(&tx, item, "valueType", "integer")?;

        assert_eq!(
            search_items(&tx, Some(item), None, None, None, None, None)?.len(),
            1
        );
        dangerous_permament_remove_item(&tx, item)?;
        assert_eq!(
            search_items(&tx, Some(item), None, None, None, None, None)?.len(),
            0
        );

        Ok(())
    }

    #[test]
    fn test_search() -> Result<()> {
        let mut conn = new_conn();
        let tx = conn.transaction()?;
        let date = Utc::now().timestamp_millis();
        let item1 = insert_item_base(&tx, "one", "Person", date, date, date, false)?;
        let _item2 = insert_item_base(&tx, "two", "Book", date, date, date, false)?;
        let _item3 = insert_item_base(&tx, "three", "Street", date, date, date, false)?;
        assert_eq!(
            search_items(&tx, None, None, Some("Person"), None, None, None)?.len(),
            1,
        );
        assert_eq!(
            search_items(&tx, None, None, Some("Void"), None, None, None)?.len(),
            0,
        );
        assert_eq!(
            search_items(&tx, Some(item1), None, None, None, None, None)?.len(),
            1,
        );
        assert_eq!(
            search_items(&tx, None, Some("one"), None, None, None, None)?.len(),
            1,
        );
        assert_eq!(
            search_items(&tx, None, Some("nothing"), None, None, None, None)?.len(),
            0,
        );
        assert_eq!(
            search_items(&tx, None, None, None, Some(date), None, None)?.len(),
            3,
        );
        assert_eq!(
            search_items(&tx, None, None, None, Some(date), Some(date + 1), None)?.len(),
            3,
        );
        assert_eq!(
            search_items(&tx, None, None, None, Some(date - 1), Some(date), None)?.len(),
            0,
        );
        assert_eq!(
            search_items(&tx, None, None, None, None, None, Some(true))?.len(),
            0,
        );
        assert_eq!(
            search_items(&tx, None, None, None, Some(date - 1), None, Some(false))?.len(),
            3,
        );
        assert!(search_items(&tx, None, None, None, None, None, None)?.len() >= 3);
        assert_eq!(
            search_items(
                &tx,
                Some(item1),
                Some("one"),
                Some("Person"),
                Some(date),
                Some(date + 1),
                Some(false)
            )?
            .len(),
            1,
        );
        Ok(())
    }
}
