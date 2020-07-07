# About
There are various components that communicate with the Pod:

* Clients like iOS app, web app;
* Indexers that enrich data/photos/other content;
* Importers/Downloaders that import data from other systems, e.g. from evernote.

All of that data goes through Pod HTTP API.
This page explains the data types that Pod can store, and current API.


## Items
Items are the main thing that is stored in Pod.
You could see it as the main holder for Pod-s data.

### item's mandatory properties

* `_type`, case-sensitive item's type. Can never be changed once created.
* `uid`, the unique identifier of the item, signed 64-bit integer.
* `dateCreated`, creation date _as seen by the client_, stored as
DateTime (see [Understanding the schema](../README.md#understanding-the-schema)).
Set by the client by default.
* `dateModified`, last modification date _as seen by the client_. Set by the client by default.
* `deleted`, a flag that, if set to `true`, will mean that the item was deleted by the user.
It is still possible to restore the item later on.
Permanent delete will be supported in future, based in deletion date.
* `version`, a number that is incremented with each update from the client.
This field is fully controlled by the Pod, all input on it will be ignored and it will always
store the real number of updates that happened to an item.

### item's additional properties
Additional properties can be set dynamically via the [Schema](../README.md#schema).


## Edges
Edges connect items together to form a
[directed graph](https://en.wikipedia.org/wiki/Directed_graph).
Pending on design decisions we're going to make, edges might also possibly
support properties in the future (don't rely on it yet).

### edge's mandatory properties

* `_source`, the `uid` of the item it points *from*
* `_target`, the `uid` of the item it points *to*
* `_type`, the type of the edge. Cannot be modified once created.

### edge's additional properties (currently hardcoded)
* `label`, an optional string
* `sequence`, an optional integer meaning the client-side ordering of items
(e.g. items reachable from a "root" item using edges of a particular _type)


# API

### GET /version
Get the version of the Pod. In future, it will also point to a specific git commit.

### GET /v1/items/{uid}
Get a single item by it's `uid`.

⚠️ UNSTABLE: currently, the endpoint returns an empty array if an item is not found,
or an array with 1 item if item exists.
In future, the endpoint might return an error if item was not found,
and the object itself if the item was found.

### GET /v1/all_items/
Get an array of all items.

### POST /v1/items/
Create a single item.

* `uid` property MUST be present in input json
* `version` from the input json will be ignored,
* `dateCreated` if not present, will be set by the backend
* `dateModified` if not present, will be set by the backend

Returns `uid` of the created item. Returns an error if an `uid` did already exist.

⚠️ UNSTABLE: In future, the endpoint might allow creating items without `uid` being explicitly set,
and just return the `uid` to the caller.

### PUT /v1/items/{uid}
Update a single item.

* `uid` from the json body will be ignored
* `_type` from the input json will be ignored
* `dateCreated` from the input json will be ignored
* `dateModified` if not present, will be set by the backend
* `version` from the input json will be ignored,
and in fact will be increased by 1 from previous database value.

Returns an empty array if the operation is successful.

### POST /v1/bulk_action/
Perform a bulk of operations atomically.

Example input json:
```json
{
  "create_items": [ { /* structure identical to the create endpoint */ } ],
  "update_items": [ { /* structure identical to the update endpoint */ } ],
  "delete_items": [ uid, uid, uid, ...],
  "create_edges": [
    { "_source": uid, "_target": uid, "_type": "AnyString", /* other properties can be set */ },
    ...
  ],
}
```

Returns an empty array if the operation is successful.

### DELETE /v1/items/{uid}
Mark an item as deleted by:
* Setting `deleted` flag to `true`
* Updating `dateModified` (server-s time is taken)

### GET /v1/deprecated/uri_exists/{uri}
⚠️ DEPRECATED Check if an item exists with the `uri`.

Returns `true` successfully if such item exists,
or returns `false` successfully if such item does not exist.

### POST /v1/search_by_fields/
Search items by their fields.
Given a json like
```
{ "_type": "Label", "color": "#CCFF00" }
```
the endpoint will return an array of all items with exactly the same properties.

### GET /v1/item_with_edges/{uid}
Get item, with edges of any type pointing from that item,
and all item's properties that those edges point to.

⚠️ UNSTABLE: Currently, the endpoint will return
an array of 1 item (and linked data) when `uid` exists,
or an empty array when this `uid` does not exist.
In future, the endpoint might return the json object itself when the `uid` exists,
or return an HTTP failure otherwise.
