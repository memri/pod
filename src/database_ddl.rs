use std::string::String;

pub fn create_mandatory_tables() {
    let items = String::from(
        "CREATE TABLE IF NOT EXISTS items (
        id INTEGER NOT NULL PRIMARY KEY,
        type TEXT NOT NULL,
        created_at REAL NOT NULL,
        modified_at REAL NOT NULL,
        read_at REAL,
        version INTEGER NOT NULL
        )",
    );

    let items_indexes = String::from(
        "CREATE INDEX IF NOT EXISTS idx_items_type ON items (type);
        CREATE INDEX IF NOT EXISTS idx_items_created_at ON items (created_at);
        CREATE INDEX IF NOT EXISTS idx_items_modified_at ON items (modified_at);
        CREATE INDEX IF NOT EXISTS idx_items_read_at ON items (read_at) WHERE read_at IS NOT NULL;",
    );

    let relations = String::from(
        "CREATE TABLE IF NOT EXISTS relations (
        source INTEGER NOT NULL,
        FOREIGN KEY (source) REFERENCES items (id);
        target INTEGER NOT NULL,
        FOREIGN KEY (target) REFERENCES items (id);
        type TEXT NOT NULL,
        created_at REAL NOT NULL,
        modified_at REAL NOT NULL,
        read_at REAL,
        UNIQUE(source, target, type)
        )",
    );

    let relations_indexes = String::from(
        "CREATE INDEX IF NOT EXISTS idx_relations_direct ON relations (source, type);
        CREATE INDEX IF NOT EXISTS idx_relations_reverse ON relations (target, type);",
    );
}
