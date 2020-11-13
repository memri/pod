#!/usr/bin/env bash
set -euETo pipefail
#### This script tests that Pod can respond to GET /version request


#### Start Pod
./examples/run_development.sh --port=4956 &
pid=$!

for attempt in {1..30}; do
  #### Try to get a successful response to /version
  if curl localhost:4956/version 1>/dev/null 2>&1; then
    echo "Got a successful response, exiting"
    kill $pid
    exit 0
  else
    sleep 1s
  fi
done

echo "Failed to get a response from Pod"
exit 1
