# Build Stage
FROM rust:1.83-slim-bookworm as builder

WORKDIR /usr/src/pangolin

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY pangolin_core ./pangolin_core
COPY pangolin_store ./pangolin_store
COPY pangolin_api ./pangolin_api

# Build dependencies and application
RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim

# Install OpenSSL and CA certificates
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /usr/src/pangolin/target/release/pangolin_api ./pangolin

# Expose port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info

# Run the application
CMD ["./pangolin"]
