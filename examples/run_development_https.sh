#!/bin/bash -euET
# This is an example of how you might run Pod during development.
# Do not ever attempt to ever use this script in production!

if !test -e Cargo.toml; then
  echo "Please run this script from Pod-s main directory."
  exit 1
fi

examples/generate_self-signed_certificate.sh

POD_OWNER_HASHES=ANY \
  POD_HTTPS_CERTIFICATE=./data/certs/pod \
  INSECURE_HTTP_HEADERS=1 \
  RUST_LOG=pod=debug,info exec cargo run
