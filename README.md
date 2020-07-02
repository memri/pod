## About

Pod is the open-source backend for [Memri](https://blog.memri.io/) project.

It's written in Rust and provides an HTTP interface for use by the clients.

## WARNING: NOT SECURE WITH PUBLIC IP!!!

The current version of Pod **DOES NOT** guarantee security yet, **DO NOT** use it for production or run it with a public IP.

* When attached with a public IP, on start, Pod will give an error and abort;
* Forcing a public IP requires environment variable `FORCE_SUPER_INSECURE=1` and will receive a warning.

## Run in docker
To run Pod inside docker:
```sh
docker-compose build
docker-compose up
```


## Local build/install

In order to build Pod locally, you need to install `rust` and `sqlcipher`:

* On MacOS: `brew install cargo sqlcipher`
* On Ubuntu: `apt-get install cargo libsqlcipher-dev`
* On ArchLinux: `pacman -S --needed rust sqlcipher`
* Submit pull requests for your other OS :)

After this, you can build Pod with:
```sh
cargo build --release
```

Or install it with:
```sh
cargo install --force
```


## Development
During development, you might want to have faster build turn-around.

Use this to incrementally compile the project (after installing [cargo-watch](https://github.com/passcod/cargo-watch)):
```sh
cargo watch
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
* Rusqlite database driver: [github.com/rusqlite/rusqlite](https://github.com/rusqlite/rusqlite)
* Warp HTTP engine: [github.com/seanmonstar/warp](https://github.com/seanmonstar/warp)
* Rust language: [rust-lang.org](https://www.rust-lang.org/)


## Database
Pod uses SQLite database as its storage mechanism.

When running Pod, a file named `pod.db` will be created. You can use `sqlite3 pod.db` to browse the database locally. For example, `.schema` will display the current database schema.

Note that the current version of Pod **DOES NOT** use encryption.
This part will be changed, and a manual import will be needed in the future.
