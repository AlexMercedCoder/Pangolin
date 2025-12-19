# Pangolin Documentation

Welcome to the comprehensive documentation for Pangolin, a multi-tenant Apache Iceberg REST Catalog with advanced features including federated catalogs, credential vending, fine-grained permissions, and Git-like branching.

## 📚 Documentation Index

### 🚀 1. Getting Started
New to Pangolin? Start here to get up and running quickly.

- **[Quick Start Guide](./getting-started/getting_started.md)** - Get Pangolin running in 5 minutes.
- **[Evaluating Pangolin](./getting-started/evaluating-pangolin.md)** - Feature comparison and evaluation.
- **[Configuration](./getting-started/configuration.md)** - Core server configuration options.
- **[Environment Variables](./getting-started/env_vars.md)** - Complete environment variable reference.
- **[Dependencies](./getting-started/dependencies.md)** - System and Rust dependencies.
- **[Deployment](./getting-started/deployment.md)** - Production deployment considerations.
- **[Docker Deployment](./getting-started/docker_deployment.md)** - Running Pangolin in containers.
- **[Client Configuration](./getting-started/client_configuration.md)** - Setting up PyIceberg, Spark, Trino, and Dremio.

### 📖 2. Core Concepts
Fundamental principles behind Pangolin's design.

- **[Multi-Tenancy](./features/multi_tenancy.md)** - How Pangolin isolates data and users.
- **[Merging Principles](./features/merge_operations.md)** - The logic behind branch merging and conflict detection.

### 🔐 3. Security & Authentication
Secure your catalog and manage access.

- **[Authentication & Setup](./authentication.md)** - Comprehensive guide to JWT, OAuth, and No-Auth modes.
- **[Service Users](./service_users.md)** - Creating programmatic identities with API keys.
- **[Permissions & RBAC](./permissions.md)** - Granular access control for catalogs and assets.
- **[Credential Vending](./features/security_vending.md)** - Securely vending S3/provider credentials to clients.

### ⚡ 4. Features
Deep dives into Pangolin's advanced capabilities.

- **[Branch Management](./features/branch_management.md)** - Git-like workflows for your data.
- **[Merge operations](./cli/admin-merge-operations.md)** - Practical guide to merging branches.
- **[Federated Catalogs](./federated_catalogs.md)** - Proxying external Iceberg REST catalogs.
- **[Time Travel](./features/time_travel.md)** - Querying historical snapshots of your data.
- **[Warehouse Management](./features/warehouse_management.md)** - Organizing multi-cloud storage.
- **[Audit Logs](./features/audit_logs.md)** - Tracking every operation in your catalog.
- **[Asset Management](./features/asset_management.md)** - Managing tables, views, and models.
- **[Business Metadata](./features/business_catalog.md)** - Enhancing discovery with search and metadata.

### 💻 5. CLI Reference
Command-line tools for administrators and users.

- **[CLI Overview](./cli/overview.md)** - Introduction to Pangolin CLI tools.
- **[CLI Configuration](./cli/configuration.md)** - Setting up your environment.
- **Admin Commands**:
    - [Tenants](./cli/admin-tenants.md), [Users](./cli/admin-users.md), [Warehouses](./cli/admin-warehouses.md), [Catalogs](./cli/admin-catalogs.md)
    - [Federated Catalogs](./cli/admin-federated-catalogs.md)
    - [Service Users](./cli/admin-service-users.md)
    - [Audit Logging](./cli/admin-audit-logging.md)
    - [Permissions](./cli/admin-permissions.md)
    - [Business Metadata](./cli/admin-metadata.md)
    - [Merge Operations](./cli/admin-merge-operations.md)
- **User Commands**:
    - [Discovery](./cli/user-discovery.md), [Branches](./cli/user-branches.md), [Tags](./cli/user-tags.md), [Tokens](./cli/user-tokens.md), [Access Requests](./cli/user-access.md)

### 🔌 6. API Reference
Integrate directly with Pangolin's REST interface.

- **[API Overview](./api/api_overview.md)** - Complete API introduction.
- **[Authentication API](./api/authentication.md)** - Endpoints for token management.
- **[CURL Examples](./api/curl_examples.md)** - Practical script snippets.

### 💾 7. Backend Storage
Metadata persistence options.

- **[Overview & Comparison](./backend_storage/README.md)** - Choosing the right backend.
- **Implementations**: [PostgreSQL](./backend_storage/postgresql.md), [MongoDB](./backend_storage/mongodb.md), [SQLite](./backend_storage/sqlite.md), [In-Memory](./backend_storage/memory.md).

### 🗄️ 8. Warehouse Storage
Configuring data storage providers.

- **[Warehouse Concepts](./warehouse/README.md)** - How warehouses work in Pangolin.
- **Providers**: [AWS S3](./warehouse/s3.md), [Azure Blob](./warehouse/azure.md), [Google Cloud Storage](./warehouse/gcs.md).

### 🏗️ 9. Architecture
Internal design and developer documentation.

- **[Architecture Overview](./architecture/architecture.md)** - System design and components.
- **[Data Models](./architecture/models.md)** - Core entity definitions.
- **[CatalogStore Trait](./architecture/catalog-store-trait.md)** - Storage abstraction layer.
- **[Signer Trait](./architecture/signer-trait.md)** - Credential vending interface.

### 🧪 10. Utilities & Testing
Tools for validation and maintenance.

- **[PyIceberg Testing](./features/pyiceberg_testing.md)** - Compatibility verification.
- **[OpenAPI Regeneration](./utilities/regenerating-openapi.md)** - Updating the REST spec.
- **[Audit Log Deployment](./DEPLOYMENT_GUIDE_AUDIT_LOGGING.md)** - Advanced audit log setup.

---

**Last Updated**: December 2025  
**Version**: Alpha  
