# About

Pod is the backend for Memri project.

It's written in Rust, communicates with [dgraph](https://dgraph.io/) database internally and
provides an HTTP interface for use by the clients.

# Preparation

Make sure you have [docker](https://www.docker.com/) installed on your local machine.

# Easiest way to start pod with dgraph

### With docker-compose:

* Run both dgraph and pod containers at once (will build the pod image first if it doesn't exist)

```
docker-compose up
```

* Add testing data of note type

```
cd tools
./add-notes.sh
```

* Stop both containers

```
docker-compose stop
```

* Remove all containers

```
docker-compose down
```

**Note:**

* To have an overview of added data, in the dgraph web UI [http://localhost:8000](http://localhost:8000), use the following query

```
{
  q(func: type(Note)) {
    uid
    dgraph.type
    expand(_all_) {
      uid
      dgraph.type
      expand(_all_)
    }
  }
}
```


* Additionally, you can restart both containers before they are removed

```
docker-compose restart
```

* Rebuild the pod image after code updates

```
docker-compose build
```


# Access dgraph and pod APIs

Use [http://localhost:8000](http://localhost:8000) to access dgraph web UI

Use http://0.0.0.0:3030/v1 to access pod APIs


# Alternative ways to build and run pod with dgraph

You can manually start the containers for development and testing, or simply run the pod on your local machine. 

Either way, the dgraph instance is running inside a container.

### Start dgraph container

* Create a custom network

```
docker network create my-net
```

*  Run a dgraph instance in a docker container

```
docker run -it -p 8000:8000 -p 8080:8080 -p 9080:9080 --network my-net --name pod_dgraph_1 dgraph/standalone:latest
```

or use the script

```
./tools/start-dgraph.sh
```

**Note:**

Option `--network` indicates the dgraph container belongs to the network `my-net` with a name `pod_dgraph_1` specified by `--name`. 

`--rm` option can be added to `docker run` to directly remove the container once it is stopped. However, for a dgraph container, all data and schema will get lost at stop.

* Additionally, restart a stopped dgraph container

```
docker restart pod_dgraph_1
```

* Remove the stopped containers

```
docker container prune
```

* Add testing data of note type

```
cd tools
./add-notes.sh
```

### Run pod in a container

* Build the pod image

```
docker build -f Dockerfile -t pod .
```

or use script

```
./build-pod.sh
```

* Run the pod image in a container

```
docker run --rm -it -p 3030:3030 -e ADD_SCHEMA_ON_START=true -e DGRAPH_HOST=pod_dgraph_1:9080 --network my-net --name pod_pod_1 -v download-volume:/data pod:latest
```

or use script

```
./tools/start-pod.sh
```

**Note:**

The pod container belongs to the same `my-net` network as the dgraph container and connects to the latter one via its internal hostname.

Available environment variables:
*  `DGRAPH_HOST`, the hostname of dgraph container, defaults to `pod_dgraph_1:9080`.
*  `ADD_SCHEMA_ON_START`, add Dgraph schema when starting the server. Defaults to `false`.
*  `DROP_SCHEMA_AND_ALL_DATA`, drop Dgraph schema and ALL underlying data, defaults to `false`.
*  `RUST_LOG=debug`, show all logs at `debug` level, default level is `info`.
*  `IMPORT_NOTES_EVERNOTE`, import notes from Evernote, default to `false`.
*  `IMPORT_NOTES_ICLOUD`, import notes from iCloud, defautl to `false`.


### Run pod on local machine

*  Build the pod on local mahine

```
cargo build --release
``` 

*  Run the pod

```
./target/release/pod
```


# Run integration tests

1. Start a new dgraph container without adding schema or data

2. At command line, run 
```
cargo test -- --test-threads 1
```
