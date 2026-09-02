# syntax=docker/dockerfile:1.7

# Frontend (Vite) — independiente del toolchain ZK.
FROM node:20-bookworm-slim AS web
WORKDIR /web
COPY web/package.json web/package-lock.json ./
RUN npm ci
COPY web/ ./
RUN npm run build

# Toolchain RISC Zero 3.0.6 sobre Ubuntu 24.04 (glibc 2.39).
FROM ubuntu:24.04 AS risc0
ARG DEBIAN_FRONTEND=noninteractive
ARG RISC0_VERSION=3.0.6
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates curl build-essential pkg-config libssl-dev git \
    && rm -rf /var/lib/apt/lists/*
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:/root/.risc0/bin:${PATH}"
RUN curl -L https://risczero.com/install | bash \
    && rzup install rust \
    && rzup install cargo-risczero ${RISC0_VERSION} \
    && rzup install r0vm ${RISC0_VERSION} \
    && rzup install cpp

# Compila guest zkVM + server + verifier-cli.
FROM risc0 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY feed-core ./feed-core
COPY methods ./methods
COPY server ./server
COPY verifier-cli ./verifier-cli
# El guest se compila igual; DEV_MODE solo afecta el proving en runtime.
ENV RISC0_DEV_MODE=1 \
    CARGO_NET_RETRY=10
RUN cargo build --release --bin server --bin verifier-cli --bin zkbench \
    && mkdir -p /out \
    && cp target/release/server /out/server \
    && cp target/release/verifier-cli /out/verifier-cli \
    && cp target/release/zkbench /out/zkbench

# Imagen final: binarios + frontend. Sin Rust ni Node.
FROM ubuntu:24.04
ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates libssl3 libstdc++6 curl \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /data /app/web/dist
COPY --from=builder /out/server /usr/local/bin/server
COPY --from=builder /out/verifier-cli /usr/local/bin/verifier-cli
COPY --from=builder /out/zkbench /usr/local/bin/zkbench
COPY --from=risc0 /root/.cargo/bin/r0vm /usr/local/bin/r0vm
COPY --from=web /web/dist /app/web/dist
ENV INZK_WEB_DIST=/app/web/dist \
    INZK_DB=/data/inzktagram.sqlite \
    RISC0_DEV_MODE=0
EXPOSE 8080
VOLUME /data
HEALTHCHECK --interval=10s --timeout=3s --start-period=20s --retries=12 \
    CMD curl -fsS http://127.0.0.1:8080/api/users >/dev/null || exit 1
WORKDIR /app
CMD ["server"]
