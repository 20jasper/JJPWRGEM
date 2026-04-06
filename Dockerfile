ARG RUST_VERSION=1.94.1
ARG APP_NAME=jjp

FROM rust:${RUST_VERSION}-slim-bullseye AS cargo-formatters
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

# layer 1: streaming formatters
RUN cargo binstall sjq -y \
    && cargo binstall jsonxf -y \
    && cargo binstall jsonformat-cli -y

# layer 2: parsing formatters + benchmark runner
RUN cargo binstall json-pp-rust -y \
    && cargo binstall jsonice -y \
    && cargo binstall hyperfine -y

RUN cargo install dprint --locked
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


RUN --mount=type=bind,source=crates,target=crates \
    --mount=type=bind,source=xtask,target=xtask \
    --mount=type=bind,source=tests,target=tests \
    --mount=type=bind,source=benches,target=benches \
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

# layer 1: heavy runtimes + npm/pipx tools (slow to install, rarely change)
RUN MISE_JOBS=1 mise install bun python uv 'npm:json-minify' 'npm:prettier' 'npm:oxfmt' 'pipx:jello'

# layer 2: lightweight binary tools + github releases (use token to avoid rate limits)
RUN --mount=type=secret,id=gh_token \
    GITHUB_TOKEN="$(cat /run/secrets/gh_token 2>/dev/null || true)" \
    MISE_JOBS=1 mise install jaq gojq jq 'github:caarlos0/jsonfmt' 'github:swaggest/json-cli' 'github:tdewolff/minify' 'github:tidwall/jj'

# fallback: install anything in mise.toml not already covered above (mise skips installed tools)
RUN --mount=type=secret,id=gh_token \
    GITHUB_TOKEN="$(cat /run/secrets/gh_token 2>/dev/null || true)" \
    MISE_JOBS=1 mise install

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

# Copy cargo-binstalled formatters and the jjp binary.
COPY --from=cargo-formatters /usr/local/cargo/bin/ /usr/local/bin/
COPY --from=build /usr/local/cargo/bin/jjp /usr/local/bin/jjp

ENV OUTPUT_DIR=/benchmark/output

# Default command runs both benchmarks
CMD ["bash", "-c", "./benchmark.sh"]
