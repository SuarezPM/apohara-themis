# THEMIS orchestrator — multi-stage Dockerfile for Fly.io
#
# Stage 1: build a static release binary
# Stage 2: minimal runtime (Debian slim, no Cargo, no source)

# ---- Build stage ----
# Rust 1.88+ is required because transitive deps (e.g. time 0.3.47,
# time-core 0.1.8) need rustc 1.88.0+.
FROM rust:1.88-slim-bookworm AS builder

# System deps for `ring` / `ed25519-dalek` (cc + linker)
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the full source. The dep cache trick (placeholder lib.rs +
# cargo build, then real source) was failing on Fly's remote builder
# with stale cache invalidation. Simpler: just copy everything and
# let Cargo's incremental compilation handle caching naturally.
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build the production binary
RUN cargo build --release --bin themis-orchestrator

# ---- Runtime stage ----
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Don't run as root
RUN useradd --create-home --shell /bin/bash themis
WORKDIR /home/themis

# Copy only the binary
COPY --from=builder /app/target/release/themis-orchestrator /usr/local/bin/themis-orchestrator

USER themis

# Fly.io sets $PORT; default 8080.
ENV PORT=8080
EXPOSE 8080

# Health check: hit / (200 expected, fast even on cold start)
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD wget -qO- http://localhost:8080/ > /dev/null || exit 1

ENTRYPOINT ["/usr/local/bin/themis-orchestrator"]
