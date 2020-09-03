#!/bin/bash -euET
# This is an example of how you might run Pod during development.
# Do not ever attempt to ever use this script in production!

if ! test -e Cargo.toml; then
  echo "Please run this script from Pod-s main directory."
  exit 1
fi

RUST_LOG=pod=debug,info \
  exec cargo run -- \
  --owners=ANY \
  --insecure-non-tls=0.0.0.0 \
  --insecure-http-headers \
  "$@"
