FROM debian:trixie-20260223-slim AS runtime-base
FROM runtime-base AS base
RUN apt-get update && apt-get install -y --no-install-recommends \
  build-essential \
  rustup \
  pkg-config \
  openssl \
  libssl-dev \
  ca-certificates
WORKDIR /usr/src
COPY ./rust-toolchain.toml .
RUN cargo install sccache --version ^0.7
RUN cargo install cargo-chef --version ^0.1
ENV RUSTC_WRAPPER=sccache SCCACHE_DIR=/sccache

FROM base AS planner
WORKDIR /usr/src
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
  cargo chef prepare --recipe-path recipe.json

FROM base AS builder
WORKDIR /usr/src
COPY --from=planner /usr/src/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
  cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
  cargo build --release

FROM runtime-base AS runner
WORKDIR /usr/app

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/target/release/rustic-prometheus-exporter rustic-prometheus-exporter

ENTRYPOINT ["/usr/app/rustic_prometheus_exporter"]

