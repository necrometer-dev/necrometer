# Stage 1: static musl binary (zero glibc dependency at runtime)
FROM docker.io/clux/muslrust:stable AS builder
WORKDIR /usr/src/necrometer
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

# Stage 2: Alpine Linux runtime (NOT scratch; NOT UBI unless compliance requires it).
#
# Unraid console:  docker exec -it necrometer sh
# Shell is BusyBox ash with essential utilities (wget for healthcheck).
#
# Permission model:
#   - ENTRYPOINT runs as root, then exec's the daemon via su-exec as PUID:PGID
#     (default 99:100). No required user env vars.
#   - Daemon binds 0.0.0.0:8080 (ports >= 1024 need no root).
FROM alpine:3.20

RUN apk add --no-cache \
        ca-certificates \
        su-exec \
        tzdata \
        wget \
    && mkdir -p /app \
    && test -x /bin/sh \
    && (test -x /bin/ash || ln -sf /bin/busybox /bin/ash)

WORKDIR /app
COPY --from=builder /usr/src/necrometer/target/x86_64-unknown-linux-musl/release/necrometer /usr/local/bin/necrometer
COPY container/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh /usr/local/bin/necrometer

# GITHUB_TOKEN optional but recommended (5000/hr vs 60/hr API limit).
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
