## About

Pod is the open-source backend for [Memri](https://memri.io/) project.

It's written in Rust and provides an HTTP interface for use by the clients.

## WARNING: NOT SECURE WITH PUBLIC IP!!!

The current version of Pod **DOES NOT** guarantee security yet,
**DO NOT** use it for production or run it with a public IP.

* When attempting to run with a public IP, Pod will give an error and refuse to start;
* Setting the environment variable `INSECURE_USE_PUBLIC_IP` to any value
will allow Pod to start even with a public IP (with the security implications above!).

## Run in docker
To run Pod inside docker:
```sh
docker-compose build
docker-compose up
```


## Local build/run

In order to build Pod locally, you need Rust and `sqlcipher`:

* On MacOS: `brew install cargo sqlcipher`
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
