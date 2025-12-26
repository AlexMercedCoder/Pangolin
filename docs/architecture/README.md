# Architecture Documentation

This directory contains detailed technical documentation for the Pangolin architecture.

## 🏗️ Core Structure
- **[High-Level Architecture](./architecture.md)**: Overall system design, component interaction, and multi-tenant isolation.
- **[API Handlers](./handlers.md)**: Map of API endpoints categorized by functional domain (Iceberg, Versioning, Admin).
- **[Models](./models.md)**: Comprehensive guide to core system structs (Tenant, Asset, Merge, User).
- **[Enums](./enums.md)**: Exhaustive list of system enumerations and their serialized values.

## 🔧 Interfaces & Logic
- **[System Traits](./traits.md)**: In-depth look at `CatalogStore` and `Signer` interfaces.
- **[Branching & Merging](./branching.md)**: Operational details of the "Git-for-Data" versioning model.
- **[Caching Strategy](./caching.md)**: multi-layered performance optimizations for metadata and cloud backends.

## 🔐 Security & Operations
- **[Authentication](./authentication.md)**: Deep dive into JWT, Service User API Keys, and RBAC.
- **[Storage & Connectivity](./storage_and_connectivity.md)**: Cloud connectivity, modular store structure, and credential vending.
- **[Dependencies](./dependencies.md)**: Final list of technology stack and library versions.

---

## 📅 Status
All documents in this directory were audited and updated in **December 2025** to reflect the modularized backend and enhanced security features.
