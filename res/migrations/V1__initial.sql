CREATE TABLE items (
    uid TEXT NOT NULL PRIMARY KEY,
    _type TEXT NOT NULL,
    dateCreated INTEGER /* datetime */ NOT NULL,
    dateModified INTEGER /* datetime */ NOT NULL,
    _dateServerModified INTEGER /* datetime */ NOT NULL,
    deleted INTEGER /* boolean */ NOT NULL DEFAULT 0,
    version INTEGER NOT NULL
);

CREATE TABLE edges (
    _source TEXT NOT NULL,
    _target TEXT NOT NULL,
    _type TEXT NOT NULL,
    FOREIGN KEY (_source) REFERENCES items(uid),
    FOREIGN KEY (_target) REFERENCES items(uid)
);

CREATE UNIQUE INDEX idx_edges_source_target_type on edges(_source, _type, _target);
