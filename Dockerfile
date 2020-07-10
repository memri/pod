#### The image base should be the same as in .gitlab-ci.yml
#### Second stage of the Dockerfile (below) should use the same base (rust:slim or it's parent)
FROM rust:slim as cargo-build

ENV PATH="/root/.cargo/bin:${PATH}"
ENV DEBIAN_FRONTEND=noninteractive
WORKDIR /usr/src/pod


#### Compile dependencies and cache them in a docker layer

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN apt-get update && apt-get install -y libsqlcipher-dev git
RUN set -x && \
  mkdir -p src && \
  echo "fn main() {println!(\"broken\")}" > src/main.rs && \
  cargo build --release && \
  rm src/main.rs && \
  find target/release/ -type f -executable -maxdepth 1 -delete


#### After the dependencies are built, copy the sources and build the real thing.

COPY res res
COPY build.rs build.rs
COPY src src
COPY .git .git
RUN cargo build --release && mv target/release/pod ./ && rm -rf target


#### After Pod has been built, create a small docker image with just the Pod

FROM debian:buster-slim
COPY --from=cargo-build /usr/src/pod/pod pod
RUN apt-get update && apt-get install -y libsqlcipher-dev docker.io
EXPOSE 3030
CMD ["./pod"]
