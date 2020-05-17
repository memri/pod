FROM ubuntu:latest as cargo-build

ENV PATH="/root/.cargo/bin:${PATH}"
ENV DEBIAN_FRONTEND=noninteractive

RUN apt update
RUN apt install -y build-essential cmake curl golang
RUN apt update

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

WORKDIR /usr/src/pod
COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml
COPY Settings.toml Settings.toml
COPY src src

RUN cargo build --release


FROM ubuntu:latest
COPY --from=cargo-build /usr/src/pod/target/release/pod pod
COPY Settings.toml Settings.toml
EXPOSE 3030
CMD ["./pod"]
