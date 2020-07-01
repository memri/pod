CREATE TABLE IF NOT EXISTS items (
    uid INTEGER NOT NULL PRIMARY KEY,
    type TEXT NOT NULL,
    dateCreated REAL NOT NULL,
    dateModified REAL NOT NULL,
    dateAccessed REAL,
    deleted INTEGER,
    version INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_items_dateCreated ON items(dateCreated);
CREATE INDEX IF NOT EXISTS idx_items_dateModified ON items(dateModified);
CREATE INDEX IF NOT EXISTS idx_items_dateAccessed ON items(dateAccessed) WHERE dateAccessed IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_items_type_dateModified ON items(type, dateModified);

CREATE TABLE IF NOT EXISTS edges (
    source INTEGER NOT NULL,
    target INTEGER NOT NULL,
    type TEXT NOT NULL,
    dateCreated REAL NOT NULL,
    dateModified REAL NOT NULL,
    dateAccessed REAL,
    FOREIGN KEY (source) REFERENCES items(uid),
    FOREIGN KEY (target) REFERENCES items(uid),
    UNIQUE(source, target, type)
);

CREATE INDEX IF NOT EXISTS idx_edges_source_type ON edges(source, type);
CREATE INDEX IF NOT EXISTS idx_edges_target_type ON edges(target, type);
CREATE INDEX IF NOT EXISTS idx_edges_dateModified ON edges(dateModified);
CREATE INDEX IF NOT EXISTS idx_edges_dateAccessed ON edges(dateAccessed) WHERE dateAccessed IS NOT NULL;
