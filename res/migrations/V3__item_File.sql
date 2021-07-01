INSERT INTO items(id, type, dateCreated, dateModified, dateServerModified, deleted) VALUES(
    "419d1188b61dfa7d0a18e20794c843",
    "ItemPropertySchema", 0, 0, 0, 0
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "419d1188b61dfa7d0a18e20794c843"),
    "itemType", "File"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "419d1188b61dfa7d0a18e20794c843"),
    "propertyName", "sha256"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "419d1188b61dfa7d0a18e20794c843"),
    "valueType", "Text"
);


INSERT INTO items(id, type, dateCreated, dateModified, dateServerModified, deleted) VALUES(
    "16fa737654dc376a20976d8ec89033c2",
    "ItemPropertySchema", 0, 0, 0, 0
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "16fa737654dc376a20976d8ec89033c2"),
    "itemType", "File"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "16fa737654dc376a20976d8ec89033c2"),
    "propertyName", "key"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "16fa737654dc376a20976d8ec89033c2"),
    "valueType", "Text"
);


INSERT INTO items(id, type, dateCreated, dateModified, dateServerModified, deleted) VALUES(
    "25994759432d4a636599c5e9c987eb95",
    "ItemPropertySchema", 0, 0, 0, 0
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "25994759432d4a636599c5e9c987eb95"),
    "itemType", "File"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "25994759432d4a636599c5e9c987eb95"),
    "propertyName", "nonce"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "25994759432d4a636599c5e9c987eb95"),
    "valueType", "Text"
);
