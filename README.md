## About

Pod is the open-source backend for [Memri](https://memri.io/) project.

It's written in Rust and provides an HTTP interface for use by the clients.

See documentation on:

* How to run Pod (this document)
* Pod [HTTP API](./docs/HTTP_API.md)
* Writing [Plugins](https://blog.memri.io/getting-started-building-a-plugin/)
* Running [Plugins](./docs/Plugins.md)
* [Security](./docs/Security.md)
* What is a [Shared Server](./docs/SharedServer.md)
* How are data types defined in [Schema](./docs/Schema.md)
* [Schema synchronization](./docs/Synchronization.md) between clients/plugins and the Pod

## Build & Run
There are 3 main ways to run Pod: building it locally/natively, building it in docker,
and using pre-built docker images to just run it.

### Run pre-built docker image of Pod
This is the fastest way to get Pod running on your system,
however it only works for Pod versions that have already been built on our server.  
To use this option:
```
# in this example, we're running Pod version "dev-de929382" ("branch-commit")
POD_VERSION="dev-de929382" docker-compose --file examples/using-prebuilt-docker.yml up
```


### Run in docker
This is the least involved way to build locally. To build&run Pod inside docker:
```sh
docker-compose up --build
```


### Local build/run
This is the fastest way to compile Pod from source,
for example, if you're making any changes in Pod and want to test it.
It will also work on any OS and CPU architecture, it will cache build artifacts whenever possible.

You will need Rust >= 1.45 and sqlcipher:

* On MacOS: `brew install rust sqlcipher`
* On ArchLinux: `pacman -S --needed rust sqlcipher base-devel`
* On Ubuntu and Debian:
```
apt-get install sqlcipher libsqlcipher-dev build-essential
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

After this, you can run Pod with either of these options:
```sh
./examples/run_development.sh
cargo run -- --help
cargo run -- --owners=ANY
cargo run --release -- owners=ANY  # release build, e.g. if you want to test performance
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


## Database
Pod uses SQLite database as its storage mechanism.

When running Pod, a file named `data/db/*.db3` will be created.
You can use the following command to browse the database locally:
```
sqlcipher -cmd "PRAGMA key = \"x'yourDatabaseKey'\";" data/db/*.db3
```
For example, `.schema` will display the current database schema.

If you want to fill the database with some example data, execute
`res/example_data.sql` inside the database.
