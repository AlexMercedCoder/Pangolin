# syntax=docker/dockerfile:1.7

# ---------------------------------------------------------------------------
# Build stage
# ---------------------------------------------------------------------------
# The base image is pinned to the Rust version the README requires. It used to
# be 1.88 while the README asked for 1.92+ (A-36).
FROM rust:1.92-slim-bookworm AS builder

WORKDIR /usr/src/pangolin

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the manifests first and build a dependency-only shell, so that editing
# application source does not recompile the entire AWS/Azure/GCP/sqlx/mongodb
# dependency graph. Every build used to start from scratch (A-36).
COPY Cargo.toml Cargo.lock rustfmt.toml ./
COPY pangolin_core/Cargo.toml        pangolin_core/
COPY pangolin_store/Cargo.toml       pangolin_store/
COPY pangolin_api/Cargo.toml         pangolin_api/
COPY pangolin_cli_common/Cargo.toml  pangolin_cli_common/
COPY pangolin_cli_admin/Cargo.toml   pangolin_cli_admin/
COPY pangolin_cli_user/Cargo.toml    pangolin_cli_user/

RUN mkdir -p pangolin_core/src pangolin_store/src pangolin_api/src \
             pangolin_cli_common/src pangolin_cli_admin/src pangolin_cli_user/src \
    && echo "" > pangolin_core/src/lib.rs \
    && echo "" > pangolin_store/src/lib.rs \
    && echo "" > pangolin_api/src/lib.rs \
    && echo "" > pangolin_cli_common/src/lib.rs \
    && echo "fn main() {}" > pangolin_api/src/main.rs \
    && echo "fn main() {}" > pangolin_cli_admin/src/main.rs \
    && echo "fn main() {}" > pangolin_cli_user/src/main.rs \
    && cargo build --release --locked --bin pangolin_api \
    && rm -rf pangolin_core/src pangolin_store/src pangolin_api/src \
              pangolin_cli_common/src pangolin_cli_admin/src pangolin_cli_user/src

# Now the real sources. `--locked` refuses to silently update Cargo.lock, so
# the image matches the lockfile that was reviewed.
COPY pangolin_core       pangolin_core
COPY pangolin_store      pangolin_store
COPY pangolin_api        pangolin_api
COPY pangolin_cli_common pangolin_cli_common
COPY pangolin_cli_admin  pangolin_cli_admin
COPY pangolin_cli_user   pangolin_cli_user

# Touch the entry points so cargo does not reuse the stub build's fingerprints.
RUN touch pangolin_core/src/lib.rs pangolin_store/src/lib.rs pangolin_api/src/lib.rs \
    && cargo build --release --locked --bin pangolin_api

# ---------------------------------------------------------------------------
# Runtime stage
# ---------------------------------------------------------------------------
FROM debian:bookworm-slim

LABEL org.opencontainers.image.title="Pangolin Catalog" \
      org.opencontainers.image.description="Iceberg REST catalog with Git-style branching" \
      org.opencontainers.image.source="https://github.com/AlexMercedCoder/pangolin" \
      org.opencontainers.image.licenses="MIT"

# libssl3, not libssl-dev: the runtime needs the shared library, not headers
# and static archives (A-36). tini gives the process a real init so SIGTERM is
# delivered and reaped properly during a rolling update (A-19).
RUN apt-get update \
    && apt-get install -y --no-install-recommends libssl3 ca-certificates tini \
    && rm -rf /var/lib/apt/lists/*

# Run as an unprivileged user. The image had no USER directive at all (A-35).
RUN groupadd --system --gid 10001 pangolin \
    && useradd --system --uid 10001 --gid pangolin --home-dir /app --shell /usr/sbin/nologin pangolin

WORKDIR /app
COPY --from=builder --chown=pangolin:pangolin \
     /usr/src/pangolin/target/release/pangolin_api ./pangolin_api

USER 10001:10001

EXPOSE 8080

ENV RUST_LOG=info

# Readiness reflects a real store round-trip, so this is a meaningful check.
HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD ["/app/pangolin_api", "--healthcheck"]

ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/app/pangolin_api"]
