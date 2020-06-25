FROM ubuntu:latest as cargo-build

ENV PATH="/root/.cargo/bin:${PATH}"
ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update
RUN apt-get install -y build-essential curl libsqlcipher-dev

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

WORKDIR /usr/src/pod
COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml
COPY src src

RUN cargo build --release


FROM ubuntu:latest
RUN apt-get update
RUN apt-get install -y libsqlcipher-dev
COPY --from=cargo-build /usr/src/pod/target/release/pod pod
EXPOSE 3030
CMD ["./pod"]
