# Pangolin Documentation Index

Welcome to the comprehensive documentation for Pangolin. Use the index below to find guides for setup, core concepts, feature deep-dives, and tool references.

## 🏁 1. Getting Started
*Everything you need to get up and running.*

- **[Installation Guide](./getting-started/getting_started.md)** - Run Pangolin in 5 minutes.
- **[Authentication Modes](./authentication.md)** - Auth vs No-Auth and OAuth setup.
- **[User Scopes](./getting-started/getting_started.md#user-scopes)** - Root, Tenant Admin, and Tenant User roles.
- **[Environment Variables](./getting-started/env_vars.md)** - Complete reference for all settings.
- **[Configuration](./getting-started/configuration.md)** - Server and storage configuration.
- **[Client Configuration](./getting-started/client_configuration.md)** - Connecting PyIceberg, Spark, and Trino.
- **[Deployment](./getting-started/deployment.md)** - Docker and production setup.

## 🏗️ 2. Core Infrastructure
*Managing warehouses, catalogs, and metadata storage.*

- **[Warehouse Management](./warehouse/README.md)** - Configuring S3, Azure, and GCS.
- **[Catalog Assets](./features/asset_management.md)** - Tables, Views, and Federated proxying.
- **[Backend Metadata](./backend_storage/README.md)** - Postgres, MongoDB, and SQLite backends.

## 🧪 3. Data & Governance
*Managing data lifecycle and security.*

- **[Git-like Branching](./features/branch_management.md)** - Branching, tagging, and forking logic.
- **[Permission System](./permissions.md)** - RBAC, TBAC, and cascading grants.
- **[Merge Operations](./features/merge_operations.md)** - Reconciliation and conflict handling.
- **[Audit Logging](./features/audit_logs.md)** - Tracking actions via API, CLI, and UI.
- **[Maintenance Utilities](./features/maintenance.md)** - Snapshots and orphan file management.
- **[Business Metadata](./features/business_catalog.md)** - Tags, discovery, and search.

## 🛠️ 4. Tools & References
*Direct guides for our interfaces and APIs.*

- **[CLI Reference](./cli/overview.md)** - Admin and User command guides.
- **[API Reference](./api/api_overview.md)** - Iceberg REST and Management endpoints.
- **[Management UI](./ui/overview.md)** - Visual guide to the administration portal.
- **[Service Users](./service_users.md)** - Programmatic access and API keys.

## 🏗️ 5. Architecture & Extending
*Internal design for developers.*

- **[System Architecture](./architecture/architecture.md)** - Design and components.
- **[Data Models](./architecture/models.md)** - Core entity definitions.
- **[Storage Abstractions](./architecture/catalog-store-trait.md)** - Implementation traits.

---

**Last Updated**: December 2025  
**Version**: Alpha  
