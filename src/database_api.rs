use crate::error::Result;
// use crate::schema::Schema;
// use crate::sql_converters;
use rusqlite::params;
use rusqlite::ToSql;
use rusqlite::Transaction;
use rusqlite::types::ToSqlOutput;
// use rusqlite::NO_PARAMS;

type Rowid = i64;
type DBTime = i64;

/// Low-level function to insert an item.
/// No Schema/type checks are done. Use other functions around instead.
#[allow(dead_code)]
fn insert_item_unchecked(
    tx: &Transaction,
    _type: &str,
    date_created_millis: DBTime,
    date_modified_millis: DBTime,
    date_server_modified_millis: DBTime,
    deleted: bool,
) -> Result<Rowid> {
    let mut stmt = tx.prepare_cached(
        "INSERT INTO items (\
            type, \
            dateCreated, \
            dateModified, \
            dateServerModified, \
            deleted, \
            version\
        ) VALUES (?, ?, ?, ?, ?, ?);",
    )?;
    stmt.execute(params![
        _type,
        date_created_millis,
        date_modified_millis,
        date_server_modified_millis,
        deleted,
        1
    ])?;
    Ok(tx.last_insert_rowid())
}

/// Low-level function to insert a scalar.
/// No Schema/type checks are done. Use other functions around instead.
#[allow(dead_code)]
fn insert_scalar_unchecked<S>(tx: &Transaction, item: Rowid, name: &str, value: S) -> Result<()>
where
    S: ToSql,
{
    let mut stmt = tx.prepare_cached("INSERT INTO scalars VALUES(?, ?, ?);")?;
    stmt.execute(params![item, name, value])?;
    Ok(())
}

/// Low-level function to insert an edge.
/// No Schema/type checks are done. Use other functions around instead.
#[allow(dead_code)]
fn insert_edge_unchecked(
    tx: &Transaction,
    source: Rowid,
    name: &str,
    target: Rowid,
    date: DBTime,
) -> Result<Rowid> {
    let item = insert_item_unchecked(tx, name, date, date, date, false)?;
    let mut stmt =
        tx.prepare_cached("INSERT INTO edges(self, source, name, target) VALUES(?, ?, ?, ?);")?;
    stmt.execute(params![item, source, name, target])?;
    Ok(item)
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Comparison {
    Equals,
    GreaterThan,
    GreaterOrEquals,
    LessThan,
    LessOrEquals,
}

fn add_integer_param<'a>(
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

#[allow(dead_code)]
fn search_items(
    tx: &Transaction,
    _rowid: Option<Rowid>,
    _type: Option<&str>,
    _date_created: Option<DBTime>,
    _date_modified: Option<DBTime>,
    _date_server_modified: Option<DBTime>,
    _deleted: DBTime,
) -> Result<()>{
    // let mut p = params![0];
    let mut params_vec: Vec<ToSqlOutput> = Vec::new();
    add_integer_param(String::new(), &mut params_vec, "age", &Comparison::Equals, 0);
    add_integer_param(String::new(), &mut params_vec, "boo", &Comparison::Equals, 0);
    let mut stmt = tx.prepare_cached("")?;
    let _ = stmt.query_map(params_vec, |_row| Ok(0))?;

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
    use rusqlite::Connection;

    fn new_conn() -> Connection {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        database_migrate_refinery::embedded::migrations::runner()
            .run(&mut conn)
            .expect("Failed to run refinery migrations");
        conn
    }

    #[test]
    fn test_insert_item() -> Result<()> {
        let mut conn = new_conn();
        let date = Utc::now().timestamp_millis();
        let tx = conn.transaction()?;
        let item = insert_item_unchecked(&tx, "Person", date, date, date, false)?;
        assert_eq!(item, 1);
        let item = insert_item_unchecked(&tx, "Book", date, date, date, false)?;
        assert_eq!(item, 2);
        Ok(())
    }

    #[test]
    fn test_insert_scalars() -> Result<()> {
        let mut conn = new_conn();
        let date = Utc::now().timestamp_millis();
        let tx = conn.transaction()?;
        let item = insert_item_unchecked(&tx, "Person", date, date, date, false)?;
        assert_eq!(item, 1);
        insert_scalar_unchecked(&tx, item, "age", 20)?;
        insert_scalar_unchecked(&tx, item, "skill", 17)?;
        Ok(())
    }

    #[test]
    fn test_insert_edge() -> Result<()> {
        let mut conn = new_conn();
        let date = Utc::now().timestamp_millis();
        let tx = conn.transaction()?;
        let source = insert_item_unchecked(&tx, "Person", date, date, date, false)?;
        assert_eq!(source, 1);
        let target = insert_item_unchecked(&tx, "Person", date, date, date, false)?;
        assert_eq!(target, 2);
        let edge = insert_edge_unchecked(&tx, source, "friend", target, date)?;
        assert_eq!(edge, 3);
        Ok(())
    }
}
