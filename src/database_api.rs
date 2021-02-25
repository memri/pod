use crate::error::Result;
use crate::schema::Schema;
use crate::schema::SchemaPropertyType;
use rusqlite::params;
use rusqlite::types::ToSqlOutput;
use rusqlite::Transaction;
use rusqlite::NO_PARAMS;
use std::collections::HashMap;

type Rowid = i64;
type DBTime = i64;

/// Low-level function to insert an item.
/// No Schema/type checks are done. Use other functions around instead.
#[allow(dead_code)]
fn insert_item_base_unchecked(
    tx: &Transaction,
    id: &str,
    _type: &str,
    date_created_millis: DBTime,
    date_modified_millis: DBTime,
    date_server_modified_millis: DBTime,
    deleted: bool,
    version: i64,
) -> Result<Rowid> {
    let mut stmt = tx.prepare_cached(
        "INSERT INTO items (\
            id, \
            type, \
            dateCreated, \
            dateModified, \
            dateServerModified, \
            deleted, \
            version\
        ) VALUES (?, ?, ?, ?, ?, ?, ?);",
    )?;
    stmt.execute(params![
        id,
        _type,
        date_created_millis,
        date_modified_millis,
        date_server_modified_millis,
        deleted,
        version
    ])?;
    Ok(tx.last_insert_rowid())
}

/// Low-level function to insert an edge.
/// No Schema/type checks are done. Use other functions around instead.
#[allow(dead_code)]
fn insert_edge_unchecked(
    tx: &Transaction,
    source: Rowid,
    name: &str,
    target: Rowid,
    id: &str,
    date: DBTime,
    version: i64,
) -> Result<Rowid> {
    let item = insert_item_base_unchecked(tx, id, name, date, date, date, false, version)?;
    let mut stmt =
        tx.prepare_cached("INSERT INTO edges(self, source, name, target) VALUES(?, ?, ?, ?);")?;
    stmt.execute(params![item, source, name, target])?;
    Ok(item)
}

#[allow(dead_code)]
fn insert_integer_unchecked(tx: &Transaction, item: Rowid, name: &str, value: i64) -> Result<()> {
    let mut stmt = tx.prepare_cached("INSERT INTO integers VALUES(?, ?, ?);")?;
    stmt.execute(params![item, name, value])?;
    Ok(())
}

#[allow(dead_code)]
fn insert_real_unchecked(tx: &Transaction, item: Rowid, name: &str, value: f64) -> Result<()> {
    let mut stmt = tx.prepare_cached("INSERT INTO reals VALUES(?, ?, ?);")?;
    stmt.execute(params![item, name, value])?;
    Ok(())
}

#[allow(dead_code)]
fn insert_string_unchecked(tx: &Transaction, item: Rowid, name: &str, value: &str) -> Result<()> {
    let mut stmt = tx.prepare_cached("INSERT INTO strings VALUES(?, ?, ?);")?;
    stmt.execute(params![item, name, value])?;
    Ok(())
}

#[allow(dead_code)]
pub fn read_schema(tx: &Transaction) -> Result<Schema> {
    let mut stmt = tx.prepare_cached("SELECT propertyName, valueType FROM itemSchema;")?;
    let mut rows = stmt.query(NO_PARAMS)?;
    let mut property_types: HashMap<String, SchemaPropertyType> = HashMap::new();
    while let Some(row) = rows.next()? {
        let property_name: String = row.get(0)?;
        let value_type: String = row.get(1)?;
        let value_type = SchemaPropertyType::from_string(&value_type)?;
        property_types.insert(property_name, value_type);
    }
    Ok(Schema { property_types })
}

pub fn read_item_schema_joins(tx: &Transaction) -> Result<Schema> {
    let mut stmt = tx.prepare_cached(
        "SELECT thisProperty.value, thisType.value \
        FROM \
            items as item,
            strings as thisProperty,
            strings as thisType
        WHERE item.type = 'ItemPropertySchema'
        AND thisProperty.item = item.rowid
        AND thisType.item = item.rowid
        AND thisProperty.name = 'propertyName'
        AND thisType.name = 'valueType';",
    )?;
    let mut rows = stmt.query(NO_PARAMS)?;
    let mut property_types: HashMap<String, SchemaPropertyType> = HashMap::new();
    while let Some(row) = rows.next()? {
        let this_property: String = row.get(0)?;
        let this_type: String = row.get(1)?;
        let value_type = SchemaPropertyType::from_string(&this_type)?;
        property_types.insert(this_property, value_type);
    }
    Ok(Schema { property_types })
}

// pub fn read_item_schema_alternative2222222(tx: &Transaction) -> Result<Schema> {
//     // Steps:
//     // * find items by type
//     // * find the two properties for each item
//     // * aggregate the results into a Schema
// }

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Comparison {
    Equals,
    GreaterThan,
    GreaterOrEquals,
    LessThan,
    LessOrEquals,
}

fn add_query_integer_param(
    mut query: String,
    params: &mut Vec<ToSqlOutput>,
    column: &str,
    operation: &Comparison,
    value: i64,
) {
    query.push_str("AND ");
    query.push_str(column);
    match operation {
        Comparison::Equals => query.push_str(" = "),
        Comparison::GreaterThan => query.push_str(" > "),
        Comparison::GreaterOrEquals => query.push_str(" >= "),
        Comparison::LessThan => query.push_str(" < "),
        Comparison::LessOrEquals => query.push_str(" <= "),
    };
    query.push_str("?");
    let value: ToSqlOutput = value.into();
    params.push(value);
}

fn add_sql_param(mut query: String, column: &str, operation: &Comparison) -> Result<()> {
    query.push_str("AND ");
    query.push_str(column);
    match operation {
        Comparison::Equals => query.push_str(" = "),
        Comparison::GreaterThan => query.push_str(" > "),
        Comparison::GreaterOrEquals => query.push_str(" >= "),
        Comparison::LessThan => query.push_str(" < "),
        Comparison::LessOrEquals => query.push_str(" <= "),
    };
    query.push_str("?");
    Ok(())
}

// fn add_query_string_param<'a>(
//     mut query: String,
//     params: &mut Vec<ToSqlOutput>,
//     column: &str,
//     operation: &Comparison,
//     value: &str,
// ) -> Result<()> {
//     query.push_str("AND ");
//     query.push_str(column);
//     match operation {
//         Comparison::Equals => query.push_str(" = "),
//         Comparison::GreaterThan => query.push_str(" > "),
//         Comparison::GreaterOrEquals => query.push_str(" >= "),
//         Comparison::LessThan => query.push_str(" < "),
//         Comparison::LessOrEquals => query.push_str(" <= "),
//     };
//     query.push_str("?");
//     let value: ToSqlOutput = value.into();
//     params.push(value);
//     Ok(())
// }

// fn add_query_param<V>(
//     mut query: String,
//     params: &mut Vec<ToSqlOutput>,
//     column: &str,
//     operation: &Comparison,
//     value: V,
// ) -> Result<()>
//     where
//         V: ToSql,
// {
//     query.push_str("AND ");
//     query.push_str(column);
//     match operation {
//         Comparison::Equals => query.push_str(" = "),
//         Comparison::GreaterThan => query.push_str(" > "),
//         Comparison::GreaterOrEquals => query.push_str(" >= "),
//         Comparison::LessThan => query.push_str(" < "),
//         Comparison::LessOrEquals => query.push_str(" <= "),
//     };
//     query.push_str("?");
//     let value: ToSqlOutput = value.to_sql()?;
//     params.push(value);
//     Ok(())
// }

#[allow(dead_code)]
fn search_items_iter(
    tx: &Transaction,
    _rowid: Option<Rowid>,
    _type: Option<&str>,
    _date_created: Option<DBTime>,
    _date_modified: Option<DBTime>,
    _date_server_modified: Option<DBTime>,
    _deleted: DBTime,
) -> Result<()> {
    // let mut p = params![0];
    let mut params_vec: Vec<ToSqlOutput> = Vec::new();
    add_query_integer_param(
        String::new(),
        &mut params_vec,
        "age",
        &Comparison::Equals,
        0,
    );
    add_query_integer_param(
        String::new(),
        &mut params_vec,
        "boo",
        &Comparison::Equals,
        0,
    );
    let mut stmt = tx.prepare_cached("")?;
    let r = stmt.query_map(params_vec, |_row| Ok(0))?;
    Ok(())
}

// /// Low-level function to get an item.
// /// No Schema/type checks are done. Use other functions around instead.
// #[allow(dead_code)]
// fn get_item_unchecked(
//     tx: &Transaction,
//     rowid: Rowid
// ) -> Result<Rowid> {
//     let mut stmt = tx.prepare_cached(
//         "SELECT \
//             type, \
//             dateCreated, \
//             dateModified, \
//             dateServerModified, \
//             deleted, \
//             version\
//         FROM items WHERE rowid = ?;",
//     )?;
//     let rows_iter = stmt.query_map(params![rowid], |row| {
//         let name: String = row.get(0)?;
//         let value = sqlite_value_to_json(row.get_raw(1), &name);
//         if let Some(value) = value {
//             Ok(Some((name, value)))
//         } else {
//             Ok(None)
//         }
//     })?;
//
//
//     stmt.execute(params![rowid])?;
//     Ok(tx.last_insert_rowid())
// }

// pub fn create_scalar_from_json(
//     tx: &Transaction,
//     item: Rowid,
//     name: &str,
//     value: serde_json::Value,
//     schema: &Schema,
// ) -> Result<()> {
//     let mut stmt = tx.prepare_cached("INSERT INTO scalars VALUES(?, ?, ?);")?;
//     let sql_val = sql_converters::json_value_and_schema_to_sqlite(&value, name, schema)?;
//     stmt.execute(params![item, name, sql_val])?;
//     Ok(())
// }

// #[allow(dead_code)]
// fn real_insert_todo() {
//     let mut conn = rusqlite::Connection::open_in_memory().unwrap();
//     insert_scalar_unchecked(&conn.transaction().unwrap(), 0, "", "");
// }

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
        let item1 = insert_item_base_unchecked(&tx, &random_id(), "Person", date, date, date, false, 1)?;
        let item2 = insert_item_base_unchecked(&tx, &random_id(), "Book", date, date, date, false, 1)?;
        assert_eq!(item2 - item1, 1);
        Ok(())
    }

    #[test]
    fn test_insert_scalars() -> Result<()> {
        let mut conn = new_conn();
        let tx = conn.transaction()?;
        let date = Utc::now().timestamp_millis();
        let item = insert_item_base_unchecked(&tx, &random_id(), "Person", date, date, date, false, 1)?;
        insert_integer_unchecked(&tx, item, "age", 20)?;
        insert_real_unchecked(&tx, item, "attack", 13.5)?;
        insert_string_unchecked(&tx, item, "trait", "resilient")?;
        Ok(())
    }

    #[test]
    fn test_insert_edge() -> Result<()> {
        let mut conn = new_conn();
        let tx = conn.transaction()?;
        let date = Utc::now().timestamp_millis();
        let source =
            insert_item_base_unchecked(&tx, &random_id(), "Person", date, date, date, false, 1)?;
        let target =
            insert_item_base_unchecked(&tx, &random_id(), "Person", date, date, date, false, 1)?;
        assert_eq!(target - source, 1);
        let edge = insert_edge_unchecked(&tx, source, "friend", target, &random_id(), date, 1)?;
        assert_eq!(edge - target, 1);
        Ok(())
    }

    #[test]
    fn test_default_schema() -> Result<()> {
        let mut conn = new_conn();
        let tx = conn.transaction()?;
        let schema = read_item_schema_joins(&tx)?;
        assert_eq!(schema.property_types.len(), 3);
        Ok(())
    }

    #[test]
    fn test_full_schema() -> Result<()> {
        let mut conn = new_conn();
        let tx = conn.transaction()?;
        let date = Utc::now().timestamp_millis();
        let item = insert_item_base_unchecked(
            &tx,
            &random_id(),
            "ItemPropertySchema",
            date,
            date,
            date,
            false,
            1,
        )?;
        insert_string_unchecked(&tx, item, "itemType", "Person")?;
        insert_string_unchecked(&tx, item, "propertyName", "age")?;
        insert_string_unchecked(&tx, item, "valueType", "integer")?;

        let item = insert_item_base_unchecked(
            &tx,
            &random_id(),
            "ItemPropertySchema",
            date,
            date,
            date,
            false,
            1,
        )?;
        insert_string_unchecked(&tx, item, "itemType", "Person")?;
        insert_string_unchecked(&tx, item, "propertyName", "name")?;
        insert_string_unchecked(&tx, item, "valueType", "text")?;

        let schema = read_item_schema_joins(&tx)?;
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
}
