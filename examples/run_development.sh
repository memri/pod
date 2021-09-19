#!/bin/bash -euET
# This is an example of how you might run Pod during development.
# Do not ever attempt to use this script in production!

if ! test -e Cargo.toml; then
  echo "Please run this script from Pod-s main directory."
  exit 1
fi

cargo build

if ! test -v RUST_LOG; then
  export RUST_LOG=pod=debug,info
fi

exec target/debug/pod \
  --owners=ANY \
  --insecure-non-tls=0.0.0.0 \
  "$@"
