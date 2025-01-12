# Always build against latest stable
ARG RUST_VERSION=1.83-slim
FROM rust:${RUST_VERSION} as builder

RUN rustup component add clippy rustfmt

# Install tool dependencies for app and git/ssh for the workspace
RUN apt-get update && apt-get install -y --no-install-recommends \
  ripgrep fd-find git ssh curl  \
  protobuf-compiler \
  libprotobuf-dev \
  pkg-config libssl-dev iputils-ping \
  make \

RUN cargo install cargo-llvm-cov cargo-nextest

COPY . /app

WORKDIR /app
