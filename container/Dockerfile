# syntax=docker/dockerfile:1.7
#
# Stage 1: static musl binary using clux/muslrust (binds to ring's
# bundled BoringSSL, so no system openssl needed at runtime).
#
# Pinned by digest. Replace after running:
#   crane digest docker.io/clux/muslrust:stable
FROM docker.io/clux/muslrust:stable@sha256:7bfc0bab393986871e78e50c90e19e2a85df4fd8764a0cb881b6028abbcee4f6 AS chef
RUN cargo install cargo-chef --locked --version 0.1.78
WORKDIR /usr/src/necrometer

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ARG SOURCE_DATE_EPOCH=0
ENV SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH}
COPY --from=planner /usr/src/necrometer/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl --features serve
COPY . .
# Reproducible: SOURCE_DATE_EPOCH is honored for filesystem timestamps.
# `--features serve` is load-bearing: without it `seance` (no args) and
# `/healthz` fail because the daemon is feature-gated.
RUN cargo build --release --target x86_64-unknown-linux-musl --bin seance --features serve \
 && cp target/x86_64-unknown-linux-musl/release/seance /tmp/seance \
 && strip /tmp/seance

# Stage 2: Alpine Linux runtime.
# Pinned by digest. Replace after running:
#   crane digest alpine:3.20
FROM alpine:3.20@sha256:d9e853e87e55526f6b2917df91a2115c36dd7c696a35be12163d44e6e2a4b6bc

RUN apk add --no-cache \
        ca-certificates \
        su-exec \
        tzdata \
        wget \
    && mkdir -p /app

WORKDIR /app
COPY --from=builder /tmp/seance /usr/local/bin/seance
COPY container/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod 0755 /usr/local/bin/entrypoint.sh /usr/local/bin/seance

ENV RUST_LOG=info \
    TZ=UTC \
    PUID=99 \
    PGID=100 \
    UMASK=022 \
    NECROMETER_BIND=0.0.0.0:8080

EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
  CMD wget -qO- http://127.0.0.1:8080/healthz || exit 1
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
