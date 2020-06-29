FROM ubuntu:latest as cargo-build

ENV PATH="/root/.cargo/bin:${PATH}"
ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update
RUN apt-get install -y libsqlcipher-dev cargo

WORKDIR /usr/src/pod

# Compile (and docker-cache) dependencies
COPY Cargo.toml Cargo.lock ./
RUN set -x && \
  mkdir -p src && \
  echo "fn main() {println!(\"broken\")}" > src/main.rs && \
  cargo build --release && \
  rm src/main.rs && \
  find target/release/ -type f -executable -maxdepth 1 -delete

# Now that the dependencies are built and cached as a docker layer,
# Copy the sources and build the real thing
COPY res res
COPY src src
RUN cargo build --release


FROM ubuntu:latest
RUN apt-get update
RUN apt-get install -y libsqlcipher-dev
COPY --from=cargo-build /usr/src/pod/target/release/pod pod
EXPOSE 3030
CMD ["./pod"]
