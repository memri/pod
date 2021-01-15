# Development notes

This document is partially here for historical reasons.
It might be removed in the future (or improved and kept).


## Requirements for DB:

When re-architecting for a database that supports
dynamic schemas and heterogeneous types,
the following requirements determine the available choices:

* cascade delete (delete all references to an object)
* quick get (get item properties from text ID)
* quick filter (get all items with "age > 80", "dateServerModified > 1000")
* both items and edges can reference items
* (bonus) both items and edges can reference edges
* both items and edges can have "scalar" properties (raw values)


## Workflows:

### Insert item {"id": "test", "age": 80}

* create object, get its rowid ?1
* put property "id"
* put property "age"

### Insert item {id: "test", friends: [{id: "fr"}]}

* create "fr" item
    - create object for "fr", get its rowid ?1
    - put property "id" for *1
* create object for "test", get its rowid ?2
* put property "id" for *2
* put reference "friends" *1 -> *2

### Search items {"author": "Someone", "yearPublished" > 2000}

This type of search queries will be slow in all of the considered DB layouts.
It is slow in the currently planned DB layout for Pod as well.

We might introduce compound search indexes in the future,
but for now and for early 2021 this is not the case.

### Search items {"author": "Someone", "dateServerModified" > ...}

A query that uses `dateServerModified` specifically,
plus up to one additional property, can be fast in the proposed DB layout.
Steps:

* Query `integers` table for lowest `item` value with scalar `name = "dateServerModified"`. 
  Store result as `r1`.
* Query `integers` table for `name = "author"`, `value = "Someone"`, `item > ?r1`

?????????????????? ??????? ?????????????????????????????????    ????? ????? ????????

### Synchronization requests from the clients

Sync is implemented by running specific API requests and analyzing the results.
For example, a client might sync all photos by querying for
```
{ "type" = "Photo", "dateServerModified" > "2021-01-13"}   (pseudo-syntax)
```
If the API makes these operations fast, the results will also be fast.




# TODO

### Questions to Toby:
* we didn't implement edge properties, item "references" and edge "references"?
* items.id is not optimized for efficient joins,
  we should've used `rowid` as the PRIMARY KEY for efficiency reasons
