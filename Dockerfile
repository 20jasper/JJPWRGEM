ARG RUST_VERSION=1.91.1
ARG APP_NAME=jjp

FROM rust:${RUST_VERSION}-slim-bullseye AS build
ARG APP_NAME
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    clang \
    lld \
    musl-dev \
    git \
    curl \
    ca-certificates

# cargo formatters
RUN set -eux; \
    curl -fsSL "https://github.com/cargo-bins/cargo-binstall/releases/download/v1.16.3/cargo-binstall-x86_64-unknown-linux-gnu.tgz" -o /tmp/cargo-binstall.tgz; \
    tar -xzf /tmp/cargo-binstall.tgz -C /tmp; \
    install -m755 /tmp/cargo-binstall /usr/local/cargo/bin/cargo-binstall; \
    rm -f /tmp/cargo-binstall /tmp/cargo-binstall.tgz
RUN cargo binstall sjq -y \
    && cargo binstall jsonxf -y \
    && cargo binstall jsonformat-cli -y \
    && cargo binstall hyperfine -y

RUN cargo install dprint --locked

RUN --mount=type=bind,source=crates,target=crates \
    --mount=type=bind,source=xtask,target=xtask \
    --mount=type=bind,source=tests,target=tests \
    --mount=type=bind,source=axolotl.txt,target=axolotl.txt \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release && \
    install -Dm755 ./target/release/$APP_NAME /usr/local/cargo/bin/$APP_NAME

FROM node:24-bullseye AS mise
ARG BENCHMARK_PATH="./xtask/bench"

COPY ${BENCHMARK_PATH}/mise.toml /mise/mise.toml
SHELL ["/bin/bash", "-o", "pipefail", "-c"]
ENV MISE_DATA_DIR="/mise"
ENV MISE_CONFIG_DIR="/mise"
ENV MISE_CACHE_DIR="/mise/cache"
ENV MISE_INSTALL_PATH="/usr/local/bin/mise"
ENV PATH="/mise/shims:$PATH"

RUN curl https://mise.run | sh

RUN mise trust -y
RUN MISE_JOBS=1 mise install

FROM node:24-bullseye AS final

RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential

ARG BENCHMARK_PATH="./xtask/bench"

# reuse mise installation layers unless mise.toml changes
COPY --from=mise /usr/local/bin/mise /usr/local/bin/mise
COPY --from=mise /mise /mise
SHELL ["/bin/bash", "-o", "pipefail", "-c"]
ENV MISE_DATA_DIR="/mise"
ENV MISE_CONFIG_DIR="/mise"
ENV MISE_CACHE_DIR="/mise/cache"
ENV MISE_INSTALL_PATH="/usr/local/bin/mise"
ENV PATH="/mise/shims:$PATH"

RUN apt-get update && apt-get install -y jshon

# Working directory
WORKDIR /benchmark

# Copy benchmark scripts, config, and JSON data
COPY --chmod=0755 ${BENCHMARK_PATH}/benchmark.sh .
COPY ${BENCHMARK_PATH}/dprint.json .
COPY ${BENCHMARK_PATH}/data/json-benchmark/data/ ./data

RUN chmod +x benchmark.sh

# Copy all cargo-binstalled binaries from the build stage.
COPY --from=build /usr/local/cargo/bin/ /usr/local/bin/

ENV OUTPUT_DIR=/benchmark/output

# Default command runs both benchmarks
CMD ["bash", "-c", "./benchmark.sh"]

