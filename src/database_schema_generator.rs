use serde::Deserialize;
use serde::Serialize;

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

#[derive(Serialize, Deserialize)]
enum DatabaseColumnType {
    Text,
    Integer,
    Real,
}

/// Given a DatabaseSchema from iOS or other components,
/// generate SQL statements that create all the needed columns and indices
fn _generate_columns_from_schema(_schema: DatabaseSchema) -> String {
    unimplemented!()
}
