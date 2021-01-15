CREATE TABLE items (
    rowid INTEGER PRIMARY KEY,
    id TEXT NOT NULL,
    type TEXT NOT NULL,
    dateCreated INTEGER /* datetime */ NOT NULL,
    dateModified INTEGER /* datetime */ NOT NULL,
    dateServerModified INTEGER /* datetime */ NOT NULL,
    deleted INTEGER /* boolean */ NOT NULL
);
CREATE UNIQUE INDEX idx_items_id on items(id);
CREATE        INDEX idx_items_type_dateServerModified on items(type, dateServerModified);
CREATE        INDEX idx_items_dateServerModified on items(dateServerModified);


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


-- Items of type "ItemPropertySchema" have property "itemType" (text)
INSERT INTO items(rowid, id, type, dateCreated, dateModified, dateServerModified, deleted) VALUES(1, "ad61e2770eb64d81a3ddfa03fa5f887f", "ItemPropertySchema", 0, 0, 0, 0);
INSERT INTO strings(item, name, value) VALUES(1, "itemType", "ItemPropertySchema");
INSERT INTO strings(item, name, value) VALUES(1, "propertyName", "itemType");
INSERT INTO strings(item, name, value) VALUES(1, "valueType", "text");

-- Items of type "ItemPropertySchema" have property "propertyName" (text)
INSERT INTO items(rowid, id, type, dateCreated, dateModified, dateServerModified, deleted) VALUES(2, "7bce841921beb3295634b224bc75a990", "ItemPropertySchema", 0, 0, 0, 0);
INSERT INTO strings(item, name, value) VALUES(2, "itemType", "ItemPropertySchema");
INSERT INTO strings(item, name, value) VALUES(2, "propertyName", "propertyName");
INSERT INTO strings(item, name, value) VALUES(2, "valueType", "text");

-- Items of type "ItemPropertySchema" have property "valueType" (text)
INSERT INTO items(rowid, id, type, dateCreated, dateModified, dateServerModified, deleted) VALUES(3, "791427a0571e394d1f29cb63ee77e5", "ItemPropertySchema", 0, 0, 0, 0);
INSERT INTO strings(item, name, value) VALUES(3, "itemType", "ItemPropertySchema");
INSERT INTO strings(item, name, value) VALUES(3, "propertyName", "valueType");
INSERT INTO strings(item, name, value) VALUES(3, "valueType", "text");
