FROM debian:trixie-20260223-slim AS runtime-base
LABEL org.opencontainers.image.source=https://github.com/wtaylor/rustic-prometheus-exporter
LABEL org.opencontainers.image.description="A Prometheus exporter built on Rustic"
LABEL org.opencontainers.image.licenses=MIT

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
RUN cargo install cargo-chef --version ^0.1

FROM base AS planner
WORKDIR /usr/src
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  cargo chef prepare --recipe-path recipe.json

FROM base AS builder
WORKDIR /usr/src
COPY --from=planner /usr/src/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  cargo build --release

FROM runtime-base AS runner
WORKDIR /usr/app
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
RUN mkdir -p /etc/rustic-prometheus-operator && chmod 750 /etc/rustic-prometheus-operator
COPY --from=builder /usr/src/target/release/rustic-prometheus-exporter rustic-prometheus-exporter
ENTRYPOINT ["/usr/app/rustic-prometheus-exporter"]
CMD [ "-c", "/etc/rustic-prometheus-exporter/config.yaml", "run" ]
