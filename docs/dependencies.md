# Dependencies Overview

Pangolin is built with the following key Rust crates:

## Core Frameworks
- **[Axum](https://github.com/tokio-rs/axum)**: Ergonomic and modular web application framework.
- **[Tokio](https://tokio.rs/)**: Asynchronous runtime.
- **[Tower](https://github.com/tower-rs/tower)**: Modular and reusable components for building robust networking clients and servers.

## Data & Serialization
- **[Serde](https://serde.rs/)**: Framework for serializing and deserializing Rust data structures.
- **[Serde JSON](https://github.com/serde-rs/json)**: JSON support for Serde.

## Storage
- **[Object Store](https://github.com/apache/arrow-rs/tree/master/object_store)**: A unified interface for object storage (S3, GCS, Azure, Local).
- **[DashMap](https://github.com/xacrimon/dashmap)**: Concurrent associative array for high-performance in-memory storage.

## Utilities
- **[Uuid](https://github.com/uuid-rs/uuid)**: UUID generation.
- **[Chrono](https://github.com/chronotope/chrono)**: Date and time handling.
- **[Tracing](https://github.com/tokio-rs/tracing)**: Application-level tracing for async Rust.
- **[Anyhow](https://github.com/dtolnay/anyhow)**: Flexible error handling.
