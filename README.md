## About

Pod is the open-source backend for [Memri](https://blog.memri.io/) project.

It's written in Rust and provides an HTTP interface for use by the clients.


## Run in docker
To run Pod inside docker:
```sh
docker-compose build
docker-compose up
```


## Local build/install

In order to build Pod locally, you need to install `rust` and `sqlcipher`:

* On Ubuntu: `apt-get install rust libsqlcipher-dev`
* On ArchLinux: `pacman -S --needed rust sqlcipher`
* On MacOS: `brew install rust sqlcipher`

After this, you can build Pod with:
```sh
cargo build --release
```

Or install it with:
```sh
cargo install --force
```


## Development
During development, you might want to have faster build turn-around. Use this to build a debug version:
```sh
cargo build
```

Run the Pod with debug logging (will re-building first if necessary):
```sh
RUST_LOG=pod=debug,info cargo run
```

Incrementally compile the project (after installing cargo-watch):
```sh
cargo watch -x check
```

You can read about various components of the server:

* Memri project: [blog.memri.io](https://blog.memri.io/)
* SQLite: [sqlite.org](https://sqlite.org)
* Rusqlite database driver: [github.com/rusqlite/rusqlite](https://github.com/rusqlite/rusqlite)
* Warp HTTP engine: [github.com/seanmonstar/warp](https://github.com/seanmonstar/warp)
* Rust language: [rust-lang.org](https://www.rust-lang.org/)


## Database
Pod uses SQLite database as its storage mechanism.

When running Pod, a file named `pod.db` will be created.

Note that the current version of Pod **DOES NOT** use encryption.
This part will be changed, and a manual import will be needed in the future.
