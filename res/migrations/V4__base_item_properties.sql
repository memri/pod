-- id
INSERT INTO items(id, type, dateCreated, dateModified, dateServerModified, deleted) VALUES(
    "ccf9f0cba490ed58fa967f6a1136fc8",
    "ItemPropertySchema", 0, 0, 0, 0
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "ccf9f0cba490ed58fa967f6a1136fc8"),
    "itemType", "Item"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "ccf9f0cba490ed58fa967f6a1136fc8"),
    "propertyName", "id"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "ccf9f0cba490ed58fa967f6a1136fc8"),
    "valueType", "text"
);


-- type
INSERT INTO items(id, type, dateCreated, dateModified, dateServerModified, deleted) VALUES(
    "904ecd61ba00152d9ab9f235c13217",
    "ItemPropertySchema", 0, 0, 0, 0
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "904ecd61ba00152d9ab9f235c13217"),
    "itemType", "Item"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "904ecd61ba00152d9ab9f235c13217"),
    "propertyName", "type"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "904ecd61ba00152d9ab9f235c13217"),
    "valueType", "text"
);


-- dateCreated
INSERT INTO items(id, type, dateCreated, dateModified, dateServerModified, deleted) VALUES(
    "cb8b4455cdcabd25ba9d0eaabcc3786",
    "ItemPropertySchema", 0, 0, 0, 0
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "cb8b4455cdcabd25ba9d0eaabcc3786"),
    "itemType", "Item"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "cb8b4455cdcabd25ba9d0eaabcc3786"),
    "propertyName", "dateCreated"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "cb8b4455cdcabd25ba9d0eaabcc3786"),
    "valueType", "datetime"
);


-- dateModified
INSERT INTO items(id, type, dateCreated, dateModified, dateServerModified, deleted) VALUES(
    "d1757e7395932c32cc1dcecc03b2977",
    "ItemPropertySchema", 0, 0, 0, 0
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "d1757e7395932c32cc1dcecc03b2977"),
    "itemType", "Item"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "d1757e7395932c32cc1dcecc03b2977"),
    "propertyName", "dateModified"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "d1757e7395932c32cc1dcecc03b2977"),
    "valueType", "datetime"
);


-- dateServerModified
INSERT INTO items(id, type, dateCreated, dateModified, dateServerModified, deleted) VALUES(
    "8f492d26bd192ad53cf07f3ad36cbae7",
    "ItemPropertySchema", 0, 0, 0, 0
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "8f492d26bd192ad53cf07f3ad36cbae7"),
    "itemType", "Item"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "8f492d26bd192ad53cf07f3ad36cbae7"),
    "propertyName", "dateServerModified"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "8f492d26bd192ad53cf07f3ad36cbae7"),
    "valueType", "datetime"
);


-- deleted
INSERT INTO items(id, type, dateCreated, dateModified, dateServerModified, deleted) VALUES(
    "8716a96393c48255542ca6d54afe2bc6",
    "ItemPropertySchema", 0, 0, 0, 0
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "8716a96393c48255542ca6d54afe2bc6"),
    "itemType", "Item"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "8716a96393c48255542ca6d54afe2bc6"),
    "propertyName", "deleted"
);
INSERT INTO strings(item, name, value) VALUES(
    (SELECT rowid FROM items WHERE id = "8716a96393c48255542ca6d54afe2bc6"),
    "valueType", "bool"
);
