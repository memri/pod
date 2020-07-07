## About
There are various components that communicate with the Pod:

* Clients like iOS app, web app
* Indexers that enrich data/photos/other content,
* Importers/Downloaders that import information from other systems, etc.

All of that information goes through Pod HTTP API.
This page explains the data types that Pod can store, and current API that shows what you can and cannot do with the Pod.


## Items
Items is the main thing that is stored in Pod.
You could see it as the main holder for Pod-s data,
with edges (see below) being the thing that connects different items to form a graph.

### item's mandatory properties

* `_type`, that can never be changed once set and represents the type of the item
* `uid`, the unique identifier of the item, signed 64-bit integer
* `dateCreated`, creation date stored as DateTime (see [Understanding the schema](../README.md))
* `dateModified`, last modification date **as seen by the client**.
This property is not controlled by the Pod
* `deleted`, a flag that, if set to `true`, will mean that the item was deleted by the user.
It is still possible to restore the item later on. Permanent delete will happen later.
* `version`, a number that is incremented with each update from the client.
This field is fully controlled by the Pod, all input on it will be ignored and it will always
store the real number of updates that happened to an item.

### item's additional properties
Additional properties can be set dynamically via the [Schema](../README.md).


## Edges
Edges connect items together. Pending on design decisions we'll have to make, edges might possibly also support arbitrary properties in the future (again pending on our discussions in the future, don't rely on it yet).

### edge's mandatory properties

* `_source`, the `uid` of the item it points *from*
* `_target`, the `uid` of the item it points *to*. Note that edges are "directed"/"asymmetrical", e.g. _source and _target are not interchangeable
* `_type`, the type of the edge. Cannot be modified once created.

### edge's additional properties (currently hardcoded)
* `label`, an optional string
* `sequence`, an optional integer meaning the client-side ordering of items (e.g. items reachable from a "root" item using edges of a particular _type)


## API endpoints

### GET /version
Get the version of the Pod. In future, we'll also point to a specific git commit here.

### GET /v1/items/{uid}
Get a single item by it's `uid`.

**UNSTABLE**: currently, we return an empty array if an item is not found,
or an array with 1 item if item exists.
In future, we might return an error if item was not found,
and the object itself if the item was found.

### GET /v1/all_items/
Get an array of all items.

### POST /v1/items/
Create a single item.

* Input json MUST have `uid` property
* `version` property of the input will be ignored if set
* If `dateCreated` is not set, it will be set by the backend
* If `dateModified` is not set, it will be set by the backend

Returns `uid` of the created item. Returns an error if an `uid` did already exist.

**UNSTABLE**: We might allow creating items without `uid` being explicitly set, and just return the `uid` to the caller in future.

### PUT /v1/items/{uid}
Update a single item.

* `uid` from the json body will be ignored
* `_type` from the input json will be ignored
* `dateCreated` from the input json will be ignored
* If `dateModified` is not set, it will be set by the backend
* `version` from the input json will be ignored, and in fact will be increased by 1 from previous database value.

### POST /v1/bulk_action/
TODO :(

### DELETE /v1/items/{uid}
Mark an item as deleted.
* Sets `deleted` flag to `true`
* Sets `dateModified` to the database DateTime

### GET /v1/deprecated/uri_exists/{uri}
Check if an item exists with the `uri`.

Returns a `true` successfully if such item exists,
or returns a `false` successfully if such item does not exist.

**DEPRECATED**.

### POST /v1/search_by_fields/
Search items by their fields.
Given a json like
```
{ "author": "Vasili", "_type": "note" }
```
the endpoint will return an array of all items with exactly the same properties.

### GET /v1/item_with_edges/{uid}
Get item, with edges of any type pointing from that item, and all item-s properties that those edges point to.

**UNSTABLE**: Currently, the endpoint will return an array of 1 item (and linked data) when `uid` exists, or an empty array when this `uid` does not exist. In future, we will return the json object itself when the `uid` exists, or return an HTTP failure otherwise.
