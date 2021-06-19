# Requirements

There are query features that we want to have _eventually_.
We might not be able to implement (or use) all of them right away.

It is our desire that _current_ query language should be compatible with future improvements.
This is not always possible because it's impossible to predict future use,
but at least we want to avoid traps that are already known to not work in the future.

Below are requirements that we already know to exist:

### JSON
For simplicity of early design, we want both requests and responses to be JSON

### Filters
We need to be able to search for properties, e.g. "age is at least 20".
```
{
  "type": "Person"
  "age>=": 20,
}
```

### More filters
We'll need also need more comparisons:

* `!=` (including `null` comparison)
* `==` (including `null` comparison)
* `>=`
* `<=`
* `>`
* `<`

### Response is structurally similar to request
We want the request to "look similar" to the response

<table>
<tr>
<th> Request </th>
<th> Response </th>
</tr>
<tr>
<td>

```json5
{
  "age": 20,
  "friends": ???
}
```

</td>
<td>

```json5
{
  "type": "Person",
  "name": "Someone",
  "age": 20,
  "friends": [
    { "id": "friend1", /* ... */ },
    { "id": "friend2", /* ... */ },
  ],
}
```

</td>
</tr>
</table>

### Property encoding
Return `dateCreated` property in milliseconds (/seconds/hours/days/..)

```
dateCreated(unit: millisecond)
```

### Independent edges
In pseudo-syntax:
```json5
{
  "id": "abcde",
  "[[oldFriends]]": {
    "name": "friend",
    "item": {
      "knownSince<": 1234567
    }
  },
  "[[bestFriends]]": {
    "name": "friend",
    "item": {
      "activity>=": 3
    }
  }
}
```

=>

```json5
{
  "id": "abcde",
  "[[oldFriends]]": [
    // ...
  ],
  "[[bestFriends]]": [
    // ...
  ]
}
```

### Out of scope
Variables, Fragments etc are out of scope for now.


# Solutions

This section is for the Work-In-Progress solution proposal.
It's not final, but it should guide us on _what to do right now_.

### Request all forward edges
The simplest request with edges, get all of them in a search request:

<table>
<tr>
<th> Request </th>
<th> Response </th>
</tr>
<tr>
<td>

```json5
{
  "id": "abcde",
  "[[edges]]": {}
}
```

</td>
<td>

```json5
{
  "id": "abcde",
  "age": 20,
  "name": "Bob",
  "[[edges]]": [
    {
      "_edge": "friend", // edge name
      "order": 1000,
      "item": {
        "id": "this_friend",
        "age": 15,
        // ...
      }
    },
    {
      "_edge": "friend",
      "order": 1001,
      "item": {
        "id": "another_friend",
        "age": 30,
        // ...
      }
    },
    // ...
  ]
  // ...
}
```

</td>
</tr>
</table>

### Future: filter edges
In this example, get friends that are know to you since a specific DateTime ("old friends").

<table>
<tr>
<th> Request </th>
<th> Response </th>
</tr>
<tr>
<td>

```json5
{
  "id": "abcde",
  "[[myOldFriends]]": {
    "_edge": "friend",
    "item": {
      "knownSince>=": 12345,
      // ...
    }
  }
}
```

</td>
<td>

```json5
{
  "id": "abcde",
  "age": 20,
  "name": "Bob",
  "[[myOldFriends]]": [
    // ... all friends with knownSince property of at least 12345
  ]
  // ...
}
```

</td>
</tr>
</table>

## Syntactic sugar
During our discussions on Schema design it was hypothesized that many people
would be interested in simple access to edges.
This means no filtering on edge properties,
and expecting responses that have edges without any properties in them.
(As opposed to, for example, Machine Learning predictions that would have edge properties,
have edges pointing to other edges, have edge labels, etc etc.)

It is thus important to make the simple use case -- simple. Solution proposal:

<table>
<tr>
<th> Request </th>
<th> Response </th>
</tr>
<tr>
<td>

```json5
{
  "id": "abcde",
  "[friend]": {
    "knownSince>=": 12345
  }
}
```

</td>
<td>

```json5
{
  "id": "abcde",
  "age": 20,
  "name": "Bob",
  "[friend]": [
    { "id": "friend1", "knownSince": 12388, /* ... */ },
    { "id": "friend2", "knownSince":  12399, /* ... */ },
    // ... Note that the intermediate edge
    // and its properties are missing in both
    // the request and the response.
  ]
}
```

</td>
</tr>
</table>

(?) All forward and backward edges simultaneously:

<table>
<tr>
<th> Request </th>
<th> Response </th>
</tr>
<tr>
<td>

```json5
{
  "id": "abcde",
  "[[edges]]": {},
  "~[[edges]]": {},
}
```

</td>
<td>

```json5
{
  "id": "abcde",
  "name": "Bob",
  "[[edges]]": [
    { "_edge": "father", "item": { /* ... */ } },
    { "_edge": "friend", "item": { /* ... */ } },
    { "_edge": "friend", "item": { /* ... */ } },
  ],
  "~[[edges]]": [
    { "_edge": "child", "item": { /* ... */ } },
  ],
  // ...
}
```

</td>
</tr>
</table>


(?) All reverse edges without edge properties:

<table>
<tr>
<th> Request </th>
<th> Response </th>
</tr>
<tr>
<td>

```json5
{
  "id": "abcde",
  "~[*]": {}
}
```

</td>
<td>

```json5
{
  "id": "abcde",
  "name": "Bob",
  "~[friend]": [
    { "id": "friend1", /* ... */ },
    { "id": "friend2", /* ... */ },
  ],
  "~[child]": [
    { "id": "father", /* ... */ },
  ],
  // ...
}
```
(Not finalized, might change)

</td>
</tr>
</table>



### Support for today
To see what is already supported by the current version of Pod, see [HTTP_API](./HTTP_API.md).
