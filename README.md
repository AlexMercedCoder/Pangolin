# Pangolin (Status:Alpha)

**A Rust-Based, Multi-Tenant, Iceberg-Compatible Lakehouse Catalog**

Pangolin is a high-performance catalog designed for modern lakehouse architectures. It supports Git-style branching, multi-tenancy, and tracks any lakehouse asset type.

## Documentation

### 📚 [Getting Started](docs/getting-started/)
- [Quick Start Guide](docs/getting-started/getting_started.md)
- [Configuration](docs/getting-started/configuration.md)
- [Deployment](docs/getting-started/deployment.md)
- [Docker Deployment](docs/getting-started/docker_deployment.md)

### 💾 [Storage](docs/storage/)
- [AWS S3](docs/storage/storage_s3.md) ✅ Production Ready
- [Azure Blob Storage](docs/storage/storage_azure.md) ✅ Implemented
- [Google Cloud Storage](docs/storage/storage_gcs.md) ✅ Implemented
- [MongoDB](docs/storage/storage_mongo.md)
- [PostgreSQL](docs/storage/storage_postgres.md)

### 🔌 [API Reference](docs/api/)
- [API Overview](docs/api/api_overview.md)
- [Authentication](docs/api/authentication.md)

### ⚡ [Features](docs/features/)
- [Branch Management](docs/features/branch_management.md) - Git-like branching
- [Time Travel](docs/features/time_travel.md) - Query historical data
- [Warehouse Management](docs/features/warehouse_management.md) - Multi-cloud storage
- [PyIceberg Integration](docs/features/pyiceberg_testing.md) - Full compatibility
- [Security & Credential Vending](docs/features/security_vending.md)
- [Audit Logs](docs/features/audit_logs.md)

### 🔬 [Research & Planning](docs/research/)
- Implementation plans and research notes

## Quick Links

- [Architecture Overview](architecture.md)
- [Repository Organization](ORGANIZATION.md)
- [Test Results](tests/pyiceberg/TEST_RESULTS.md): Runtime configuration options.
  - [Client Configuration](docs/getting-started/client_configuration.md): PyIceberg, PySpark, Trino, Dremio setup.
  - [Environment Variables](docs/getting-started/env_vars.md): List of supported env vars.
- [Dependencies](docs/getting-started/dependencies.md): Overview of Rust crates used.
- [Entities & Models](docs/features/entities.md): Explanation of core data models (Tenant, Branch, Asset).
- [API Overview](docs/api/api_overview.md): Details on Iceberg and Extended APIs.
- [Authentication Setup](docs/setup/authentication.md): JWT and OAuth configuration guide.
- [Management UI](pangolin_ui/README.md): Guide for the SvelteKit UI.

## Features
- **Multi-Tenancy**: Full isolation with Tenant IDs.
- **Iceberg REST Catalog**: Full compliance including Commit support.
- **Security**: JWT-based Authentication and RBAC.
- **Management UI**: SvelteKit-based web interface.
- **Warehouse Management**: Manage storage configurations per tenant.
- **Git-like Branching**: Branch and merge catalogs for experiment isolation.
- **S3 Integration**: Persist data to S3 or MinIO.
- **Non-Iceberg Assets**: Manage Views and other assets alongside Iceberg tables.

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
- **Multi-Tenancy**: Built-in tenant isolation.
- **Warehouse Management**: Manage storage configurations per tenant.
- **Git-like Branching**: Branch and merge catalogs for experiment isolation.
- **Iceberg Native**: Fully compatible with the Iceberg REST Catalog spec.
- **S3 Integration**: Persist data to S3 or MinIO.
- **Non-Iceberg Assets**: Manage Views and other assets alongside Iceberg tables.

## License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
