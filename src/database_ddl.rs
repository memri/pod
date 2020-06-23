use std::string::String;

pub fn create_mandatory_tables() {
    let items = String::from(
        "CREATE TABLE IF NOT EXISTS items (
        id integer PRIMARY KEY,
        type text NOT NULL,
        created_at text NOT NULL,
        modified_at text NOT NULL,
        read_by_user_at text NOT NULL,
        version integer NOT NULL,
        memri_id text NOT NULL UNIQUE
        )",
    );

    let edges = String::from(
        "CREATE TABLE IF NOT EXISTS edges (
        source integer NOT NULL,
        target integer NOT NULL,
        type text NOT NULL,
        created_at text,
        modified_at text,
        read_at text
        )",
    );
}
