#!/bin/bash -euET
# You can use this to generate a self-signed certificate for Pod during development

if !test -e Cargo.toml; then
  echo "Please run this script from Pod-s main directory."
  exit 1
fi

if test -f data/pod.crt && test -f data/pod.key; then
  echo "Skipping certificate generation as cert already exists"
  exit 0
fi

exec openssl req -x509 -newkey rsa:4096 -keyout data/pod.key -out data/pod.crt -days 365 -nodes -subj "/C=US/ST=Oregon/L=Portland/O=Company Name/OU=Org/CN=www.example.com"

