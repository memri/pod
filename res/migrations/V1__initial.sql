CREATE TABLE items (
    id INTEGER NOT NULL PRIMARY KEY,
    type TEXT NOT NULL,
    dateCreated INTEGER /* datetime */ NOT NULL,
    dateModified INTEGER /* datetime */ NOT NULL,
    dateServerModified INTEGER /* datetime */ NOT NULL,
    deleted INTEGER /* boolean */ NOT NULL DEFAULT 0,
    version INTEGER NOT NULL
);

CREATE TABLE itemproperties (
    itemId INTEGER NOT NULL,
    name TEXT NOT NULL,
    value BLOB NOT NULL,
    FOREIGN KEY (itemId) REFERENCES items(id)
);
CREATE INDEX idx_itemproperties_item on itemproperties(itemId);


CREATE TABLE edges (
    source INTEGER NOT NULL,
    target INTEGER NOT NULL,
    type TEXT NOT NULL,
    id INTEGER NOT NULL PRIMARY KEY,
    FOREIGN KEY (source) REFERENCES items(uid),
    FOREIGN KEY (target) REFERENCES items(uid)
);
CREATE UNIQUE INDEX idx_edges_source_target_type on edges(source, type, target);

CREATE TABLE edgeproperties (
    edgeId INTEGER NOT NULL,
    name TEXT NOT NULL,
    value BLOB NOT NULL,
    FOREIGN KEY (edgeId) REFERENCES edges(id)
);
CREATE INDEX idx_edgeproperties_item on edgeproperties(edgeId);
