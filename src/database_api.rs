use crate::api_model::SortOrder;
use crate::error::Error;
use crate::error::ErrorContext;
use crate::error::Result;
use crate::schema::Schema;
use crate::schema::SchemaPropertyType;
use field_count::FieldCount;
use log::debug;
use rusqlite::params;
use rusqlite::types::ToSqlOutput;
use rusqlite::Row;
use rusqlite::Rows;
use rusqlite::Transaction as Tx;
use std::collections::HashMap;
use warp::http::StatusCode;

pub type Rowid = i64;
pub type DbTime = i64;

#[derive(FieldCount)]
pub struct ItemBase {
    pub rowid: Rowid,
    pub id: String,
    pub _type: String,
    pub date_created: DbTime,
    pub date_modified: DbTime,
    pub date_server_modified: DbTime,
    pub deleted: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn insert_item_base(
    tx: &Tx,
    id: &str,
    _type: &str,
    date_created_millis: DbTime,
    date_modified_millis: DbTime,
    date_server_modified_millis: DbTime,
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

pub fn get_item_base(tx: &Tx, rowid: Rowid) -> Result<Option<ItemBase>> {
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
    let item = search_items(tx, &database_search)?.into_iter().next();
    Ok(item)
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

pub struct DatabaseSearch<'a> {
    pub rowid: Option<Rowid>,
    pub id: Option<&'a str>,
    pub _type: Option<&'a str>,
    pub date_server_modified_gte: Option<DbTime>,
    pub date_server_modified_lt: Option<DbTime>,
    pub deleted: Option<bool>,
    pub sort_order: SortOrder,
    pub _limit: u64,
}

fn parse_item_base(row: &Row) -> Result<ItemBase> {
    Ok(ItemBase {
        rowid: row.get(0)?,
        id: row.get(1)?,
        _type: row.get(2)?,
        date_created: row.get(3)?,
        date_modified: row.get(4)?,
        date_server_modified: row.get(5)?,
        deleted: row.get(6)?,
    })
}

#[allow(clippy::comparison_chain)]
pub fn search_items(tx: &Tx, query: &DatabaseSearch) -> Result<Vec<ItemBase>> {
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
    if let Some(r) = query.rowid {
        add_sql_param(&mut sql_query, "rowid", &Comparison::Equals);
        params_vec.push(r.into());
    }
    if let Some(id) = &query.id {
        add_sql_param(&mut sql_query, "id", &Comparison::Equals);
        params_vec.push((*id).into());
    }
    if let Some(typ) = &query._type {
        add_sql_param(&mut sql_query, "type", &Comparison::Equals);
        params_vec.push((*typ).into());
    }
    if let Some(dt) = query.date_server_modified_gte {
        add_sql_param(
            &mut sql_query,
            "dateServerModified",
            &Comparison::GreaterOrEquals,
        );
        params_vec.push(dt.into());
    }
    if let Some(dt) = query.date_server_modified_lt {
        add_sql_param(&mut sql_query, "dateServerModified", &Comparison::LessThan);
        params_vec.push(dt.into());
    }
    if let Some(deleted) = query.deleted {
        add_sql_param(&mut sql_query, "deleted", &Comparison::Equals);
        params_vec.push(deleted.into());
    }
    sql_query.push_str("1 "); // older sqlite versions do not support `true`
    sql_query.push_str(&format!("ORDER BY dateServerModified {}", query.sort_order));
    sql_query.push(';');
    debug!("Executing search SQL: {}", sql_query);

    let mut stmt = tx
        .prepare_cached(&sql_query)
        .context(|| format!("SQL query: {}", sql_query))?;

    for (index, param) in params_vec.into_iter().enumerate() {
        // SQLite parameters are 1-based, not 0-based, so we need to add 1 to the index.
        stmt.raw_bind_parameter(index + 1, param)?;
    }
    let mut rows: Rows = stmt.raw_query();

    let mut result = Vec::new();
    let mut num_left = query._limit;
    let mut last_date: Option<DbTime> = None;
    while let Some(row) = rows.next()? {
        let item = parse_item_base(row)?;
        if num_left > 1 {
            num_left -= 1;
        } else if num_left == 1 {
            num_left = 0;
            last_date = Some(item.date_server_modified);
        } else if let Some(last_date) = last_date {
            if last_date != item.date_server_modified {
                break;
            }
        } else {
            break;
        }
        result.push(item);
    }
    Ok(result)
}

/// Search for items that have a certain property equal to certain value
pub fn search_strings(tx: &Tx, property_name: &str, value: &str) -> Result<Vec<Rowid>> {
    let mut stmt = tx.prepare_cached("SELECT item FROM strings WHERE name = ? AND value = ?;")?;
    let mut rows = stmt.query(params![property_name, value])?;
    let mut result = Vec::new();
    while let Some(row) = rows.next()? {
        let item: Rowid = row.get(0)?;
        result.push(item);
    }
    Ok(result)
}

pub fn get_strings_for_item(tx: &Tx, item_rowid: Rowid) -> Result<HashMap<String, String>> {
    let mut stmt = tx.prepare_cached("SELECT name, value FROM strings WHERE item = ?;")?;
    let mut rows = stmt.query(params![item_rowid])?;
    let mut result = HashMap::new();
    while let Some(row) = rows.next()? {
        result.insert(row.get(0)?, row.get(1)?);
    }
    Ok(result)
}
pub struct StringsNameValue {
    pub name: String,
    pub value: String,
}
pub fn get_strings_records_for_item(tx: &Tx, item_rowid: Rowid) -> Result<Vec<StringsNameValue>> {
    let mut stmt = tx.prepare_cached("SELECT name, value FROM strings WHERE item = ?;")?;
    let mut rows = stmt.query(params![item_rowid])?;
    let mut result = Vec::new();
    while let Some(row) = rows.next()? {
        result.push(StringsNameValue {
            name: row.get(0)?,
            value: row.get(1)?,
        });
    }
    Ok(result)
}

pub fn get_integers_for_item(tx: &Tx, item_rowid: Rowid) -> Result<HashMap<String, i64>> {
    let mut stmt = tx.prepare_cached("SELECT name, value FROM integers WHERE item = ?;")?;
    let mut rows = stmt.query(params![item_rowid])?;
    let mut result = HashMap::new();
    while let Some(row) = rows.next()? {
        result.insert(row.get(0)?, row.get(1)?);
    }
    Ok(result)
}
pub struct IntegersNameValue {
    pub name: String,
    pub value: i64,
}
pub fn get_integers_records_for_item(tx: &Tx, item_rowid: Rowid) -> Result<Vec<IntegersNameValue>> {
    let mut stmt = tx.prepare_cached("SELECT name, value FROM integers WHERE item = ?;")?;
    let mut rows = stmt.query(params![item_rowid])?;
    let mut result = Vec::new();
    while let Some(row) = rows.next()? {
        result.push(IntegersNameValue {
            name: row.get(0)?,
            value: row.get(1)?,
        });
    }
    Ok(result)
}

pub fn get_reals_for_item(tx: &Tx, item_rowid: Rowid) -> Result<HashMap<String, f64>> {
    let mut stmt = tx.prepare_cached("SELECT name, value FROM reals WHERE item = ?;")?;
    let mut rows = stmt.query(params![item_rowid])?;
    let mut result = HashMap::new();
    while let Some(row) = rows.next()? {
        result.insert(row.get(0)?, row.get(1)?);
    }
    Ok(result)
}
pub struct RealsNameValue {
    pub name: String,
    pub value: f64,
}
pub fn get_reals_records_for_item(tx: &Tx, item_rowid: Rowid) -> Result<Vec<RealsNameValue>> {
    let mut stmt = tx.prepare_cached("SELECT name, value FROM reals WHERE item = ?;")?;
    let mut rows = stmt.query(params![item_rowid])?;
    let mut result = Vec::new();
    while let Some(row) = rows.next()? {
        result.push(RealsNameValue {
            name: row.get(0)?,
            value: row.get(1)?,
        });
    }
    Ok(result)
}

pub fn check_integer_exists(tx: &Tx, item_rowid: Rowid, name: &str, value: i64) -> Result<bool> {
    let mut stmt =
        tx.prepare_cached("SELECT 1 FROM integers WHERE item = ? AND name = ? AND value = ? ;")?;
    Ok(stmt.exists(params![item_rowid, name, value])?)
}

pub fn check_string_exists(tx: &Tx, item_rowid: Rowid, name: &str, value: &str) -> Result<bool> {
    let mut stmt =
        tx.prepare_cached("SELECT 1 FROM strings WHERE item = ? AND name = ? AND value = ? ;")?;
    Ok(stmt.exists(params![item_rowid, name, value])?)
}

pub fn check_real_exists(tx: &Tx, item_rowid: Rowid, name: &str, value: f64) -> Result<bool> {
    let mut stmt =
        tx.prepare_cached("SELECT 1 FROM reals WHERE item = ? AND name = ? AND value = ? ;")?;
    Ok(stmt.exists(params![item_rowid, name, value])?)
}

pub fn update_item_date_server_modified(tx: &Tx, rowid: Rowid, date: DbTime) -> Result<()> {
    let sql = "UPDATE items SET dateServerModified = ? WHERE rowid = ?;";
    let mut stmt = tx.prepare_cached(sql)?;
    stmt.execute(params![date, rowid])?;
    Ok(())
}

pub fn update_item_base(
    tx: &Tx,
    rowid: Rowid,
    date_modified: DbTime,
    date_server_modified: DbTime,
    deleted: Option<bool>,
) -> Result<()> {
    let mut sql = "UPDATE items SET dateModified = ?, dateServerModified = ?".to_string();
    if deleted.is_some() {
        sql.push_str(", deleted = ?");
    }
    sql.push_str(" WHERE rowid = ?;");
    let mut stmt = tx.prepare_cached(&sql)?;
    if let Some(deleted) = deleted {
        stmt.execute(params![date_modified, date_server_modified, deleted, rowid])?;
        Ok(())
    } else {
        stmt.execute(params![date_modified, date_server_modified, rowid])?;
        Ok(())
    }
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

pub fn insert_edge(
    tx: &Tx,
    self_rowid: Rowid,
    source: Rowid,
    name: &str,
    target: Rowid,
) -> Result<Rowid> {
    let mut stmt =
        tx.prepare_cached("INSERT INTO edges(self, source, name, target) VALUES(?, ?, ?, ?);")?;
    stmt.execute(params![self_rowid, source, name, target])?;
    Ok(tx.last_insert_rowid())
}

pub struct EdgeBase {
    pub rowid: Rowid,
    pub name: String,
    pub source: Rowid,
    pub target: Rowid,
}

/// Edge pointer, either for outgoing edge or for incoming edge.
/// `item` is essentially either `source` or `target`, depending on the direction.
pub struct EdgePointer {
    pub rowid: Rowid,
    pub name: String,
    pub item: Rowid,
}

pub fn get_self_edge(tx: &Tx, self_rowid: Rowid) -> Result<Option<EdgeBase>> {
    let mut stmt = tx.prepare_cached("SELECT source, name, target FROM edges WHERE self = ?;")?;
    let mut rows = stmt.query(params![self_rowid])?;
    if let Some(row) = rows.next()? {
        Ok(Some(EdgeBase {
            rowid: self_rowid,
            source: row.get(0)?,
            name: row.get(1)?,
            target: row.get(2)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn get_outgoing_edges(tx: &Tx, source: Rowid) -> Result<Vec<EdgePointer>> {
    let mut stmt = tx.prepare_cached("SELECT self, target, name FROM edges WHERE source = ?;")?;
    let mut rows = stmt.query(params![source])?;
    let mut result = Vec::new();
    while let Some(row) = rows.next()? {
        let rowid = row.get(0)?;
        let item = row.get(1)?;
        let name = row.get(2)?;
        result.push(EdgePointer { rowid, name, item })
    }
    Ok(result)
}

pub fn get_incoming_edges(tx: &Tx, target: Rowid) -> Result<Vec<EdgePointer>> {
    let mut stmt = tx.prepare_cached("SELECT self, source, name FROM edges WHERE target = ?;")?;
    let mut rows = stmt.query(params![target])?;
    let mut result = Vec::new();
    while let Some(row) = rows.next()? {
        let rowid = row.get(0)?;
        let item = row.get(1)?;
        let name = row.get(2)?;
        result.push(EdgePointer { rowid, name, item })
    }
    Ok(result)
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
    let mut rows = stmt.query([])?;
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
pub mod tests {
    use super::super::database_migrate_refinery;
    use super::super::error::Result;
    use super::*;
    use chrono::Utc;
    use rusqlite::Connection;
    use std::ops::Not;

    pub fn new_conn() -> Connection {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        database_migrate_refinery::embedded::migrations::runner()
            .run(&mut conn)
            .expect("Failed to run refinery migrations");
        conn
    }

    pub fn random_id() -> String {
        rand::random::<i64>().to_string()
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
        let item = insert_item_base(&tx, &random_id(), "Edge", date, date, date, false)?;
        let edge = insert_edge(&tx, item, source, "friend", target)?;
        assert_eq!(edge - target, 1);
        Ok(())
    }

    #[test]
    fn test_default_schema() -> Result<()> {
        let mut conn = new_conn();
        let tx = conn.transaction()?;
        let schema = get_schema(&tx)?;
        assert!(schema.property_types.len() >= 3);
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
        insert_string(&tx, item, "valueType", "Integer")?;

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
        insert_string(&tx, item, "valueType", "Text")?;

        let schema = get_schema(&tx)?;
        assert_eq!(
            schema.property_types.get("age"),
            Some(&SchemaPropertyType::Integer)
        );
        assert_eq!(
            schema.property_types.get("name"),
            Some(&SchemaPropertyType::Text)
        );
        assert!(schema.property_types.len() >= 3);
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
        insert_string(&tx, item, "valueType", "Integer")?;

        assert_eq!(
            search_items_params(&tx, Some(item), None, None, None, None, None)?.len(),
            1
        );
        dangerous_permament_remove_item(&tx, item)?;
        assert_eq!(
            search_items_params(&tx, Some(item), None, None, None, None, None)?.len(),
            0
        );

        Ok(())
    }

    fn search_items_params(
        tx: &Tx,
        rowid: Option<Rowid>,
        id: Option<&str>,
        _type: Option<&str>,
        date_server_modified_gte: Option<DbTime>,
        date_server_modified_lt: Option<DbTime>,
        deleted: Option<bool>,
    ) -> Result<Vec<ItemBase>> {
        let database_search = DatabaseSearch {
            rowid,
            id,
            _type,
            date_server_modified_gte,
            date_server_modified_lt,
            deleted,
            sort_order: SortOrder::Asc,
            _limit: u64::MAX,
        };
        search_items(tx, &database_search)
    }

    #[test]
    fn test_search() -> Result<()> {
        let mut conn = new_conn();
        let tx = conn.transaction()?;
        let date = Utc::now().timestamp_millis();
        let item1 = insert_item_base(&tx, "one", "Person", date, date, date, false)?;
        let _item2 = insert_item_base(&tx, "two", "Book", date, date, date + 1, false)?;
        let _item3 = insert_item_base(&tx, "three", "Street", date, date, date + 2, false)?;
        assert_eq!(
            search_items_params(&tx, None, None, Some("Person"), None, None, None)?.len(),
            1,
        );
        assert_eq!(
            search_items_params(&tx, None, None, Some("Void"), None, None, None)?.len(),
            0,
        );
        assert_eq!(
            search_items_params(&tx, Some(item1), None, None, None, None, None)?.len(),
            1,
        );
        assert_eq!(
            search_items_params(&tx, None, Some("one"), None, None, None, None)?.len(),
            1,
        );
        assert_eq!(
            search_items_params(&tx, None, Some("nothing"), None, None, None, None)?.len(),
            0,
        );
        assert_eq!(
            search_items_params(&tx, None, None, None, Some(date), None, None)?.len(),
            3,
        );
        assert_eq!(
            search_items_params(&tx, None, None, None, Some(date), Some(date + 3), None)?.len(),
            3,
        );
        assert_eq!(
            search_items_params(&tx, None, None, None, Some(date - 1), Some(date), None)?.len(),
            0,
        );
        assert_eq!(
            search_items_params(&tx, None, None, None, None, None, Some(true))?.len(),
            0,
        );
        assert_eq!(
            search_items_params(&tx, None, None, None, Some(date - 1), None, Some(false))?.len(),
            3,
        );
        assert!(search_items_params(&tx, None, None, None, None, None, None)?.len() >= 3);
        {
            let mut query = DatabaseSearch {
                rowid: None,
                id: None,
                _type: None,
                date_server_modified_gte: Some(date),
                date_server_modified_lt: None,
                deleted: None,
                sort_order: SortOrder::Asc,
                _limit: 1,
            };
            assert_eq!(search_items(&tx, &query)?.len(), 1);

            query._limit = 3;
            let three_results = search_items(&tx, &query)?;
            assert_eq!(three_results.len(), 3);

            let first = &three_results[0];
            let third = &three_results[2];
            assert_ne!(first.rowid, third.rowid);
            assert_ne!(first.date_server_modified, third.date_server_modified);

            query.sort_order = SortOrder::Asc;
            let search_asc = &search_items(&tx, &query)?;
            query.sort_order = SortOrder::Desc;
            let search_desc = &search_items(&tx, &query)?;
            assert_eq!(search_asc[0].rowid, search_desc[2].rowid);
            assert_eq!(search_asc[2].rowid, search_desc[0].rowid);

            query._limit = 100;
            assert!(search_items(&tx, &query)?.len() < 100); // there aren't 100 items there
        }
        assert_eq!(
            search_items_params(
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

    #[test]
    fn test_limiting() -> Result<()> {
        let mut conn = new_conn();
        let tx = conn.transaction()?;
        let date = Utc::now().timestamp_millis();
        let _item1 = insert_item_base(&tx, "one", "Person", date, date, date, false)?;
        let _item2 = insert_item_base(&tx, "two", "Book", date, date, date, false)?;
        let _item3 = insert_item_base(&tx, "three", "Street", date, date, date + 1, false)?;
        let query = DatabaseSearch {
            rowid: None,
            id: None,
            _type: None,
            date_server_modified_gte: Some(date),
            date_server_modified_lt: None,
            deleted: None,
            sort_order: SortOrder::Asc,
            _limit: 1,
        };
        // 1 main item + 1 other item with identical `dateServerModified`
        assert_eq!(search_items(&tx, &query)?.len(), 2);
        Ok(())
    }

    #[test]
    fn test_property_checks() -> Result<()> {
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
        insert_string(&tx, item, "valueType", "Integer")?;

        assert!(check_string_exists(&tx, item, "itemType", "Person")?);
        assert!(check_string_exists(&tx, item, "itemType", "Person2")?.not());

        // The property should have a String value,
        // so normally this would be a schema check error.
        // However, database_api is the lowest layer and it's unaware of schemas.
        // The result is a successful check with the result "no, such integer value is not found")
        assert!(check_integer_exists(&tx, item, "itemType", 1)?.not());
        assert!(check_real_exists(&tx, item, "itemType", 1.)?.not());

        Ok(())
    }
}
