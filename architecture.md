# Pangolin Architecture

## Overview
Pangolin is a Rust-based, multi-tenant, branch-aware lakehouse catalog. It is designed to be compatible with the Apache Iceberg REST API while providing extended capabilities for branching, tagging, and managing non-Iceberg assets.

## Core Components

### 1. API Layer (`pangolin_api`)
- **Framework**: Axum (Async Rust).
- **Responsibility**: Handles HTTP requests, routing, serialization, and auth.
- **Key Modules**:
    - `iceberg_handlers`: Standard Iceberg REST specification implementation.
    - `pangolin_handlers`: Extended capabilities (branching, commits).
    - `business_metadata_handlers`: Business catalog features (metadata, access requests).
    - `user_handlers`, `permission_handlers`: RBAC and user management.
    - `service_user_handlers`: Service user management and API key operations.
    - `middleware`: JWT authentication, API key authentication, and tenant context resolution.

### 2. Core Domain (`pangolin_core`)
- **Responsibility**: Defines shared data models, traits, and business logic.
- **Key Models**:
    - `Tenant`: Multi-tenancy root entity.
    - `Asset`: Unified representation of Tables, Views, and other resources.
    - `BusinessMetadata`: Descriptive metadata, tags, and properties.
    - `User`, `Role`, `Permission`: RBAC entities.
    - `ServiceUser`: Programmatic identities with API key authentication.
    - `Branch`: Git-like commit history reference.

### 3. Storage Layer (`pangolin_store`)
- **Responsibility**: Abstract persistence layer via `CatalogStore` trait.
- **Backend Storage (Metadata)**:
    - `MemoryStore`: High-performance, concurrent in-memory store (using `DashMap`) for testing/dev.
    - `PostgresStore`: ✅ Production-ready relational storage with ACID guarantees.
    - `MongoStore`: ✅ Production-ready NoSQL document storage with horizontal scalability.
    - `SqliteStore`: ✅ Production-ready embedded storage for development and edge deployments.
- **Warehouse Storage (Data)**:
    - S3/GCS/Azure: Object storage for Iceberg table data files (via `object_store` crate).
    - **Credential Vending**: AWS STS AssumeRole, Azure OAuth2, GCP service account tokens.
    - **Cloud Features**: Optional feature flags (`aws-sts`, `azure-oauth`, `gcp-oauth`) for production credential vending.
- **Merge Operations**: Tracks merge operations and conflicts with full lifecycle management.

### 4. Security & Authentication
- **Modes**:
    - **No Auth**: Open access for development (`PANGOLIN_NO_AUTH=true`).
    - **JWT**: Bearer token authentication with `bcrypt` password hashing.
    - **API Keys**: Service user authentication via `X-API-Key` header.
    - **OAuth 2.0**: OIDC integration with Google, Microsoft, GitHub, Okta.
- **Authorization**:
    - **RBAC**: 3-tier role system (Root, TenantAdmin, TenantUser).
    - **Service Users**: Dedicated programmatic identities for CI/CD, ETL, and automation.
    - **Granular Logic**: Scope-based permissions (Catalog, Namespace, Asset, Tag).

### 5. Merge Conflict Resolution
- **Conflict Detection**: Automatic detection of schema, deletion, metadata, and data overlap conflicts.
- **Conflict Types**: 4 categories with specific detection algorithms.
- **Resolution Strategies**: Manual (TakeSource, TakeTarget, Custom) and automatic resolution.
- **Lifecycle Management**: Full tracking from initiation to completion/abort.
- **API Endpoints**: 6 dedicated endpoints for managing merge operations and conflicts.

### 6. Federated Catalogs
- **Transparent Proxy**: Connect to external Iceberg REST catalogs seamlessly.
- **Unified Authentication**: Users authenticate to Pangolin, not individual catalogs.
- **Cross-Tenant Federation**: Tenant A can access Tenant B's catalogs through Pangolin.
- **Multiple Auth Types**: Support for None, BasicAuth, BearerToken, and ApiKey.
- **RBAC Integration**: Pangolin permissions apply to federated catalogs.
- **Management API**: 5 endpoints for creating, listing, testing, and deleting federated catalogs.

### 7. Federated Catalogs
- **All Entities**: Full create, read, update, delete operations for:
    - Tenants: 5 endpoints (list, create, get, update, delete)
    - Warehouses: 5 endpoints (list, create, get, update, delete)
    - Catalogs: 5 endpoints (list, create, get, update, delete)
    - Users: 5 endpoints (list, create, get, update, delete)
    - Roles & Permissions: 8 endpoints (full RBAC management)
- **Backend Support**: All operations implemented across Memory, SQLite, PostgreSQL, and MongoDB stores.
- **Production Ready**: Comprehensive error handling, validation, and logging.

### 8. Management UI (`pangolin_ui`)
- **Framework**: SvelteKit + TailwindCSS.
- **Goal**: Provide a modern visual interface for catalog management, RBAC, and data discovery.
- **Status**: Alpha (Data Explorer, Auth, and Basic Administration implemented).

### 9. CLI Tools
- **Pangolin Admin (`pangolin-admin`)**: Administrative CLI for managing tenants, warehouses, catalogs, users, and permissions.
- **Pangolin User (`pangolin-user`)**: User-facing CLI for discovery, branching, and access requests.
- **Architecture**:
    - `pangolin_cli_common`: Shared configuration and API client code.
    - `pangolin_cli_admin`: Admin specific commands.
    - `pangolin_cli_user`: User specific commands.

## Data Model

### Branching Model
Pangolin uses a Git-like branching model:
- **Main Branch**: The default branch for production data.
- **Ingest Branch**: For isolating write operations.
- **Experimental Branch**: For testing and experimentation.

### Asset Identification
Assets are identified by `Tenant -> Namespace -> Name`.
To support branching in standard Iceberg clients, we use a `table@branch` syntax in the table name.

## Request Flow
1. **Request**: Client sends HTTP request (e.g., `GET /v1/ns/tables/mytable@dev`).
2. **Middleware**: Resolves Tenant ID from Auth header.
3. **Handler**: Parses `mytable@dev` to extract table name `mytable` and branch `dev`.
4. **Store**: Queries the `CatalogStore` for the asset on the specified branch.
5. **Response**: Returns metadata or error.

## Testing Strategy
- **Unit Tests**: Cover core logic in `pangolin_core` (serialization) and `pangolin_store` (CRUD operations).
- **Integration Tests**: End-to-end API tests in `pangolin_api` using `axum::test` and `tower::ServiceExt`.
- **Manual Verification**: `curl` based verification for new features.
