# About
This documentation is part of [Pod](../README.md).

In order to store items in the database, Pod needs to be aware of their types in advance.
This information is stored in a "schema".

[comment]: <> (There are three types of information in the Pod:)
[comment]: <> (* Items. They are the main thing that is stored.)
[comment]: <> (* Edges that connect link items to each other.)


## Items
Items are the main thing that is stored in Pod.
You could see it as the main holder for Pod's data.

### Item's mandatory properties

* `type`, case-sensitive item's type. Can never be changed once created.
* `id`, the unique identifier of the item, signed 64-bit integer.
* `dateCreated`, creation date _as seen by the client_, stored as
  DateTime.
  Set by the client by default, or, if missing, by Pod.
* `dateModified`, last modification date _as seen by the client_.
  Updated by the client by default, or, if missing, by Pod.
* `deleted`, a flag that, if set to `true`, will mean that the item was deleted by the user.
  It is still possible to restore the item later on.
  Permanent delete will be supported in the future, based in deletion date.

### Item's additional properties
Additional properties can be set dynamically via the [Schema API](../HTTP_API.md#schema_api).


## Edges
Edges connect items together to form a
[directed graph](https://en.wikipedia.org/wiki/Directed_graph).


### Edge's mandatory properties
* `_source`, the `id` of the item it points *from*
* `_target`, the `id` of the item it points *to*
* `name`, the name of the edge. Cannot be modified once created.
* all mandatory item's properties

### Edge's additional properties
Same as for items, additional properties can be set dynamically
via the [Schema API](../HTTP_API.md#schema_api).


### Understanding the schema
The Schema lists all types that can be stored in Pod, and their properties.

Valid types for properties are, at the moment:

* `Text` UTF-8 string.
* `Integer` Signed 8-byte integer.
* `Real` 8-byte IEEE floating-point number.
* `Bool` Boolean. Internally, booleans are stored as Integers 0 and 1. This is never exposed
to the clients, however, and clients should only ever receive/send `true` and `false`.
* `DateTime` The number of non-leap-milliseconds since 00:00 UTC on January 1, 1970.
Use this database type to denote DateTime.
Internally stored as Integer and should be passed as Integer.

All properties of the same case-insensitive name MUST have the same type and indexing.
All property names MUST consist of `a-zA-Z0-9_` characters only, and start with `a-zA-Z`.
All type names MUST consist of `a-zA-Z0-9_` characters only, and start with `a-zA-Z`
(same as column names).

### Contributing your schema
TODO: the way of working with Schema has changed and is not fully documented yet.
Refer to the [HTTP_API](./HTTP_API.md) to see how you can change the Schema for now.
