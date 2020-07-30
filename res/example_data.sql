-- This is an example data that you can insert into Pod for testing purposes.

-- Use it for example as:
--   sqlcipher -cmd "PRAGMA key = \"x'2DD29CA851E7B56E4697B0E1F08507293D761A05CE4D1B628663F411A8086D99'\";" data/db/*.db < res/example_data.sql

INSERT INTO items (uid, _type, dateCreated, dateModified, deleted, version, firstName, gender, lastName) VALUES
  (1, "Person", 0, 0, 0, 1, "John", "male", "Doe"),
  (2, "Person", 0, 0, 0, 1, "David", "male", null),
  (3, "Person", 0, 0, 0, 1, "Eli", "male", null),
  (4, "Person", 0, 0, 0, 1, "George", "male", "Sears"),
  (5, "Person", 0, 0, 0, 1, "Jack", "male", null);
INSERT INTO items (uid, _type, dateCreated, dateModified, deleted, version, state, city) VALUES
  (6, "Address", 0, 0, 0, 1, "United States", null);
INSERT INTO items (uid, _type, dateCreated, dateModified, deleted, version, email) VALUES
  (7, "Company", 0, 0, 0, 1, "zanzibarland"),
  (8, "Company", 0, 0, 0, 1, "foxhound"),
  (9, "Company", 0, 0, 0, 1, "thepatriots");

INSERT INTO edges (_source, _target, _type) VALUES
  (1, 2, "Father of"),
  (1, 3, "Father of"),
  (1, 4, "Father of"),
  (4, 5, "Father of"),
  (2, 3, "Brother of"),
  (2, 4, "Brother of"),
  (3, 4, "Brother of"),
  (2, 5, "Friend of"),
  (1, 6, "Born in"),
  (2, 6, "Born in"),
  (3, 6, "Born in"),
  (1, 7, "Belong to"),
  (1, 8, "Belong to"),
  (1, 9, "Belong to"),
  (2, 8, "Belong to"),
  (3, 8, "Belong to"),
  (4, 9, "Belong to"),
  (5, 8, "Belong to");
