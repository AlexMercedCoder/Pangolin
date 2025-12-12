# Pangolin  
## A Rust-Based, Multi-Tenant, Iceberg-Compatible Lakehouse Catalog  
### Full Technical Specification for AI-Assisted Implementation

This document defines the architecture, data model, API surface, security model, and operational requirements for **Pangolin**, a Rust-based, branch-aware, multi-tenant catalog service designed for modern lakehouse architectures. Pangolin provides Git-style versioning, tracks any lakehouse asset, supports Apache Iceberg REST Catalog functionality, and includes a management UI for governance and observability.

---

# 1. Core Objectives

- Deliver a **transactional, branch-aware metadata catalog** supporting branching, tagging, merging, and time-travel.
- Fully implement the **Apache Iceberg REST Catalog API**. (located in rest-catalog-open-api.yaml)
- Track **any lakehouse asset type**, not only Iceberg tables.
- Support **multi-tenancy** with strong isolation at metadata and storage levels.
- Provide **RBAC and TBAC** controls with OAuth2/OIDC-based authentication.
- Integrate with **S3, GCS, ADLS, and S3-compatible** object stores.
- Provide **credential vending** and **request signing** for secure access to object storage.
- Ship with a **robust catalog management UI** featuring search, audit logs, branch navigation, lineage, and multi-tenant administration.
- Architect for **horizontal scalability**, **high reliability**, and future extensibility.
- Robust unit tests and integration tests.

---

# 2. Functional Overview

## 2.1 Branch-Aware Catalog Semantics
Pangolin must support:

When you create a branch you select tables to be part of that branch. That branch will track the current metadata.json for that table for transactions on that branch.

There are two types of branches:
- ingest branches: There are for isolating changes meant to be merged into the main branch, there can only be one ingest branch per table.
- experimental branches: This is a branch that is isolating changes not meant to be merged into the main branch.

- Branch creation, deletion, and listing  
- Tag creation and deletion  
- Commit operations with multiple asset updates  
- Optimistic concurrency control  
- Merge operations with conflict detection at asset granularity  
- Time-travel access using branch and timestamp selectors  
- Periodic cleanup task that'll detect which branches were created from expired snapshots on main and clean up the storage for those branches. Should maintain an audit log of cleanup activities.

## 2.2 Multi-Tenancy
Each tenant operates in a fully isolated environment:

The hierarchy is as follows:

- Tenant (one deployment of Pangolin can have multiple tenants)
- Catalog (one tenant can have multiple catalogs)
- Namespace (catalogs can have multiple namespaces)
- Asset (namespaces can have multiple assets)

- Multi-table-scoped branches  
- Catalog-scoped asset namespaces  
- Catalog / Namespace / Role based -specific storage warehouses (A catalog level warehouse is required, but if a namespace level warehouse is specified, it will be used for that namespace, and if a role level warehouse is specified, it will be used for that role. Hierarchy being Catalog > Namespace > Role)  
- Catalog / Namespace / Asset based -specific RBAC and TBAC rules  
- Soft and hard isolation options  
- Support for global administrators who can manage all tenants  
- UI-level tenant switching and tenant quota management  

## 2.3 Universal Lakehouse Asset Model
Every tracked asset includes:

- Name  
- Type (e.g., ICEBERG_TABLE, DELTA_TABLE, HUDI_TABLE, PARQUET_TABLE, CSV_TABLE, JSON_TABLE, PDF_TABLE, IMAGE_TABLE, VIDEO_TABLE, AUDIO_TABLE, TEXT_TABLE, VIEW, ML_MODEL, FEATURE_SET)  
- Storage location URI  
- Properties (arbitrary key–value metadata for format-specific details)

---

# 3. API Design

## 3.1 Iceberg REST Catalog API (Full Spec)
Pangolin must support:

- Namespace listing, creation, and deletion  
- Table listing, retrieval, creation, commit/update, and deletion  
- Branch-aware endpoints (`/iceberg/v1/{branch}/...`)  
- Credential fields returned when necessary  
- Strict optimistic concurrency  
- Standardized error formats and status codes  

This should work only with Iceberg tables, and return an a graceful error for non-iceberg tables to use extended API.

## 3.2 Pangolin Catalog API (Extended)
Endpoints under `/api/v1` provide:

- Branch operations: create, list, delete, assign, merge  
- Commit history and diff operations  
- CRUD for non-Iceberg assets  
- Tenant lifecycle operations: create tenant, update quotas, delete tenant  
- Tenant audit log access  
- Storage warehouse administration  
- CRUD operations for non-iceberg assets including vending credentials and signing requests for these assets

## 3.3 Multi-Tenant API Routing
Two supported mechanisms:

- Path-based: `/tenant/{tenantId}/api/...`  
- Token-derived: tenant identity inferred from JWT claim  

---

# 4. Metadata Model

## 4.1 Commit
Each commit includes:

- Commit ID  
- Parent commit ID  
- Timestamp  
- Author identity  
- Message  
- List of operations:  
  - Put (adds or updates an asset)  
  - Delete (removes an asset)  
  - Unmodified (asserts no change; supports conflict detection)  

## 4.2 Content Key
Hierarchical identifier:

- One or more path segments (e.g., namespace and table name)  
- Prefixed or scoped by tenant  

## 4.3 Asset (Content Object)
Each asset includes:

- Name  
- Type  
- Storage location URI  
- Properties: arbitrary key–value metadata  

## 4.4 Branch
A named reference to a commit:

- Branch name  
- Head commit hash  
- Tables/Assets tracked by branch
- Type of Branch (ingest/experimental)

## 4.5 Tag
Immutable reference to a commit:

- Tag name  
- Associated commit hash  

## 4.6 Tenant
A tenant includes:

- Tenant ID  
- Display name  
- Tenant-specific storage warehouse mappings  
- RBAC/TBAC configuration  
- Metadata quota and storage quota  

## 4.7 Storage Warehouse
A warehouse includes:

- Warehouse ID  
- Base URI  
- Cloud provider  
- Credential configuration  
- Optional assume-role or delegated identity  

---

# 5. Cloud Storage Integration

## 5.1 Required Providers
- AWS S3  
- Google Cloud Storage  
- Azure ADLS Gen2  
- S3-compatible stores  
- File (For Development)

## 5.2 Credential Modes
- Credential vending (STS/GCS tokens/SAS tokens)  
- Request signing (SigV4, GCS-style signatures, SAS)  

## 5.3 Warehouse-Level Configuration
Each warehouse defines:

- Identifier  
- Cloud provider type  
- Endpoint or base URI  
- Mechanism for credential issuance  

---

# 6. Security Architecture

- Should have a pluggable security architecture
- Should support authentication and authorization
- Should have "no auth" mode for development

## 6.1 Authentication
- OAuth2 and OIDC with JWT bearer tokens  
- Optional PAT support for automation  
- Support for enterprise identity providers (Okta, Auth0, Azure AD, Keycloak)  

## 6.2 Authorization
### RBAC
- Roles: admin, engineer, analyst, viewer  
- Permissions mapped to:
  - Branch management  
  - Asset CRUD  (each should be a seperate permission per catalog, namespace and asset type)
  - Commit operations  
  - Tenant administration  
  - Warehouse administration  

### TBAC
- Policies determined by:  
  - User attributes (e.g., region, clearance)  
  - Asset tags (e.g., sensitivity level, domain)  

### Multi-Tenant Enforcement
- Tenants cannot access each other's:  

  - Catalogs
  - Namespaces
  - Branches  
  - Assets  
  - Commits  
  - Warehouses  
  - Logs  
- UI and API must enforce tenant boundaries consistently.

---

# 7. Pangolin Management UI

### Required Features
- Tenant overview dashboard  
- Branch browser with commit graph visualization  
- Commit detail views  
- Asset explorer with schema and property inspection  
- Lineage visualization for all asset types  
- Audit log viewer  
- Merge and conflict resolution tools  
- User and role management  
- Storage usage dashboards per tenant  
- Time-travel browser for visualizing historical states  

### Technology Requirements
- Should communicate with Pangolin API  
- Should support multi-tenant-aware session handling  
- Should render diffs, lineage, and branches with interactive components  

---

# 8. Storage Layer Implementation

Pangolin will use an embedded or pluggable metadata store. Required supporting structures:

- Should support SQL Databases and MongoDB for persistence
- Should support in memory storage for development

### Branch Storage
- Key: branch name  
- Value: commit hash  

### Commit Storage
- Key: commit hash  
- Value: commit record (metadata + operations)  

### Asset Storage
- Key: hash + content key  
- Value: asset object  

### Tenant Storage
- Key: tenant ID  
- Value: tenant configuration  

### Warehouse Storage
- Key: warehouse ID  
- Value: warehouse definition  

---

# 9. System Architecture and Libraries

### Recommended Rust Libraries
- Axum (HTTP framework)  
- Tokio (runtime)  
- Serde (serialization)  
- RocksDB or Sled (metadata storage)  
- AWS/GCP/Azure SDKs  
- object_store crate for unified storage abstraction  
- jsonwebtoken for JWT validation  
- oauth2 for OAuth flows  
- Oso or homemade engine for RBAC/TBAC policies  

---

# 10. MVP Scope

The MVP release must include:

- Branching, tagging, commit workflow  
- Multi-tenant metadata isolation  
- Iceberg REST Catalog operations  
- Basic RBAC  
- S3-only storage backend (GCS/ADLS optional)
- Support for S3-compatible storage
- dockerfile for easy deployment and a docker-compose configuring the catalog with minio
- throughou documentation, with a markdown for each feature, a markdown covering environmental variables, markdowns for each api, markdown for deployment, markdown for configuring each storage type  
- Request signing or credential vending for S3  
- Minimal management UI:  
  - List branches  
  - List assets  
  - View commits  

---

# 11. Post-MVP Enhancements

- Full tenant onboarding workflows  
- Advanced conflict resolution UI  
- Multi-format asset helpers (Delta, Hudi, ML models)  
- Multi-region warehouse support  
- Horizontal scaling with shared metadata backend  
- Complete governance dashboards  
- Automated data lineage mapping  
- Export/import tools for disaster recovery  

---

# Pangolin Summary

Pangolin is designed to become a **universal, multi-tenant, Iceberg-compatible, branch-aware lakehouse catalog** built in Rust. It provides strong governance, multi-cloud integration, a flexible asset model, and a UI designed for enterprise data teams.

This specification enables Gemini 3 to generate modular, production-ready Rust code aligned with modern data platform needs.