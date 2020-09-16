# About
This documentation is part of [Pod](../README.md).

Integrators are various components that can enrich your data,
help you import your data from external services, push some data outside if you want, etc.

This page explains how Pod runs various integrators.

# Running integrators from Pod

### How to trigger
First, the Pod needs to receive a request to run an integrator.
This is done via [HTTP API](./HTTP_API.md).

In the future, it is planned to also support database triggers to execute various integrators.

### What is triggered
Upon receiving an integrator request, Pod will extract the `uid` from the request
and check that item with this uid exists in the database.
Pod will then determine the relevant **docker image**,
and run it with specific environment variables set (see below).

* For Downloaders, docker image `memri-downloaders:latest` will be run
* For Importers, docker image `memri-importers:latest` will be run
* For Indexers, docker image `memri-indexers:latest` will run

### How are integrators started
Integrators are started via **docker**.
Pod will set the following environment variables for integrators running in docker:

* `POD_FULL_ADDRESS`, the address of Pod to call back,
  e.g. `https://x.x.x.x:80` or `http://localhost:3030`.
  You can call the endpoints via a URL like `$POD_FULL_ADDRESS/v2/version`.
* `POD_ADDRESS`, same of the above, but without the scheme and port.
* `RUN_UID`, the item `uid` that the integrator needs to run against.
  This item is commonly the first thing that the integrator requests from the Pod in order
  to understand the task and continue going forward.
* `POD_SERVICE_PAYLOAD`, a JSON that is taken from `servicePayload` from Pod-s HTTP request body,
  and passed-through to the integrator. The JSON is not escaped anyhow, and can be parsed directly.

Additionally, Downloaders and Importers will have a volume `/usr/src/importers/data`
shared with them, so that files can be stored
in that directory by e.g. Downloaders and read by e.g. Importers.
