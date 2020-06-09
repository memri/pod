docker run --rm -it -p 3030:3030 -e ADD_SCHEMA_ON_START=true -e DGRAPH_HOST=pod_dgraph_1:9080 --network my-net --name pod_pod_1 -v download-volume:/data pod:latest
