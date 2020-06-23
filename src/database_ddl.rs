use std::string::String;

pub fn create_mandatory_tables() {
    let items = String::from(
        "CREATE TABLE IF NOT EXISTS items (
        id integer PRIMARY KEY,
        type text NOT NULL,
        created_at text NOT NULL,
        modified_at text NOT NULL,
        read_at text,
        version integer NOT NULL,
        memri_id text
        )",
    );

    let items_indexes = String::from(
        "CREATE UNIQUE INDEX idx_items_id
        ON items (id);
        CREATE INDEX idx_items_memri_type
        ON items (type);
        CREATE INDEX idx_items_memri_created_at
        ON items (created_at);
        CREATE INDEX idx_items_memri_modified_at
        ON items (modified_at);
        CREATE INDEX idx_items_memri_read_at
        ON items (read_at);
        CREATE UNIQUE INDEX idx_items_memri_id
        ON items (memri_id);",
    );

    let relations = String::from(
        "CREATE TABLE IF NOT EXISTS edges (
        source integer NOT NULL,
        target integer NOT NULL,
        type text NOT NULL,
        created_at text NOT NULL,
        modified_at text NOT NULL,
        read_at text
        )",
    );

    let relations_indexes = String::from(
        "CREATE UNIQUE INDEX idx_relations_source
        ON relations (source);
        CREATE UNIQUE INDEX idx_relations_target
        ON relations (target);
        CREATE UNIQUE INDEX idx_relations_type
        ON relations (type);
        CREATE INDEX idx_relations_direct
        ON relations (source, type);
        CREATE INDEX idx_relations_reverse
        ON relations (target, type);",
    );
}
