-- Items of type "PluginRun" have property "containerImage" (text)
INSERT INTO items(id, type, dateCreated, dateModified, dateServerModified, deleted) VALUES(
    "8c914705a392432381561db94da76ad9",
    "ItemPropertySchema", 0, 0, 0, 0
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "8c914705a392432381561db94da76ad9"),
    "itemType", "PluginRun"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "8c914705a392432381561db94da76ad9"),
    "propertyName", "containerImage"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "8c914705a392432381561db94da76ad9"),
    "valueType", "Text"
);

-- Items of type "PluginRun" have property "targetItemId" (text)
INSERT INTO items(id, type, dateCreated, dateModified, dateServerModified, deleted) VALUES(
    "5f43d3c0120c4f0cac3c98b8129c9cb7",
    "ItemPropertySchema", 0, 0, 0, 0
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "5f43d3c0120c4f0cac3c98b8129c9cb7"),
    "itemType", "PluginRun"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "5f43d3c0120c4f0cac3c98b8129c9cb7"),
    "propertyName", "targetItemId"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "5f43d3c0120c4f0cac3c98b8129c9cb7"),
    "valueType", "Text"
);
