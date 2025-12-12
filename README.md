# Pangolin

**A Rust-Based, Multi-Tenant, Iceberg-Compatible Lakehouse Catalog**

Pangolin is a high-performance catalog designed for modern lakehouse architectures. It supports Git-style branching, multi-tenancy, and tracks any lakehouse asset type.

## Documentation Index
- [Getting Started](docs/getting_started.md): Setup and basic usage guide.
- **Storage Backends**:
  - [S3 Storage](docs/storage_s3.md)
  - [Postgres Storage](docs/storage_postgres.md)
  - [MongoDB Storage](docs/storage_mongo.md)
- **Features**:
  - [Branch Management](docs/branch_management.md)
  - [Tag Management](docs/tag_management.md)
  - [Time Travel](docs/time_travel.md)
  - [Audit Logs](docs/audit_logs.md)
  - [Table Maintenance](docs/maintenance.md)
  - [Security & Vending](docs/security_vending.md)
  - [Authentication](docs/authentication.md)
  - [RBAC & UI](docs/rbac_ui.md)
- **Deployment**:
  - [Docker Deployment](docs/docker_deployment.md)
  - [Configuration](docs/configuration.md): Runtime configuration options.
  - [Client Configuration](docs/client_configuration.md): PyIceberg, PySpark, Trino, Dremio setup.
  - [Environment Variables](docs/env_vars.md): List of supported env vars.
- [Dependencies](docs/dependencies.md): Overview of Rust crates used.
- [Entities & Models](docs/entities.md): Explanation of core data models (Tenant, Branch, Asset).
- [API Overview](docs/api_overview.md): Details on Iceberg and Extended APIs.
- [Architecture](architecture.md): System design and component overview.
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
