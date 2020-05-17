# pod

Backend for Memri project.

Thanks for checking this, we are currently in the process of enabling the community to join in and co-create with us.
Can't wait? Reach out to us via our [Slack](https://app.slack.com/client/TSSDHE1JN/CT4PAP7FE)

## How to build and run backend pod with dgraph?

### On local machine:


*  To build the backend on local mahine

`cargo build --release` 


*  To run the backend

`./target/release/pod`


### With docker:


* ####  Dgraph


*  To run a dgraph instance in a docker container

`docker run --rm -it -p 8000:8000 -p 8080:8080 -p 9080:9080 --network my-net --name pod_dgraph_1 dgraph/standalone:latest`

or use the script

`./tools/start-dgraph.sh`

**Note:**

Option `--network` indicates the dgraph container belongs to the network `my-net` with a name `pod_dgraph_1` specified by `--name`. 


* ####  pod



*  To build a docker image of the pod

`docker build -f Dockerfile -t pod .`

or use script

`./build-pod.sh`


*  To run the pod image in a container

`docker run --rm -it -p 3030:3030 -e SCHEMA_SET=true -e DGRAPH_HOST=pod_dgraph_1 --network my-net --name pod_pod_1 pod:latest`

or use script

`./tools/start-pod.sh`

**Note:**

The pod container belongs to the same `my-net` network as the dgraph container and connects to the latter one via its internal hostname.

Available environment variables:
*  `DGRAPH_HOST`, the hostname of dgraph container, by default `pod_dgraph_1`.
*  `SCHEMA_SET`, set up schema, by default `false`, set it to `true` when start running the pod.
*  `SCHEMA_DROP`, drop schema, by default `false`.


### With docker-compose:


*  To build and run both dgraph and pod containers at once 

`docker-compose up`


*  To remove all containers

`docker-compose down`


*  To rebuild the pod image

`docker-compose build`


## How to access dgraph and pod APIs?

Use http://localhost:8000 to access dgraph web UI

Use http://0.0.0.0:3030/v1 to access pod APIs
