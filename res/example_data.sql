-- This is an example data that you can insert into Pod for testing purposes.

-- Use it for example as:
--   sqlcipher -cmd "PRAGMA key = \"x'2DD29CA851E7B56E4697B0E1F08507293D761A05CE4D1B628663F411A8086D99'\";" data/db/*.db < res/example_data.sql

INSERT INTO items(rowid, id, type, dateCreated, dateModified, dateServerModified, deleted) VALUES(19000, "id-person-age", "ItemPropertySchema", 0, 0, 0, 0);
INSERT INTO strings(item, name, value) VALUES(19000, "itemType", "Person");
INSERT INTO strings(item, name, value) VALUES(19000, "propertyName", "age");
INSERT INTO strings(item, name, value) VALUES(19000, "valueType", "Integer");

INSERT INTO items(rowid, id, type, dateCreated, dateModified, dateServerModified, deleted) VALUES(19001, "id-person-name", "ItemPropertySchema", 0, 0, 0, 0);
INSERT INTO strings(item, name, value) VALUES(19001, "itemType", "Person");
INSERT INTO strings(item, name, value) VALUES(19001, "propertyName", "name");
INSERT INTO strings(item, name, value) VALUES(19001, "valueType", "Text");

INSERT INTO items(rowid, id, type, dateCreated, dateModified, dateServerModified, deleted) VALUES(19002, "first-id", "Person", 0, 0, 0, 0);
INSERT INTO integers(item, name, value) VALUES(19002, "age", 20);
INSERT INTO strings(item, name, value) VALUES(19002, "firstName", "Ada");
INSERT INTO strings(item, name, value) VALUES(19002, "lastName", "Lovelace");

SELECT item.type, item.id, valueType.name, valueType.value FROM items as item, strings as valueType WHERE valueType.item = item.rowid;
SELECT item.type, item.id, valueType.name, valueType.value FROM items as item, strings as valueType WHERE item.type = "ItemPropertySchema" AND valueType.item = item.rowid;

SELECT propertyName.value, valueType.value
FROM
   items as item,
   strings as propertyName,
   strings as valueType
WHERE item.type = 'ItemPropertySchema'
AND propertyName.item = item.rowid
AND valueType.item = item.rowid
AND propertyName.name = 'propertyName'
AND valueType.name = 'valueType';
