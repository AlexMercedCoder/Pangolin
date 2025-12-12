# Pangolin

**A Rust-Based, Multi-Tenant, Iceberg-Compatible Lakehouse Catalog**

Pangolin is a high-performance catalog designed for modern lakehouse architectures. It supports Git-style branching, multi-tenancy, and tracks any lakehouse asset type.

## Documentation Index
- [Getting Started](docs/getting_started.md): Setup and basic usage guide.
- [Deployment](docs/deployment.md): How to build and deploy Pangolin.
- [Configuration](docs/configuration.md): Runtime configuration options.
- [Environment Variables](docs/env_vars.md): List of supported env vars.
- [Dependencies](docs/dependencies.md): Overview of Rust crates used.
- [Entities & Models](docs/entities.md): Explanation of core data models (Tenant, Branch, Asset).
- [S3 Storage Configuration](docs/storage_s3.md)
- [API Overview](docs/api_overview.md): Details on Iceberg and Extended APIs.
- [Architecture](architecture.md): System design and component overview.

## Quick Start

### Prerequisites
- Rust 1.92+
- Docker (optional, for MinIO)

### Running Locally
```bash
cd pangolin
cargo run --bin pangolin_api
```

### API Usage
See [Walkthrough](walkthrough.md) for example `curl` commands.

## Features
- **Iceberg REST API**: Compatible with standard Iceberg clients.
- **Branching**: Create and manage branches for your data.
- **Multi-tenancy**: Isolated environments for different teams/customers.
- **Universal Asset Tracking**: Track Tables, Views, ML Models, and more.

## License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
