## About

Pod is the open-source backend for [Memri](https://memri.io/) project.

It's written in Rust and provides an HTTP interface for use by the clients.


## Run in docker
To run Pod inside docker:
```sh
docker-compose up --build
```


## Local build/run

In order to build Pod locally, you need Rust and sqlcipher:

* On MacOS: `brew install rust sqlcipher`
* On ArchLinux: `pacman -S --needed rust sqlcipher`
* On Ubuntu and Debian:
```
apt-get install libsqlcipher-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
* Submit merge requests for your other OS :)

After this, you can run Pod with:
```sh
cargo run --release
```


## Development
If you develop Pod, you might want to have faster build turn-around.

Use this to incrementally compile the project (after installing [cargo-watch](https://github.com/passcod/cargo-watch)):
```sh
cargo watch --ignore docs
```

To build (debug version):
```sh
cargo build
```

Run:
```sh
RUST_LOG=pod=debug,info cargo run
```

You can read about various components of the server:

* Memri project: [blog.memri.io](https://blog.memri.io/)
* SQLite: [sqlite.org](https://sqlite.org)
* Sqlcipher: [zetetic.net/sqlcipher](https://www.zetetic.net/sqlcipher/open-source/)
* Rusqlite database driver: [github.com/rusqlite/rusqlite](https://github.com/rusqlite/rusqlite)
* Warp HTTP engine: [github.com/seanmonstar/warp](https://github.com/seanmonstar/warp)
* Rust language: [rust-lang.org](https://www.rust-lang.org/)


## HTTP API
Pod's API is documented in detail [here](./docs/HTTP_API.md).


## Database
Pod uses SQLite database as its storage mechanism.

When running Pod, a file named `data/db/*.db` will be created.
You can use the following command to browse the database locally:
```
sqlcipher -cmd "PRAGMA key = \"x'yourDatabaseKey'\";" data/db/*.db
```
For example, `.schema` will display the current database schema.

If you want to fill the database with some example data, execute
`res/example_data.sql` inside the database.


## Schema
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
All column names MUST consist of `a-zA-Z_` characters only, and start with `a-zA-Z`.
All type names MUST consist of `a-zA-Z_` characters only, and start with `a-zA-Z`
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
