#### The image base should be the same as in .gitlab-ci.yml
#### Second stage of the Dockerfile (below) should use the same base (rust:slim or it's parent)
FROM rust:slim as cargo-build

WORKDIR /usr/src/pod
RUN DEBIAN_FRONTEND=noninteractive apt-get update && apt-get install -y docker.io && rm -rf /var/lib/apt/lists/*


#### Compile dependencies and cache them in a docker layer

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN DEBIAN_FRONTEND=noninteractive apt-get update && apt-get install -y libsqlcipher-dev && rm -rf /var/lib/apt/lists/*
RUN set -x && \
  mkdir -p src && \
  echo "fn main() {println!(\"broken\")}" > src/main.rs && \
  mkdir -p benches && \
  echo "" > benches/rusqlite_reconnection.rs && \
  cargo build --release && \
  rm src/main.rs && \
  find target/release/ -type f -executable -maxdepth 1 -delete


#### After the dependencies are built, copy the sources and build the real thing.

COPY res/migrations res/migrations
COPY build.rs build.rs
COPY src src
COPY benches benches
RUN cargo build --release && mv target/release/pod ./ && rm -rf target


#### After Pod has been built, create a small docker image with just the Pod

FROM debian:buster-slim
COPY --from=cargo-build /usr/bin/docker /usr/bin/docker
COPY --from=cargo-build /usr/src/pod/pod pod
RUN DEBIAN_FRONTEND=noninteractive apt-get update && apt-get install -y libsqlcipher-dev curl && rm -rf /var/lib/apt/lists/*
ARG use_kubernetes=false
RUN if [ "$use_kubernetes" = "true" ] ; then \
      curl -LO https://storage.googleapis.com/kubernetes-release/release/$(curl -s https://storage.googleapis.com/kubernetes-release/release/stable.txt)/bin/linux/amd64/kubectl \
      && chmod +x ./kubectl  \
      &&  mv ./kubectl /usr/local/bin/kubectl ; fi

# Check that library versions match (sqlcipher, libc, etc)
RUN ./pod --version 1>/dev/null 2>&1

EXPOSE 3030
CMD ["./pod"]
