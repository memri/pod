# About

Pod is the backend for Memri project.
It's written in Rust, communicates with `dgraph` internally and
provides an HTTP interface for use by the clients.

## How to build and run backend pod with dgraph?

### Preparation:

Make sure you have `docker` installed on your local machine. 

### On local machine:


* To run a dgraph instance in a docker container

`docker network create my-net`

`./tools/start-dgraph.sh`

*  To build the backend on local mahine

`cargo build --release` 

*  To run the backend

`./target/release/pod`


### With docker:

* ####  Dgraph

* To create a custom network

`docker network create my-net`

*  To run a dgraph instance in a docker container

`docker run -it -p 8000:8000 -p 8080:8080 -p 9080:9080 --network my-net --name pod_dgraph_1 dgraph/standalone:latest`

or use the script

`./tools/start-dgraph.sh`

* To restart a stopped dgraph container

`docker restart pod_dgraph_1`

* To remove the stopped containers

`docker container prune`

**Note:**

Option `--network` indicates the dgraph container belongs to the network `my-net` with a name `pod_dgraph_1` specified by `--name`. 

`--rm` option can be added to `docker run` to directly remove the container once it is stopped. However, for a dgraph container, all data and schema will get lost at stop.

* To add testing data of `note` type

`cd tools`

`./add-notes.sh`

**Note:**

To have an overview of added data, in the dgraph web UI, use the following query

`{
  q(func: type(note)) {
    uid
    dgraph.type
    expand(_all_) {
      uid
      dgraph.type
      expand(_all_)
    }
  }
}`

* ####  pod



*  To build a `docker` image of the pod

`docker build -f Dockerfile -t pod .`

or use script

`./build-pod.sh`


*  To run the pod image in a container

`docker run --rm -it -p 3030:3030 -e ADD_SCHEMA_ON_START=true -e DGRAPH_HOST=pod_dgraph_1:9080 --network my-net --name pod_pod_1 pod:latest`

or use script

`./tools/start-pod.sh`

**Note:**

The pod container belongs to the same `my-net` network as the dgraph container and connects to the latter one via its internal hostname.

Available environment variables:
*  `DGRAPH_HOST`, the hostname of dgraph container, defaults to `pod_dgraph_1:9080`.
*  `ADD_SCHEMA_ON_START`, add Dgraph schema when starting the server. Defaults to `false`.
*  `DROP_SCHEMA_AND_ALL_DATA`, drop Dgraph schema and ALL underlying data, defaults to `false`.
*  `RUST_LOG=debug`, show all logs at `debug` level, default level is `info`.


### With docker-compose:


*  To build and run both dgraph and pod containers at once 

`docker-compose up`

* To stop both containers

`docker-compose stop`

* To restart both containers

`docker-compose restart`

*  To remove all containers

`docker-compose down`

*  To rebuild the pod image

`docker-compose build`


## How to access dgraph and pod APIs?

Use http://localhost:8000 to access dgraph web UI

Use http://0.0.0.0:3030/v1 to access pod APIs


## How to run tests?

1. Start a dgraph container without adding schema or data

2. At command line, run `cargo test -- --test-threads 1`