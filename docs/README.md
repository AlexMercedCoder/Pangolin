# Pangolin Documentation

Welcome to the comprehensive documentation for **Pangolin**, the cloud-native Apache Iceberg REST Catalog. Use the categories below to navigate the guides, feature deep-dives, and tool references.

---

## 🏁 1. Getting Started
*Quickest path from zero to a running lakehouse.*

- **[Onboarding Index](./getting-started/README.md)** - **Start Here!**
- **[Installation Guide](./getting-started/getting_started.md)** - Run Pangolin in 5 minutes.
- **[Evaluating Pangolin](./getting-started/evaluating-pangolin.md)** - Rapid local testing with `NO_AUTH` mode.
- **[Deployment Guide](./getting-started/deployment.md)** - Local, Docker, and Production setup.
- **[Environment Variables](./getting-started/env_vars.md)** - Complete system configuration reference.

---

## 🏗️ 2. Core Infrastructure
*Managing the foundations: storage and metadata.*

- **[Infrastructure Features](./features/README.md)** - Index of all platform capabilities.
- **[Warehouse Management](./warehouse/README.md)** - Configuring S3, Azure, and GCS storage.
- **[Metadata Backends](./backend_storage/README.md)** - Memory, Postgres, MongoDB, and SQLite.
- **[Asset Management](./features/asset_management.md)** - Tables, Views, and CRUD operations.
- **[Federated Catalogs](./features/federated_catalogs.md)** - Proxying external REST catalogs.
- **[Known Issues](./known-issues/README.md)** - Documented limitations and active bugs (e.g., SQL backend quirks).

---

## ⚖️ 3. Governance & Security
*Multi-tenancy, RBAC, and auditing.*

- **[Security Concepts](./features/security_vending.md)** - Identity and Credential Vending principles.
- **[Credential Vending (IAM Roles)](./features/iam_roles.md)** - Scoped cloud access (STS, SAS, Downscoped).
- **[Permission System](./permissions.md)** - Understanding RBAC and granular grants.
- **[Service Users](./features/service_users.md)** - Programmatic access and API key management.
- **[Audit Logging](./features/audit_logs.md)** - Global action tracking and compliance.

---

## 🧪 4. Data Life Cycle
*Git-for-Data and maintenance workflows.*

- **[Branch Management](./features/branch_management.md)** - Working with isolated data environments.
- **[Merge Operations](./features/merge_operations.md)** - The 3-way merge workflow.
- **[Merge Conflicts](./features/merge_conflicts.md)** - Theory and resolution strategies.
- **[Business Metadata & Discovery](./features/business_catalog.md)** - Search, tags, and access requests.
- **[Maintenance Utilities](./features/maintenance.md)** - Snapshot expiration and compaction.

---

## 🛠️ 5. Interfaces & Integration
*Connecting tools and using our management layers.*

- **[Management UI](./ui/README.md)** - Visual guide to the administration portal.
- **[PyPangolin SDK (Official)](../pypangolin/README.md)** - Rich Python client with Git-like operations and types.
- **[PyIceberg Integration](./pyiceberg/README.md)** - Native Python client configuration.
- **[CLI Reference](./cli/README.md)** - Documentation for `pangolin-admin` and `pangolin-user`.
- **[API Reference](./api/README.md)** - Iceberg REST and Management API specs.

---

## 🏗️ 6. Architecture & Internals
*Deep-dives for developers and contributors.*

- **[Architecture Overview](./architecture/README.md)** - System design and component interaction.
- **[Technical Logic Deep-Dive](./architecture/README.md)** - Caching, Branching (Git-for-Data), and Trait details.
- **[Data Models](./architecture/models.md)** - Understanding the internal schema.
- **[CatalogStore Trait](./architecture/catalog-store-trait.md)** - Extending Pangolin storage.
- **[Developer Utilities](./utilities/README.md)** - Tools for contributors (e.g. OpenAPI generation).

---

## 🎓 7. Best Practices
*Production guides and operational wisdom.*

- **[Best Practices Index](./best-practices/README.md)** - Complete guide to operating Pangolin.
- **[Deployment & Security](./best-practices/deployment.md)** - Production checklists.
- **[Scalability](./best-practices/scalability.md)** - Tuning for high performance.
- **[Iceberg Tuning](./best-practices/iceberg.md)** - Optimizing table layout and compaction.

---

**Last Updated**: December 2025  
**Project Status**: Alpha
