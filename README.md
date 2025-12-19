![Pangolin Logo](pangolin_logo.png)

# Pangolin (Status: Alpha)

**A Rust-Based, Multi-Tenant, Iceberg-Compatible Lakehouse Catalog**

Pangolin is a high-performance catalog designed for modern lakehouse architectures. It supports Git-style branching, multi-tenancy, federated catalogs, and tracks any lakehouse asset type.

## Why Pangolin?

A pangolin is a strong metaphor for a data lakehouse catalog because its defining traits align closely with the core responsibilities of a catalog.

First, a pangolin is covered in layered scales. Each scale is distinct but part of a coherent whole. A lakehouse catalog works the same way. It organizes many independent assets—tables, views, files, models, and metadata—into a single, structured system. Each asset has its own schema, properties, and lineage, yet all are discoverable through one catalog.

Second, pangolins are defensive by design. They protect what matters by curling into a secure form. A catalog plays a similar role in governance. It enforces access controls, tracks ownership, and provides guardrails around sensitive data. Rather than blocking access outright, it enables safe and intentional use.

Third, pangolins are precise and deliberate. They move carefully and use strong claws to uncover food hidden beneath the surface. A lakehouse catalog does the same for data. It helps users uncover datasets buried across object storage, warehouses, and streams, exposing meaning through metadata, classification, and search.

Finally, pangolins are rare and specialized. They exist for a specific purpose and excel at it. A data lakehouse catalog is not a generic system. It is a purpose-built layer focused on clarity, trust, and navigation across complex data environments.

---

## 🚀 Quick Start

### Prerequisites
- Rust 1.92+
- Docker (optional, for MinIO)

### Running Locally
```bash
cd pangolin
cargo run --bin pangolin_api
```

### API Usage
See [Quick Start Guide](docs/getting-started/getting_started.md) for detailed setup and example `curl` commands.

---

## ✨ Key Features

- **Multi-Tenancy**: Full tenant isolation with dedicated namespaces and warehouses.
- **Iceberg REST Catalog**: 100% compliant with Apache Iceberg REST spec.
- **Git-like Branching**: Branch, tag, and merge catalogs for safe experimentation.
- **3-Way Merging**: Intelligent conflict detection with manual and automatic resolution strategies.
- **Federated Catalogs**: Connect to external Iceberg catalogs as a transparent proxy.
- **Service Users**: API key authentication for CI/CD, ETL, and automated pipelines.
- **Advanced Audit Logging**: Comprehensive tracking of 40+ actions across 19 resource types.
- **Multi-Cloud Storage**: Native support for AWS S3, Azure Blob, and Google Cloud Storage.
- **Credential Vending**: Securely vends AWS STS, Azure SAS, and GCP downscoped credentials.
- **Multiple Backends**: Metadata persistence via PostgreSQL, MongoDB, SQLite, or In-Memory.
- **Management UI**: Modern SvelteKit-based interface for Admins and Data Explorers.

---

## 📚 Documentation Index

### 🎯 1. Getting Started
- [Quick Start Guide](docs/getting-started/getting_started.md) - Get running in 5 minutes.
- [Configuration](docs/getting-started/configuration.md) - Server configuration options.
- [Environment Variables](docs/getting-started/env_vars.md) - Complete env var reference.
- [Client Configuration](docs/getting-started/client_configuration.md) - PyIceberg, Spark, and Trino setup.

### 🔐 2. Security & Authentication
- [Authentication Setup](docs/authentication.md) - JWT, OAuth, and No-Auth modes.
- [Service Users](docs/service_users.md) - Programmatic API key access.
- [Permissions & RBAC](docs/permissions.md) - Granular asset-level access control.
- [Credential Vending](docs/features/security_vending.md) - Secure data access management.

### ⚡ 3. Core Features
- [Git-like Branching](docs/features/branch_management.md) - Workflows for data versioning.
- [Merging & Conflicts](docs/features/merge_operations.md) - Understanding data reconciliation.
- [Audit Logging](docs/features/audit_logs.md) - Comprehensive security and compliance tracking.
- [Federated Catalogs](docs/federated_catalogs.md) - Unified access to remote catalogs.

### 💻 4. CLI Reference
- [CLI Overview](docs/cli/overview.md) - Introduction to `pangolin-admin` and `pangolin-user`.
- **Admin**: [Tenants](docs/cli/admin-tenants.md), [Users](docs/cli/admin-users.md), [Warehouses](docs/cli/admin-warehouses.md), [Catalogs](docs/cli/admin-catalogs.md).
- **User**: [Discovery](docs/cli/user-discovery.md), [Branches](docs/cli/user-branches.md), [Tags](docs/cli/user-tags.md).

### 🖥️ 5. Management UI
- [UI Overview](docs/ui/overview.md) - Layout, navigation, and authentication.
- [Administration](docs/ui/administration.md) - Managing Tenants and Infrastructure.
- [Data Explorer](docs/ui/explorer.md) - Browsing and managing data assets.

---

## 🚦 Project Status

**Current Version**: Alpha

**Production-Ready Features**:
- ✅ Iceberg REST Catalog API
- ✅ Multi-Tenancy & Tenant Isolation
- ✅ Branch & Tag Management
- ✅ Advanced Audit Logging
- ✅ Service Users & API Keys
- ✅ PostgreSQL, MongoDB, and SQLite Backends
- ✅ AWS S3 Warehouse Support & STS Vending
- ✅ Management UI (Core Administrative & Explorer Features)

---

## 📖 Quick Examples

### Create a Catalog
```bash
POST /api/v1/catalogs
{
  "name": "production",
  "warehouse_name": "s3_warehouse",
  "storage_location": "s3://my-bucket/warehouse"
}
```

### Use with PyIceberg
```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080",
        "warehouse": "production",
        "token": "your-jwt-token",
    }
)

table = catalog.load_table("analytics.sales")
df = table.scan().to_pandas()
```

---

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

---

## 📞 Support

- **Documentation**: See [docs/](docs/) directory.
- **Issues**: [GitHub Issues](https://github.com/AlexMercedCoder/Pangolin/issues).
- **Discussions**: [GitHub Discussions](https://github.com/AlexMercedCoder/Pangolin/discussions).
