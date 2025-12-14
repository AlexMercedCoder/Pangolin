# Dependencies Overview

Pangolin is built with the following key Rust crates:

## Core Frameworks
- **[Axum](https://github.com/tokio-rs/axum)**: Ergonomic and modular web application framework.
- **[Tokio](https://tokio.rs/)**: Asynchronous runtime for Rust.
- **[Tower](https://github.com/tower-rs/tower)**: Modular and reusable components for building robust networking clients and servers.

## Data & Serialization
- **[Serde](https://serde.rs/)**: Framework for serializing and deserializing Rust data structures.
- **[Serde JSON](https://github.com/serde-rs/json)**: JSON support for Serde.
- **[Bytes](https://github.com/tokio-rs/bytes)**: Utilities for working with bytes (used in federated proxy).

## Storage
- **[Object Store](https://github.com/apache/arrow-rs/tree/master/object_store)**: Unified interface for object storage (S3, GCS, Azure, Local).
- **[DashMap](https://github.com/xacrimon/dashmap)**: Concurrent associative array for high-performance in-memory storage.
- **[SQLx](https://github.com/launchbadge/sqlx)**: Async SQL toolkit for PostgreSQL, MongoDB, and SQLite.
  - Features: `postgres`, `mongodb`, `sqlite`, `uuid`, `chrono`, `json`
- **[MongoDB](https://github.com/mongodb/mongo-rust-driver)**: Official MongoDB driver for Rust.

## Security & Authentication
- **[JSON Web Token](https://github.com/Keats/jsonwebtoken)**: JWT creation and validation.
- **[bcrypt](https://github.com/Keats/rust-bcrypt)**: Password hashing for users and API key hashing for service users.
- **[AWS SDK for S3](https://github.com/awslabs/aws-sdk-rust)**: Official AWS SDK for S3 operations and STS credential vending.
- **[AWS Config](https://github.com/awslabs/aws-sdk-rust)**: AWS configuration loading.

## HTTP Client (Federated Catalogs)
- **[reqwest](https://github.com/seanmonstar/reqwest)**: HTTP client for forwarding requests to external catalogs.

## Utilities
- **[Uuid](https://github.com/uuid-rs/uuid)**: UUID generation for entities.
- **[Chrono](https://github.com/chronotope/chrono)**: Date and time handling.
- **[Tracing](https://github.com/tokio-rs/tracing)**: Application-level tracing for async Rust.
- **[Anyhow](https://github.com/dtolnay/anyhow)**: Flexible error handling.

## Development Dependencies
- **[tokio-test](https://docs.rs/tokio-test)**: Testing utilities for async code.
- **[mockall](https://github.com/asomers/mockall)**: Mocking framework for tests.

## Feature-Specific Dependencies

### Service Users
- `bcrypt` - API key hashing
- `uuid` - Service user ID generation
- `chrono` - Expiration tracking

### Merge Conflict Resolution
- `uuid` - Merge operation and conflict ID generation
- `chrono` - Timestamp tracking
- `serde` - Conflict serialization

### Federated Catalogs
- `reqwest` - HTTP client for proxying requests
- `bytes` - Request/response body handling
- `axum::http` - HTTP types for forwarding

## System Requirements

### Minimum
- Rust 1.92+
- 2GB RAM (for development)
- Linux, macOS, or Windows

### Recommended (Production)
- Rust 1.92+
- 4GB+ RAM
- Linux (Ubuntu 20.04+ or similar)
- **Backend Storage** (choose one):
  - PostgreSQL 12+ (recommended for production)
  - MongoDB 5+ (recommended for cloud-native)
  - SQLite 3.35+ (recommended for development/embedded)
- **Warehouse Storage**: S3, Azure Blob, or GCS

## Building from Source

```bash
# Clone repository
git clone https://github.com/your-org/pangolin.git
cd pangolin

# Build all crates
cargo build --release

# Run tests
cargo test

# Run API server
cargo run --bin pangolin_api --release
```

## Docker Dependencies

If using Docker deployment:
- Docker 20.10+
- Docker Compose 1.29+ (optional)

## Related Documentation

- [Getting Started](./getting_started.md) - Quick start guide
- [Configuration](./configuration.md) - Configuration options
- [Deployment](./deployment.md) - Production deployment
