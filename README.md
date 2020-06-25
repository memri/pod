## About

Pod is the open-source backend for [Memri](https://blog.memri.io/) project.

It's written in Rust and provides an HTTP interface for use by the clients.

## Dependencies

In order to build Pod, you need to install `sqlcipher`.

On Ubuntu: `apt-get install sqlcipher`

On ArchLinux: `pacman -S --needed sqlcipher`

## Install
To install Pod locally, you can use the default rust cargo mechanism:
```sh
RUSTUP_TOOLCHAIN=stable cargo install --force
```

## Build
Alternative to the above, if you want to build Pod on one x86_64 Linux machine
to be executed later on another x86_64 Linux machine, use this:

```sh
cargo build --release --target=x86_64-unknown-linux-musl
```

The compiled binary should be around 8Mb (4Mb if compiled with LTO),
and placed in `target/x86_64-unknown-linux-musl/release/pod`.

It is runnable by just executing it. You can read more on Rust and MUSL [here](https://doc.rust-lang.org/edition-guide/rust-2018/platform-and-target-support/musl-support-for-fully-static-binaries.html).

## Development
During development, you might want to have faster build turn-around. Use this to build a debug version:
```sh
cargo build
```

Or this to incrementally compile the project (after installing cargo-watch):
```sh
cargo watch -x check
```

You can read about various components of the server:

* Memri project: [blog.memri.io](https://blog.memri.io/)
* SQLite: [sqlite.org](https://sqlite.org)
* Rusqlite database driver: [github.com/rusqlite/rusqlite](https://github.com/rusqlite/rusqlite)
* Warp HTTP engine: [github.com/seanmonstar/warp](https://github.com/seanmonstar/warp)
* Rust language: [rust-lang.org](https://www.rust-lang.org/)
