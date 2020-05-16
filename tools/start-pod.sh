docker run --rm -it -p 3030:3030 -e SCHEMA_SET=true -e DGRAPH_HOST=pod_dgraph_1 --network my-net --name pod_pod_1 pod:latest
