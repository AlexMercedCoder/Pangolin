# Pangolin Features Guide

This directory contains detailed documentation for the core features of the Pangolin platform.

## 🗂️ Asset & Data Management
Basic data operations and lifecycle management.
- **[Entities](./entities.md)**: Understanding the Pangolin core models.
- **[Asset Management](./asset_management.md)**: Handling Tables, Views, and other assets.
- **[Generic Assets](./generic_assets.md)**: Cataloging ML Models, Videos, and Files.
- **[Modern Table Formats](./table_formats.md)**: Support for Delta Lake, Hudi, and Paimon.
- **[Warehouse Management](./warehouse_management.md)**: Configuring storage backends.
- **[Time Travel](./time_travel.md)**: Querying historical data states.
- **[Tag Management](./tag_management.md)**: Versioning with tags.

## 🌿 Versioning & Branches
Git-like workflows for your data lake.
- **[Branch Management](./branch_management.md)**: Creating and managing branches.
- **[Merge Operations](./merge_operations.md)**: Workflow for 3-way merges.
- **[Merge Conflicts](./merge_conflicts.md)**: Principles of data conflict resolution.

## 🛡️ Security & Access
Governance and authentication mechanisms.
- **[Multi-Tenancy](./multi_tenancy.md)**: Isolation principles and tenant management.
- **[RBAC](./rbac.md)**: Role-Based Access Control system.
- **[Credential Vending](./iam_roles.md)**: Cloud IAM and STS integration.
- **[Security Concepts](./security_vending.md)**: Concept guide for vending and signing.

## 🔍 Discovery & Audit
Finding data and tracking changes.
- **[Business Catalog](./business_catalog.md)**: Data discovery portal and business metadata.
- **[Audit Logging](./audit_logs.md)**: Comprehensive action tracking and compliance.

## 🌐 Integration & Federation
Connecting to external systems.
- **[Federated Catalogs](./federated_catalogs.md)**: Connecting to remote Iceberg catalogs.
- **[Catalog Management](./catalog_management.md)**: Managing local and federated catalogs.
- **[Service Users](./service_users.md)**: API keys for programmatic access.

## 🧪 Integration & Testing
Using Pangolin with other tools.
- **[PyIceberg Integration](./pyiceberg_testing.md)**: Guide for Python users.

## 🛠️ Maintenance
Optimizing and maintaining your data lake.
- **[Maintenance Operations](./maintenance.md)**: Snapshot management and orphan file cleanup.

---

## Feature Status Matrix

| Feature | Category | Status |
|---------|----------|--------|
| Multi-Tenancy | System | ✅ Implemented |
| Branching | Versioning | ✅ Implemented |
| 3-Way Merge | Versioning | ✅ Implemented |
| RBAC | Security | ✅ Implemented |
| Audit Logging | Security | ✅ Implemented |
| Business Catalog | Discovery | ✅ Implemented |
| Credential Vending (Static) | Security | ✅ Implemented |
| Credential Vending (STS) | Security | 🚧 Beta |
| Federated Catalogs | Integration | ✅ Implemented |
| View Management | Data | ✅ Implemented |
