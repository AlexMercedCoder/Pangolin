![Pangolin Logo](pangolin_logo.png)

# Pangolin (Status: Alpha)

**A Rust-Based, Multi-Tenant, Iceberg-Compatible Lakehouse Catalog**

Pangolin is a high-performance catalog designed for modern lakehouse architectures. It supports Git-style branching, multi-tenancy, federated catalogs, and tracks any lakehouse asset type.

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
See [Getting Started Guide](docs/getting-started/getting_started.md) for detailed setup and example `curl` commands.

---

## ✨ Key Features

- **Multi-Tenancy**: Full tenant isolation with dedicated namespaces
- **Iceberg REST Catalog**: 100% compliant with Apache Iceberg REST spec
- **Git-like Branching**: Branch and merge catalogs for safe experimentation
- **Enhanced Merge Conflict Resolution**: Intelligent conflict detection with manual and automatic resolution
- **Federated Catalogs**: Connect to external Iceberg catalogs as a transparent proxy
- **Service Users**: API key authentication for CI/CD, ETL, and automation
- **Security**: JWT authentication, OAuth 2.0, RBAC, and credential vending
- **Multi-Cloud Storage**: S3, Azure Blob, Google Cloud Storage support
- **Multiple Backend Options**: PostgreSQL, MongoDB, SQLite for metadata storage
- **Management UI**: SvelteKit-based web interface (Explorer, Auth, Admin)
- **Cloud Credential Vending**: AWS STS, Azure OAuth2, GCP service account tokens
- **Complete CRUD Operations**: Full create, read, update, delete for all entities

---

## 📚 Documentation Index

### 🎯 Getting Started
- [Quick Start Guide](docs/getting-started/getting_started.md) - Get up and running in 5 minutes
- [Configuration](docs/getting-started/configuration.md) - Server configuration options
- [Environment Variables](docs/getting-started/env_vars.md) - Complete env var reference
- [Dependencies](docs/getting-started/dependencies.md) - Rust crates and system dependencies
- [Deployment](docs/getting-started/deployment.md) - Production deployment guide
- [Docker Deployment](docs/getting-started/docker_deployment.md) - Containerized deployment
- [Client Configuration](docs/getting-started/client_configuration.md) - PyIceberg, Spark, Trino, Dremio setup

### 🔐 Security & Authentication
- [Authentication Setup](docs/authentication.md) - JWT and OAuth configuration
- [Service Users](docs/service_users.md) - API key authentication for programmatic access
- [Security & Credential Vending](docs/features/security_vending.md) - S3 credential vending
- [Permissions](docs/permissions.md) - Role-based access control

### 💻 CLI Reference
- [Overview](docs/cli/overview.md) - Introduction to Pangolin CLI
- [Configuration](docs/cli/configuration.md) - Configuring the CLI
- **Admin Commands**:
    - [Tenants](docs/cli/admin-tenants.md)
    - [Users](docs/cli/admin-users.md)
    - [Warehouses](docs/cli/admin-warehouses.md)
    - [Catalogs](docs/cli/admin-catalogs.md)
    - [Permissions](docs/cli/admin-permissions.md)
- **User Commands**:
    - [Discovery](docs/cli/user-discovery.md)
    - [Branches](docs/cli/user-branches.md)
    - [Tags](docs/cli/user-tags.md)
    - [Access Requests](docs/cli/user-access.md)

### ⚡ Core Features
- [Branch Management](docs/features/branch_management.md) - Git-like branching for catalogs
- [Merge Conflict Resolution](docs/merge_conflicts.md) - Intelligent conflict detection and resolution
- [Federated Catalogs](docs/federated_catalogs.md) - Connect to external Iceberg catalogs
- [Time Travel](docs/features/time_travel.md) - Query historical data states
- [Warehouse Management](docs/features/warehouse_management.md) - Multi-cloud storage configuration
- [Audit Logs](docs/features/audit_logs.md) - Track all catalog operations
- [Audit Logs](docs/features/audit_logs.md) - Track all catalog operations
- [Entities & Models](docs/features/entities.md) - Core data models (Tenant, Branch, Asset)
- [Data Explorer](docs/features/data_explorer.md) - **New** Browsing and Creation interface

### 💾 Backend Storage (Metadata Persistence)
- [Overview & Comparison](docs/backend_storage/README.md) - Choose the right backend
- [In-Memory](docs/backend_storage/memory.md) - ✅ Development & Testing (Ephemeral)
- [SQLite](docs/backend_storage/sqlite.md) - ✅ Production Ready (Development/Embedded)
- [PostgreSQL](docs/backend_storage/postgresql.md) - ✅ Production Ready
- [MongoDB](docs/backend_storage/mongodb.md) - ✅ Production Ready
- [Detailed Comparison](docs/backend_storage/comparison.md) - Feature comparison matrix

### 🗄️ Warehouse Storage (Object Storage for Data)
- [Warehouse Concept](docs/warehouse/README.md) - Understanding warehouses
- [AWS S3](docs/warehouse/s3.md) - ✅ Production Ready
- [Azure Blob Storage](docs/warehouse/azure.md) - ✅ Production Ready
- [Google Cloud Storage](docs/warehouse/gcs.md) - ✅ Production Ready

### 🔌 API Reference
- [API Overview](docs/api/api_overview.md) - Complete API documentation
- [Authentication](docs/api/authentication.md) - API authentication methods

### 🧪 Testing & Integration
- [PyIceberg Integration](docs/features/pyiceberg_testing.md) - Full compatibility testing
- [Test Results](tests/pyiceberg/TEST_RESULTS.md) - Latest test results

### 🏗️ Architecture
- [Architecture Overview](architecture.md) - System design and components
- [Repository Organization](ORGANIZATION.md) - Project structure

---

## 🎯 Use Cases

### Data Engineering
- **Branch-based Development**: Test schema changes on branches before merging to production
- **Multi-Environment Management**: Separate dev, staging, and production catalogs
- **Cross-Team Collaboration**: Share catalogs across teams with federated catalogs

### Data Science
- **Experiment Isolation**: Create branches for ML experiments without affecting production
- **Time Travel**: Compare model results across different data snapshots
- **Federated Access**: Access partner datasets through unified interface

### DevOps & Automation
- **CI/CD Integration**: Use service users for automated data pipelines
- **Infrastructure as Code**: Manage catalogs programmatically via REST API
- **Audit & Compliance**: Track all catalog changes with audit logs

---

## 🔧 Architecture Highlights

### Multi-Tenancy
- Complete tenant isolation
- Per-tenant warehouses and catalogs
- Role-based access control (RBAC)

### Git-like Branching
- Create branches from any commit
- Merge branches with conflict detection
- Tag important states

### Federated Catalogs
- Connect to external Iceberg REST catalogs
- Unified authentication and access control
- Cross-tenant federation support

### Security
- JWT-based authentication
- OAuth 2.0 integration (Google, Microsoft, GitHub, Okta)
- Service users with API keys
- S3 credential vending with STS (AWS, Azure, GCP)
- Cloud provider OAuth2 token generation

---

## 🚦 Project Status

**Current Version**: Alpha

**Production-Ready Features**:
- ✅ Iceberg REST Catalog API
- ✅ Multi-tenancy
- ✅ Branch management
- ✅ Merge conflict resolution
- ✅ Federated catalogs
- ✅ JWT authentication
- ✅ Service users
- ✅ PostgreSQL backend
- ✅ MongoDB backend
- ✅ SQLite backend
- ✅ S3, Azure, GCS warehouse support
- ✅ AWS STS, Azure OAuth2, GCP credential vending
- ✅ Complete CRUD operations (all entities)

**In Development**:
- 🚧 Management UI
- 🚧 Advanced RBAC features

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

### Create a Branch
```bash
POST /api/v1/branches
{
  "name": "experiment",
  "catalog": "production"
}
```

### Create a Federated Catalog
```bash
POST /api/v1/federated-catalogs
{
  "name": "partner_catalog",
  "config": {
    "base_url": "https://partner.example.com",
    "auth_type": "ApiKey",
    "credentials": {
      "api_key": "pgl_key_xyz..."
    }
  }
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

table = catalog.load_table("namespace.table")
df = table.scan().to_pandas()
```

---

## 🤝 Contributing

Contributions are welcome! Please see our contributing guidelines (coming soon).

---

## 📄 License

MIT License - see LICENSE file for details.

---

## 🔗 Related Projects

- [Apache Iceberg](https://iceberg.apache.org/) - Table format specification
- [PyIceberg](https://py.iceberg.apache.org/) - Python library for Iceberg
- [Apache Spark](https://spark.apache.org/) - Distributed processing engine

---

## 📞 Support

- Documentation: See docs/ directory
- Issues: GitHub Issues
- Discussions: GitHub Discussions
