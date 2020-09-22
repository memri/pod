# About
This documentation is part of [Pod](../README.md).

In order to store items in the database, Pod needs to be aware of their types in advance.
This information is stored in a "schema".

### Understanding the schema
The schema is located in `/res/autogenerated_database_schema.json`.
It lists all types that can be stored on Pod, and their properties.

Valid types for properties are, at the moment:

* `Text` UTF-8 string.
* `Integer` Signed 8-byte integer.
* `Real` 8-byte IEEE floating-point number.
* `Bool` Boolean. Internally, booleans are stored as Integers 0 and 1. This is never exposed
to the clients, however, and clients should only ever receive/send `true` and `false`.
* `DateTime` The number of non-leap-milliseconds since 00:00 UTC on January 1, 1970.
Use this database type to denote DateTime.
Internally stored as Integer and should be passed as Integer.

All column definitions of the same case-insensitive name MUST have the same type and indexing.
All column names MUST consist of `a-zA-Z0-9_` characters only, and start with `a-zA-Z`.
All type names MUST consist of `a-zA-Z0-9_` characters only, and start with `a-zA-Z`
(same as column names).

### Changing the schema locally
If you want to make local changes to the schema while developing
new functionality, you can edit the schema directly.
It's located in `/res/autogenerated_database_schema.json`.

Simply re-start the Pod to apply the changes.

### Contributing your schema
The schema is also used in iOS and other projects.
To make it available universally, please submit your schema to the "schema" repository:
[https://gitlab.memri.io/memri/schema](https://gitlab.memri.io/memri/schema).

Changes made to "schema" repository will allow you to generate new definitions
for other projects, and for Pod.
You can copy the newly generated JSON to Pod during development.

You can contribute to the schemas by making a Merge Requests for the "schema" repository.
Please refer to that repo's documentation on how to work with it and do it best.