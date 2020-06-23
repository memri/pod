pub fn create_mandatory_tables() {
    let items = "CREATE TABLE IF NOT EXISTS items (
        id INTEGER NOT NULL PRIMARY KEY,
        _type TEXT NOT NULL,
        created_at REAL NOT NULL,
        modified_at REAL NOT NULL,
        read_at REAL,
        version INTEGER NOT NULL
        );";

    let items_indexes = "CREATE INDEX IF NOT EXISTS idx_items_created_at ON items (created_at);
        CREATE INDEX IF NOT EXISTS idx_items_modified_at ON items (modified_at);
        CREATE INDEX IF NOT EXISTS idx_items_read_at ON items (read_at) WHERE read_at IS NOT NULL;
        CREATE INDEX IF NOT EXISTS idx_items_type_modified_at ON items (_type, modified_at);";

    let relations = "CREATE TABLE IF NOT EXISTS relations (
        source INTEGER NOT NULL,
        target INTEGER NOT NULL,
        _type TEXT NOT NULL,
        created_at REAL NOT NULL,
        modified_at REAL NOT NULL,
        read_at REAL,
        FOREIGN KEY (source) REFERENCES items (id),
        FOREIGN KEY (target) REFERENCES items (id),
        UNIQUE(source, target, _type)
        );";

    let relations_indexes =
        "CREATE INDEX IF NOT EXISTS idx_relations_direct ON relations (source, _type);
        CREATE INDEX IF NOT EXISTS idx_relations_reverse ON relations (target, _type);
        CREATE INDEX IF NOT EXISTS idx_relations_modified_at ON relations (modified_at);
        CREATE INDEX IF NOT EXISTS idx_relations_read_at ON relations (read_at) WHERE read_at IS NOT NULL;";
}
