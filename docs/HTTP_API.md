# About
This documentation is part of [Pod](../README.md).

HTTP API is the interface that Pod provides to store and access user data.
This document explains the data types that Pod can store,
and current API provided for that.


# Items
Items are the main thing that is stored in Pod.
You could see it as the main holder for Pod's data.

### Item's mandatory properties

* `_type`, case-sensitive item's type. Can never be changed once created.
* `uid`, the unique identifier of the item, signed 64-bit integer.
* `dateCreated`, creation date _as seen by the client_, stored as
DateTime (see [Understanding the schema](../README.md#understanding-the-schema)).
Set by the client by default.
* `dateModified`, last modification date _as seen by the client_. Set by the client by default.
* `deleted`, a flag that, if set to `true`, will mean that the item was deleted by the user.
It is still possible to restore the item later on.
Permanent delete will be supported in the future, based in deletion date.
* `version`, a number that is incremented with each update from the client.
This field is fully controlled by the Pod, all input on it will be ignored and it will always
store the real number of updates that happened to an item.

### Item's additional properties
Additional properties can be set dynamically via the [Schema](../README.md#schema).


# Edges
Edges connect items together to form a
[directed graph](https://en.wikipedia.org/wiki/Directed_graph).
Pending on design decisions we're going to make, edges might also possibly
support properties in the future (don't rely on it yet).

### Edge's mandatory properties

* `_source`, the `uid` of the item it points *from*
* `_target`, the `uid` of the item it points *to*
* `_type`, the type of the edge. Cannot be modified once created.

### Edge's additional properties (currently hardcoded)
* `edgeLabel`, an optional string
* `sequence`, an optional integer meaning the client-side ordering of items
(e.g. items reachable from a "root" item using edges of a particular _type)


# API Authentication & Credentials
Some endpoints require additional authentication / credentials.

In text below, `databaseKey` means a 64-character hex string to be used with sqlcipher.
It should be generated by the client once and kept there.
This key will never be written to disk by Pod, and is used to open the database.
You can read more on how this key is used in sqlcipher
[here](https://github.com/sqlcipher/sqlcipher).

In text below, `owner_key` means the full client's public [ED25519](https://ed25519.cr.yp.to/) key
(see also [wikipedia](https://en.wikipedia.org/wiki/Curve25519)).
It should be encoded as 64-character hex string.

It is also used as authentication mechanism in the current version of Pod.
Pod will calculate the `blake2b` hash of the `owner_key` bytes,
and if it matches pre-defined values, will accept the request.
When you need to calculate the hash to send to Pod, you can use one of the libraries:
[javascript](https://github.com/emilbayes/blake2b)
(or the [wasm](https://github.com/jedisct1/libsodium.js) version,
[example](https://github.com/jedisct1/libsodium.js/blob/master/test/sodium_utils.js#L113)),
[swift](https://github.com/jedisct1/swift-sodium/blob/master/Sodium/GenericHash.swift),
[rust](https://crates.io/crates/blake2),
[libsodium](https://doc.libsodium.org/hashing/generic_hashing),
CLI `b2sum --length=256`.
During development, you can also just send any request to the Pod and see it's logs,
which will contain the owner denial along with the expected hash.
Additionally, you can use the word "ANY" for owner list in Pod, which will make Pod accept
requests from any owner -- so called multi-tenancy.

⚠️ UNSTABLE: The use of this key for authentication will be changed in the nearest future.
Note that incorrect database key (below) will also fail any request.


# Items API

### GET /version
Get version of the Pod: the git commit and cargo version that it was built from.


### POST /v2/$owner_key/get_item
```json
{
  "databaseKey": "2DD29CA851E7B56E4697B0E1F08507293D761A05CE4D1B628663F411A8086D99",
  "payload": $uid
}
```
Get a single item by its `uid`.

⚠️ UNSTABLE: currently, the endpoint returns an empty array if an item is not found,
or an array with 1 item if item exists.
In the future, the endpoint might return an error if item was not found,
and the object itself if the item was found.


### POST /v2/$owner_key/get_all_items
```json
{
  "databaseKey": "2DD29CA851E7B56E4697B0E1F08507293D761A05CE4D1B628663F411A8086D99",
  "payload": null
}
```
Get an array of all items.


### POST /v2/$owner_key/create_item
```json
{
  "databaseKey": "2DD29CA851E7B56E4697B0E1F08507293D761A05CE4D1B628663F411A8086D99",
  "payload": { "_type": "...", ... }
}
```
Create a single item.

* `_type` sets the type of the item, cannot ever be changed
* `uid` if set, will be taken as new item's uid;
    otherwise, a new uid will be generated by the database
* `version` from the input json will be ignored
* `dateCreated` if not present, will be set by the backend
* `dateModified` if not present, will be set by the backend

Returns `uid` of the created item. Returns an error if an `uid` did already exist.

⚠️ UNSTABLE: In the future, the endpoint might allow creating items without `uid` being explicitly set,
and just return the `uid` to the caller.


### POST /v2/$owner_key/update_item
```json
{
  "databaseKey": "2DD29CA851E7B56E4697B0E1F08507293D761A05CE4D1B628663F411A8086D99",
  "payload": { "uid": $uid, ... }
}
```
Update a single item.

* `uid` from the input json will be taken as the item's uid
* `_type` from the input json will be ignored
* `dateCreated` from the input json will be ignored
* `dateModified` if not present, will be set by the backend
* `version` from the input json will be ignored,
and instead will be increased by 1 from previous database value.

Returns an empty object if the operation is successful.


### POST /v2/$owner_key/bulk_action/
```json
{
  "databaseKey": "2DD29CA851E7B56E4697B0E1F08507293D761A05CE4D1B628663F411A8086D99",
  "payload": {
    "createItems": [
      { "uid": 12345, "_type": "Person", ... }, ...
    ],
    "updateItems": [
      { "uid": 12345, ... }, ...
    ],
    "deleteItems": [ uid, uid, uid, ...],
    "createEdges": [
      { "_source": uid, "_target": uid, "_type": "AnyString", ... }, ...
    ],
    "deleteEdges": [
      { "_source": uid, "_target": uid, "_type": "Some Type", ... }, ...
    ],
  }
}
```
Perform a bulk of operations in one request.
The endpoint is "atomic", meaning that either all of the operations succeed,
or the database won't be changed at all.

If `createEdges` array is not empty, all items in `createItems` MUST have `uid` set.

Returns an empty object if the operation is successful.


### POST /v2/$owner_key/delete_item
```json
{
  "databaseKey": "2DD29CA851E7B56E4697B0E1F08507293D761A05CE4D1B628663F411A8086D99",
  "payload": $uid
}
```
Mark an item as deleted:
* Set `deleted` flag to `true`
* Update `dateModified` (server's time is taken)


### POST /v2/$owner_key/insert_tree
```json
{
  "databaseKey": "2DD29CA851E7B56E4697B0E1F08507293D761A05CE4D1B628663F411A8086D99",
  "payload": { /* item definition (see below) */ }
}
```
Insert a tree with edges (of arbitrary depth) in one batch.

Each item should either be "a reference" (an object with only `uid` and `_edges` fields),
or a full item which will then be created.

"Reference" objects should look like that
(the `uid` property mandatory, no other properties are present):
```json
{
  "uid": 123456789 /* uid of the item to create edge with */,
  "_edges": [ /* see below edges definition*/ ]
}
```

Items which have other properties besides `uid` and `_edges` will be
considered new and will be created. For example:
```json
{
  "_type": "SomeItemType",
  /* other item properties here */
  "_edges": [ /* see below edges definition*/ ],
}
```

Each edge in the array above should have the following form:
```json
{
  "_type": "SomeEdgeType",
  /* optional edge properties here */
  "_target": { /* item of identical structure to the above */ }
}
```

As always, inserting edges will result in updating timestamps for `_source` items
(even if they are referenced by `uid` only).

The method will return the `uid` of the created root item, e.g. `123456789`.


### POST /v2/$owner_key/search_by_fields/
```json
{
  "databaseKey": "2DD29CA851E7B56E4697B0E1F08507293D761A05CE4D1B628663F411A8086D99",
  "payload": { "_type": "Label", "color": "#CCFF00", "_dateServerModifiedAfter": 123456789, ... }
}
```
Search items by their fields.

Ephemeral underscore field `_dateServerModifiedAfter`, if specified,
is treated specially. It will filter out those items that have
`_dateServerModified` higher (`>`) than the specified value.

The endpoint will return an array of all items with exactly the same properties.


### POST /v2/$owner_key/get_items_with_edges
```json
{
  "databaseKey": "2DD29CA851E7B56E4697B0E1F08507293D761A05CE4D1B628663F411A8086D99",
  "payload": [1, 20, 30, 100000, ...]
}
```
Given an input array of `uid`-s, for each `uid`:

* find the underlying item
* within each item, find all "outgoing" edges
* for each edge, attach the target item's properties

If at least one input `uid` doesn't exist, return 404 NOT_FOUND for the whole request.


# Services API
Services help getting data into your Pod and enriching it.
Services can only be ever run / authorized to run by the user.
Typical examples of services are services that import emails/messages into Pod.

### POST /v2/$owner_key/run_downloader
```json
{
  "databaseKey": "2DD29CA851E7B56E4697B0E1F08507293D761A05CE4D1B628663F411A8086D99",
  "payload": {
    "uid": $uid,
    "servicePayload": {
      "databaseKey": "2DD29CA851E7B56E4697B0E1F08507293D761A05CE4D1B628663F411A8086D99",
      "ownerKey": $owner_key
    }
  }
}
```
Run a downloader on an item with the given uid.
See [Integrators](./Integrators.md).

⚠️ UNSTABLE: Downloaders might be merged with importers soon.


### POST /v2/$owner_key/run_importer
```json
{
  "databaseKey": "2DD29CA851E7B56E4697B0E1F08507293D761A05CE4D1B628663F411A8086D99",
  "payload": {
    "uid": $uid,
    "servicePayload": {
      "databaseKey": "2DD29CA851E7B56E4697B0E1F08507293D761A05CE4D1B628663F411A8086D99",
      "ownerKey": $owner_key
    }
  }
}
```
Run an importer on an item with the given uid.
See [Integrators](./Integrators.md).


### POST /v2/$owner_key/run_indexer
```json
{
  "databaseKey": "2DD29CA851E7B56E4697B0E1F08507293D761A05CE4D1B628663F411A8086D99",
  "payload": {
    "uid": $uid,
    "servicePayload": {
      "databaseKey": "2DD29CA851E7B56E4697B0E1F08507293D761A05CE4D1B628663F411A8086D99",
      "ownerKey": $owner_key
    }
  }
}
```
Run an indexer on an item with the given uid.
See [Integrators](./Integrators.md).


# File API


### POST /v2/$owner_key/upload_file/$database_key/$sha256hashOfTheFile
```text
RAW-FILE-BINARY
```
Upload a file into Pod and verify it's `sha256`.
`owner_key`, database key and sha256 are all hex-encoded.

If a file with a given `sha256` has already being uploaded, the request will fail.
If the provided `sha256` doesn't match the hash of the contents, the request will fail.
If no item with this `sha256` exists in the database, Pod wouldn't be able to store
cryptographic information about the file, and the request will also fail.

If `sha256` matches, the file has not yet been uploaded to Pod and if an item
with such `sha256` already exists in DB, Pod will accept the file and store it.
The fields `nonce` and `key` will be updated for this item.


### POST /v2/$owner_key/get_file
```json
{
  "databaseKey": "2DD29CA851E7B56E4697B0E1F08507293D761A05CE4D1B628663F411A8086D99",
  "payload": {
    "sha256": $sha256
  }
}
```
Get a file by its sha256 hash.
If the file does not yet exist in Pod, a 404 NOT FOUND error will be returned.
