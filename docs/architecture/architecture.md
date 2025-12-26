# Pangolin Architecture

## Overview
Pangolin is a Rust-based, multi-tenant, branch-aware lakehouse catalog. It is fully compatible with the Apache Iceberg REST API while providing enterprise-grade extensions for Git-like branching, unified discovery, and cross-catalog federation.

## Core Components

### 1. API Layer (`pangolin_api`)
- **Framework**: Axum (Async Rust).
- **Core Engine**: Handles HTTP routing, JSON (de)serialization, and OpenAPI schema generation via `utoipa`.
- **Key Modules**:
    - `iceberg_handlers`: Faithful implementation of the Iceberg REST specification (Namespaces, Tables, Scans).
    - `branch_handlers`: Logic for Git-like workflows (Branching, Tagging, Merging).
    - `business_metadata_handlers`: Business catalog features (Search, Tags, Access Requests).
    - `auth_handlers`: Multi-mode authentication logic (JWT, API Keys, OAuth2).
    - `management_handlers`: Administrative CRUD for Tenants, Warehouses, and Users.
    - `federated_handlers`: Proxy logic for external REST catalogs.

### 2. Core Domain (`pangolin_core`)
- **Responsibility**: Defines the system's "Source of Truth" models and validation logic.
- **Key Models**:
    - `Tenant`: The root multi-tenancy unit; all data is isolated by Tenant ID.
    - `Asset`: Unified representation of `IcebergTable`, `View`, and other data resources.
    - `Branch`: References to commit chains (`Ingest` vs `Experimental` types).
    - `Commit`: Immutable snapshots of catalog state tracking `Put` and `Delete` operations.
    - `PermissionScope`: Granular target definitions (Tenant, Catalog, Namespace, Asset, Tag).

### 3. Storage Layer (`pangolin_store`)
- **Metadata Persistence**: Abstracted via the `CatalogStore` trait.
    - **Modular Backends**: All backends are refactored into focused submodules (e.g., `tenants.rs`, `warehouses.rs`, `assets.rs`) for better maintainability.
    - `MemoryStore`: Concurrent in-memory store for rapid development/testing.
    - `PostgresStore`: SQL backend using `sqlx` for production scale.
    - `MongoStore`: Document backend for high-availability deployments.
    - `SqliteStore`: Embedded backend for local dev and edge use cases.
- **Performance**: Direct `assets_by_id` lookup for O(1) authorization checks.
- **Data Storage**: Object storage (S3/GCS/Azure) via the `object_store` crate.
- **Credential Vending**: Integrated `Signer` trait to vend temporary tokens (AWS STS, Azure SAS, GCP Downscoped).

### 4. Security & Isolation
- **Authentication**:
    - **JWT**: Standard for UI and corporate identity access.
    - **API Keys**: Managed via **Service Users** for machine-to-machine/CI-CD access. Includes automatic rotation and usage tracking.
    - **OAuth 2.0 / OIDC**: Native integration with Google, Microsoft, GitHub, and custom providers.
- **Authorization**:
    - **RBAC**: Role-based access control with 3 default tiers (Root, TenantAdmin, TenantUser).
    - **TBAC**: Tag-based access control allowing permissions to flow to assets with specific business labels.
    - **Access Requests**: Integrated workflow for users to request access to restricted assets via the UI.
- **Tenant Isolation**: Strictly enforced at the middleware layer; all store queries are scoped by `tenant_id`.

### 5. Git-like Data Lifecycle
- **Branching Engine**: Supports full and partial catalog branching.
- **Fork-on-Write**: Writes to a branch create new commits without affecting the parent branch until merged.
- **3-Way Merging**: Automated conflict detection using common ancestor (base commit) analysis.
- **Conflict Types**: Detects Schema, Data (partition overlap), Metadata, and Deletion conflicts.

### 6. Catalog Federation
- **REST Proxy**: Pangolin acts as a unified entry point, proxying requests to external Iceberg REST catalogs.
- **Auth Translation**: Translates Pangolin credentials to the authentication required by remote catalogs (Basic, Bearer, ApiKey).
- **Global Governance**: Apply Pangolin RBAC and Audit policies even to external federated data.

## Management Interfaces

### 1. Management UI (`pangolin_ui`)
- **Stack**: SvelteKit + Vanilla CSS (modern, glassmorphic design).
- **Features**: Visual management of Users, Permissions, Audit Logs, and the Data Discovery portal.

### 2. CLI Tools
- **pangolin-admin**: High-level system administration (Tenants, Warehouses, Roles).
- **pangolin-user**: Engineering workflow tool (Branching, Code generation, Search).

## Request Flow
1. **Auth Middleware**: Resolves user identity from Header. Validates JWT or API Key.
2. **Tenant Middleware**: Resolves `X-Pangolin-Tenant` or extracts it from User session.
3. **Handler**: Executes business logic. Interacts with `CatalogStore` for metadata.
4. **Vending (Optional)**: If requesting a table load, vends temporary S3/Cloud credentials.
5. **Audit Middleware**: Asynchronously logs the operation result (Success/Failure) to the audit store.

## Testing Strategy
- **Store Tests**: Generic test suite run against all 4 backends (Memory, PG, Mongo, SQLite).
- **API Tests**: Axum integration tests covering all RBAC permutations.
- **Live Verification**: End-to-end scripts using PyIceberg, Spark, and the Pangolin CLI.
