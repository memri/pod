## About

Pod is the open-source backend for [Memri](https://memri.io/) project.

It's written in Rust and provides an HTTP interface for use by the clients.

See documentation on:

* Pod-s [HTTP API](./docs/HTTP_API.md)
* Running [Integrators](./docs/Integrators.md)
* What are [Data Collectives](./docs/DataCollectives.md)
* How are data types defined in [Schema](./docs/Schema.md)
* How to run Pod (this document)

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
cargo run -- --help
cargo run -- --owners=ANY
```

Or the easy-to-use development version:
```
./examples/run_development.sh
```


## Pod development
If you develop Pod, you might want to have faster build turn-around.

Use this to incrementally compile the project (after installing [cargo-watch](https://github.com/passcod/cargo-watch)):
```sh
cargo watch --ignore docs -s 'cargo check'
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
