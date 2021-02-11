CREATE TABLE items (
  rowid INTEGER PRIMARY KEY
);


CREATE TABLE scalars (
  item INTEGER NOT NULL,
  name TEXT NOT NULL,
  value BLOB NOT NULL,
  FOREIGN KEY (item) REFERENCES items(rowid)
);
CREATE UNIQUE INDEX idx_scalars_item_name on scalars(item, name);
CREATE        INDEX idx_scalars_name_value on scalars(name, value);
CREATE        INDEX idx_scalars_name_item on scalars(name, item);


CREATE TABLE edges (
  source INTEGER NOT NULL,
  name TEXT NOT NULL,
  target INTEGER NOT NULL,
  self INTEGER NOT NULL,
  FOREIGN KEY (source) REFERENCES items(rowid),
  FOREIGN KEY (target) REFERENCES items(rowid),
  FOREIGN KEY (self) REFERENCES items(rowid)
);
CREATE INDEX idx_refs_source_name on edges(source, name);
CREATE INDEX idx_refs_target_name on edges(target, name);
