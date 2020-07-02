CREATE TABLE items (
    uid INTEGER NOT NULL PRIMARY KEY,
    type TEXT NOT NULL,
    dateCreated REAL NOT NULL,
    dateModified REAL NOT NULL,
    deleted INTEGER /* boolean */ NOT NULL,
    version INTEGER NOT NULL
);

CREATE TABLE edges (
    source INTEGER NOT NULL,
    target INTEGER NOT NULL,
    type TEXT NOT NULL,
    FOREIGN KEY (source) REFERENCES items(uid),
    FOREIGN KEY (target) REFERENCES items(uid)
);

CREATE UNIQUE INDEX idx_edges_source_target_type on edges(source, type, target);
