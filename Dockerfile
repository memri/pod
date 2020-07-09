FROM ubuntu:latest as cargo-build

ENV PATH="/root/.cargo/bin:${PATH}"
ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update
RUN apt-get install -y libsqlcipher-dev cargo

WORKDIR /usr/src/pod

# Compile dependencies and cache them in a docker layer
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN set -x && \
  mkdir -p src && \
  echo "fn main() {println!(\"broken\")}" > src/main.rs && \
  cargo build --release && \
  rm src/main.rs && \
  find target/release/ -type f -executable -maxdepth 1 -delete

# After the dependencies are built, copy the sources and build the real thing.
COPY res res
COPY build.rs build.rs
COPY src src
RUN cargo build --release && mv target/release/pod ./ && rm -rf target


FROM ubuntu:latest
RUN apt-get update
RUN apt-get install -y libsqlcipher-dev
COPY --from=cargo-build /usr/src/pod/pod pod
EXPOSE 3030
CMD ["./pod"]
