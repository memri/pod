FROM ubuntu:latest

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

RUN cargo build

EXPOSE 3030

CMD ["./target/debug/pod"]
