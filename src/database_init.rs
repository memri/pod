use log::debug;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::NO_PARAMS;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::hash::Hash;

/// Constraints:
///
/// * All column definitions of the same case-insensitive name MUST have the same type and indexing
///
/// * All column names MUST consist of `a-zA-Z_` characters only,
/// and start with `a-zA-Z`
///
/// * All type names MUST consist of `a-zA-Z_` characters only,
/// and start with `a-zA-Z` (same as column names)
#[derive(Serialize, Deserialize)]
struct DatabaseSchema {
    types: Vec<DatabaseType>,
}

#[derive(Serialize, Deserialize)]
struct DatabaseType {
    name: String,
    columns: Vec<DatabaseColumn>,
}

#[derive(Serialize, Deserialize)]
struct DatabaseColumn {
    name: String,
    indexed: bool,
    _type: DatabaseColumnType,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
enum DatabaseColumnType {
    /// UTF-8 string
    Text,
    /// Signed 8-byte integer
    Integer,
    // 8-byte float
    Real,
}

pub const AUTOGENERATED_SCHEMA: &[u8] = include_bytes!("../res/autogenerated_database_schema.json");

pub fn init(sqlite: &Pool<SqliteConnectionManager>) {
    debug!("Initializing database schema (mandatory tables)");
    let conn = sqlite
        .get()
        .expect("Failed to aquire SQLite connection during db initialization");
    conn.execute_batch(ITEMS_TABLE_DDL)
        .unwrap_or_else(|err| panic!("Failed to create items table, {}", err));
    conn.execute_batch(RELATIONS_TALBLE_DDL)
        .unwrap_or_else(|err| panic!("Failed to create relations table, {}", err));
    conn.execute_batch(ITEMS_INDEX_DDL)
        .unwrap_or_else(|err| panic!("Failed to create items indexes, {}", err));
    conn.execute_batch(RELATION_INDEXES_DDL)
        .unwrap_or_else(|err| panic!("Failed to create relations indexes, {}", err));

    debug!("Initializing database schema (additional columns)");
    let parsed_schema: DatabaseSchema = serde_json::from_slice(AUTOGENERATED_SCHEMA)
        .expect("Failed to parse autogenerated_database_schema to JSON");
    let (indexed_columns, declared_columns) = get_column_info(parsed_schema, &conn);
    let sql = generate_sql(indexed_columns, declared_columns);

    conn.execute_batch(&sql)
        .unwrap_or_else(|err| panic!("Failed to execute SQL\n{}\n{}", sql, err));
}

fn get_column_info(
    parsed_schema: DatabaseSchema,
    conn: &PooledConnection<SqliteConnectionManager>,
) -> (Vec<String>, HashMap<String, DatabaseColumnType>) {
    let mut indexed_columns = Vec::new();
    let mut declared_columns = HashMap::new();

    let columns = parsed_schema.types.iter().flat_map(|t| &t.columns);
    let columns_grouped = group_by(columns, |c| c.name.to_lowercase());
    for (column_name, column_group) in columns_grouped {
        if MANDATORY_ITEMS_FIELDS.contains(&column_name.as_str()) {
            continue;
        }

        let needs_index = column_group.iter().any(|c| c.indexed);
        if needs_index {
            indexed_columns.push(column_name.to_string());
        };

        if column_exists("items", &column_name, conn) {
            continue;
        }

        let db_type = column_group
            .first()
            .expect("All groups are guaranteed to be non-empty");
        let db_type = db_type._type;
        let consistent_types = column_group.iter().all(|c| c._type == db_type);
        assert!(
            consistent_types,
            "Column {} has inconsistent database type",
            column_name
        );
        declared_columns.insert(column_name, db_type);
    }
    (indexed_columns, declared_columns)
}

// https://stackoverflow.com/questions/18920136/check-if-a-column-exists-in-sqlite
fn column_exists(
    table: &str,
    column: &str,
    conn: &PooledConnection<SqliteConnectionManager>,
) -> bool {
    let sql = format!(
        "SELECT COUNT(*) AS CNTREC FROM pragma_table_info('{}') WHERE name='{}';",
        table, column
    );
    let result: i64 = conn
        .query_row(&sql, NO_PARAMS, |row| row.get(0))
        .expect("Failed to query SQLite column information");
    result != 0
}

fn generate_sql(
    indexed_columns: Vec<String>,
    declared_columns: HashMap<String, DatabaseColumnType>,
) -> String {
    let mut result = String::new();

    for (column, db_type) in declared_columns {
        let creation = format!("ALTER TABLE items ADD {} {:?};", column, db_type);
        result.push_str(&creation);
        result.push_str("\n");
    }

    for column in indexed_columns {
        let index = format!(
            "CREATE INDEX IF NOT EXISTS idx_items_{} ON items({}) WHERE {} IS NOT NULL;",
            column, column, column
        );
        result.push_str(&index);
        result.push_str("\n");
    }

    result
}

pub fn group_by<T, K, F>(collection: T, grouping_func: F) -> HashMap<K, Vec<T::Item>>
where
    T: IntoIterator,
    F: Fn(&T::Item) -> K,
    K: Eq + Hash,
{
    let mut map = HashMap::new();
    for item in collection {
        let group = grouping_func(&item);
        map.entry(group).or_insert_with(|| vec![]).push(item);
    }
    map
}

const MANDATORY_ITEMS_FIELDS: &[&str] = &[
    "id",
    "_type",
    "created_at",
    "modified_at",
    "read_at",
    "deleted_at",
    "version",
];

const ITEMS_TABLE_DDL: &str = "CREATE TABLE IF NOT EXISTS items (
    id INTEGER NOT NULL PRIMARY KEY,
    _type TEXT NOT NULL,
    created_at REAL NOT NULL,
    modified_at REAL NOT NULL,
    read_at REAL,
    deleted_at REAL,
    version INTEGER NOT NULL
);";

const ITEMS_INDEX_DDL: &str = "
    CREATE INDEX IF NOT EXISTS idx_items_created_at ON items(created_at);
    CREATE INDEX IF NOT EXISTS idx_items_modified_at ON items(modified_at);
    CREATE INDEX IF NOT EXISTS idx_items_read_at ON items(read_at) WHERE read_at IS NOT NULL;
    CREATE INDEX IF NOT EXISTS idx_items_type_modified_at ON items(_type, modified_at);
";

const RELATIONS_TALBLE_DDL: &str = "CREATE TABLE IF NOT EXISTS relations (
    source INTEGER NOT NULL,
    target INTEGER NOT NULL,
    _type TEXT NOT NULL,
    created_at REAL NOT NULL,
    modified_at REAL NOT NULL,
    read_at REAL,
    FOREIGN KEY (source) REFERENCES items(id),
    FOREIGN KEY (target) REFERENCES items(id),
    UNIQUE(source, target, _type)
);";

const RELATION_INDEXES_DDL: &str = "
    CREATE INDEX IF NOT EXISTS idx_relations_source_type ON relations(source, _type);
    CREATE INDEX IF NOT EXISTS idx_relations_target_type ON relations(target, _type);
    CREATE INDEX IF NOT EXISTS idx_relations_modified_at ON relations(modified_at);
    CREATE INDEX IF NOT EXISTS idx_relations_read_at ON relations(read_at) WHERE read_at IS NOT NULL;
";
