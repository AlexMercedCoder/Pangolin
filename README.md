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

### 🏁 1. Getting Started
- **[Installation & Setup](docs/getting-started/getting_started.md)** - Get running in 5 minutes.
- **[Auth Modes](docs/authentication.md)** - Understanding Auth vs No-Auth and OAuth.
- **[Service Users](docs/features/service_users.md)** - API keys for programmatic access.
- **[Multi-Tenancy](docs/features/multi_tenancy.md)** - Understanding isolation.
- **[User Scopes](docs/getting-started/getting_started.md#user-scopes)** - Roles: Root, Tenant Admin, and Tenant User.
- **[Configuration](docs/getting-started/configuration.md)** - Server configuration options.
- **[Environment Variables](docs/getting-started/env_vars.md)** - Complete metadata and storage reference.

### 🏗️ 2. Core Infrastructure
- **[Warehouses](docs/warehouse/README.md)** - Managing S3, Azure, and GCS storage.
- **[Credential Vending](docs/features/security_vending.md)** - Secure direct-to-storage access.
- **[Catalogs](docs/features/asset_management.md)** - Creating Local and Federated catalogs.
- **[Backend Storage](docs/backend_storage/README.md)** - Metadata persistence with Postgres, Mongo, or SQLite.

### 🧪 3. Data Management (API, CLI, UI)
- **[Branching & Versioning](docs/features/branch_management.md)** - Git-style workflows and auto-add nuances.
- **[Permissions & RBAC](docs/permissions.md)** - Asset-level access and cascading grants.
- **[IAM Roles](docs/features/iam_roles.md)** - Cloud provider integration.
- **[Business Metadata](docs/features/business_catalog.md)** - Tags, search, and data discovery.
- **[Audit Logging](docs/features/audit_logs.md)** - Security tracking across all tools.
- **[Maintenance](docs/features/maintenance.md)** - Snapshots, orphan files, and storage optimization.

### 🛠️ 4. Tooling & APIs
- **[CLI Reference](docs/cli/overview.md)** - Full guide for `pangolin-admin` and `pangolin-user`.
- **[API Reference](docs/api/api_overview.md)** - Iceberg REST and Pangolin Management APIs.
- **[Management UI](docs/ui/overview.md)** - Visual administration and data discovery.
- **[Python Client](pypangolin/README.md)** - Official Python library (`pypangolin`).
- **[Client Setup](docs/getting-started/client_configuration.md)** - Connecting PyIceberg, Spark, and Trino.

### 📖 5. Best Practices
- **[Deployment](docs/best-practices/deployment.md)** - Production deployment, Docker, Kubernetes, HA setup.
- **[Scalability](docs/best-practices/scalability.md)** - Horizontal scaling, database optimization, caching.
- **[Security](docs/best-practices/security.md)** - Authentication, encryption, audit logging, compliance.
- **[Permissions Management](docs/best-practices/permissions.md)** - RBAC patterns, least privilege, access control.
- **[Branch Management](docs/best-practices/branching.md)** - Git-like workflows, merge strategies, conflict resolution.
- **[Business Metadata](docs/best-practices/metadata.md)** - Metadata strategy, governance, data classification.
- **[Apache Iceberg](docs/best-practices/iceberg.md)** - Table design, partitioning, schema evolution, performance.
- **[Generic Assets](docs/best-practices/generic-assets.md)** - Managing ML models, files, media, and artifacts.

---

## 🚦 Project Status

**Current Version**: Alpha

**Production-Ready Features**:
- ✅ Iceberg REST Catalog API (100% Compliant)
- ✅ Multi-Tenancy & Tenant Isolation
- ✅ Git-like Branching & Tagging
- ✅ Advanced Audit Logging (UI/CLI/API)
- ✅ Service Users & API Keys
- ✅ PostgreSQL, MongoDB, and SQLite Backends
- ✅ Multi-Cloud Storage (S3, Azure, GCS)
- ✅ Management UI for Admins & Explorers

---

## 📖 Quick Examples

### Create a Catalog (API)
```bash
curl -X POST http://localhost:8080/api/v1/catalogs \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
  "name": "production",
  "warehouse_name": "main_s3",
  "storage_location": "s3://my-bucket/warehouse"
}'
```

### Create a Branch (CLI)
```bash
pangolin-user create-branch dev --from main --catalog production
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
        "header.X-Iceberg-Access-Delegation": "vended-credentials",
    }
)

# Load a table on the 'dev' branch
table = catalog.load_table("analytics.sales@dev")
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
