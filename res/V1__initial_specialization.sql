--    type TEXT NOT NULL,
--    dateCreated INTEGER /* datetime */ NOT NULL,
--    dateModified INTEGER /* datetime */ NOT NULL,
--    dateServerModified INTEGER /* datetime */ NOT NULL,
--    deleted INTEGER /* boolean */ NOT NULL DEFAULT 0,
--    version INTEGER NOT NULL


CREATE TABLE items (
    rowid INTEGER PRIMARY KEY,
    id TEXT NOT NULL
);
CREATE UNIQUE INDEX idx_items_id on items(id);

CREATE TABLE props (
    item_rowid INTEGER NOT NULL,
    name TEXT NOT NULL,
    value BLOB NOT NULL,
    FOREIGN KEY (item_rowid) REFERENCES items(rowid)
);
CREATE INDEX idx_props_itemrowid_name on props(item_rowid, name);


CREATE TABLE edges (
    rowid INTEGER PRIMARY KEY,
    source_rowid INTEGER NOT NULL,
    target_rowid INTEGER NOT NULL,
    type TEXT NOT NULL,
    FOREIGN KEY (source_rowid) REFERENCES items(rowid),
    FOREIGN KEY (target_rowid) REFERENCES items(rowid)
);
CREATE UNIQUE INDEX idx_edges_source_target_type on edges(source_rowid, type, target_rowid);
///// TODO: reverse index

CREATE TABLE edgeproperties (
    edge_rowid INTEGER NOT NULL,
    name TEXT NOT NULL,
    value BLOB NOT NULL,
    FOREIGN KEY (edge_rowid) REFERENCES edges(rowid)
);
CREATE INDEX idx_edgeproperties_item on edgeproperties(edge_rowid);


