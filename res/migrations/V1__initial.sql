CREATE TABLE items (
    rowid INTEGER PRIMARY KEY,
    id TEXT NOT NULL,
    type TEXT NOT NULL,
    dateCreated INTEGER /* datetime */ NOT NULL,
    dateModified INTEGER /* datetime */ NOT NULL,
    dateServerModified INTEGER /* datetime */ NOT NULL,
    deleted INTEGER /* boolean */ NOT NULL,
    version INTEGER NOT NULL
);
CREATE UNIQUE INDEX idx_items_id on items(id);
CREATE        INDEX idx_items_type_dateServerModified on items(type, dateServerModified);


CREATE TABLE edges (
    self INTEGER PRIMARY KEY,
    source INTEGER NOT NULL,
    name TEXT NOT NULL,
    target INTEGER NOT NULL,
    FOREIGN KEY (source) REFERENCES items(rowid),
    FOREIGN KEY (target) REFERENCES items(rowid),
    FOREIGN KEY (self) REFERENCES items(rowid)
);
CREATE INDEX idx_edges_source_name on edges(source, name);
CREATE INDEX idx_edges_target_name on edges(target, name);


CREATE TABLE integers (
    item INTEGER NOT NULL,
    name TEXT NOT NULL,
    value INTEGER NOT NULL,
    FOREIGN KEY (item) REFERENCES items(rowid)
);
CREATE UNIQUE INDEX idx_integers_item_name on integers(item, name);
CREATE        INDEX idx_integers_name_value on integers(name, value);
CREATE        INDEX idx_integers_name_item on integers(name, item);


CREATE TABLE strings (
    item INTEGER NOT NULL,
    name TEXT NOT NULL,
    value TEXT NOT NULL,
    FOREIGN KEY (item) REFERENCES items(rowid)
);
CREATE UNIQUE INDEX idx_strings_item_name on strings(item, name);
CREATE        INDEX idx_strings_name_value on strings(name, value);
CREATE        INDEX idx_strings_name_item on strings(name, item);


CREATE TABLE reals (
    item INTEGER NOT NULL,
    name TEXT NOT NULL,
    value REAL NOT NULL,
    FOREIGN KEY (item) REFERENCES items(rowid)
);
CREATE UNIQUE INDEX idx_reals_item_name on reals(item, name);
CREATE        INDEX idx_reals_name_value on reals(name, value);
CREATE        INDEX idx_reals_name_item on reals(name, item);


CREATE TABLE itemSchema (
    itemType TEXT NOT NULL,
    propertyName TEXT NOT NULL,
    valueType TEXT NOT NULL
);

CREATE TABLE edgeSchema (
    sourceType TEXT NOT NULL,
    edgeName TEXT NOT NULL,
    targetType TEXT NOT NULL
);
