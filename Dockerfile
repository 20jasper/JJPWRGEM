ARG APP_NAME=jjp

FROM rustlang/rust:nightly-slim@sha256:613250fdd1a3a2e4b8077f98bb3ff63428965a6d0859fe7afe45c61ff01d1fb4 AS build
ARG APP_NAME
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    clang \
    lld \
    musl-dev \
    git \
    curl \
    ca-certificates \
  && rm -rf /var/lib/apt/lists/*

RUN --mount=type=bind,source=crates,target=crates \
    --mount=type=bind,source=xtask,target=xtask \
    --mount=type=bind,source=tests,target=tests \
    --mount=type=bind,source=benches,target=benches \
    --mount=type=bind,source=axolotl.txt,target=axolotl.txt \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=rust-toolchain.toml,target=rust-toolchain.toml \
    --mount=type=bind,source=.cargo,target=.cargo \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release && \
    install -Dm755 ./target/release/$APP_NAME /usr/local/cargo/bin/$APP_NAME

FROM mise-tools AS final

ARG BENCHMARK_PATH="./benches/docker"

COPY --from=build /usr/local/cargo/bin/jjp /usr/local/bin/jjp

WORKDIR /benchmark

COPY --chmod=0755 ${BENCHMARK_PATH}/benchmark.sh .
COPY ${BENCHMARK_PATH}/dprint.json .
COPY benches/data/ ./data

ENV OUTPUT_DIR=/benchmark/output

CMD ["bash", "-c", "./benchmark.sh"]
