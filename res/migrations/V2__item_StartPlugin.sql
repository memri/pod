-- Items of type "StartPlugin" have property "container" (text)
INSERT INTO items(id, type, dateCreated, dateModified, dateServerModified, deleted) VALUES(
    "8c914705a392432381561db94da76ad9",
    "ItemPropertySchema", 0, 0, 0, 0
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "8c914705a392432381561db94da76ad9"),
    "itemType", "StartPlugin"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "8c914705a392432381561db94da76ad9"),
    "propertyName", "container"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "8c914705a392432381561db94da76ad9"),
    "valueType", "text"
);

-- Items of type "StartPlugin" have property "targetItemId" (text)
INSERT INTO items(id, type, dateCreated, dateModified, dateServerModified, deleted) VALUES(
    "5f43d3c0120c4f0cac3c98b8129c9cb7",
    "ItemPropertySchema", 0, 0, 0, 0
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "5f43d3c0120c4f0cac3c98b8129c9cb7"),
    "itemType", "StartPlugin"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "5f43d3c0120c4f0cac3c98b8129c9cb7"),
    "propertyName", "targetItemId"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "5f43d3c0120c4f0cac3c98b8129c9cb7"),
    "valueType", "text"
);
