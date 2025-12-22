# Pangolin API Handlers

This document lists the handler modules responsible for the API implementation, grouped by their domain/functionality.

## Iceberg Core
**File**: `iceberg_handlers.rs`
**Purpose**: Implements the Apache Iceberg REST Catalog Specification.
- `get_iceberg_catalog_config_handler`: Catalog configuration endpoint.
- `list_namespaces` / `create_namespace` / `delete_namespace`: Namespace management.
- `list_tables` / `create_table`: Table lifecycle.
- `load_table` / `update_table` / `delete_table`: Table operations.
- `report_metrics`: Metrics reporting.

## Tenant & Storage Management
**Files**: 
- `tenant_handlers.rs`
- `warehouse_handlers.rs`
**Purpose**: Manages the multi-tenant architecture and physical storage layers.
- **Tenants**: Create, list, details, delete tenants.
- **Warehouses**: Configure S3/GCS/Azure storage buckets and vending strategies.

## Catalog Management
**Files**: 
- `pangolin_handlers.rs` (Partial)
- `federated_catalog_handlers.rs`
**Purpose**: Manages logical catalogs and federation.
- **Catalogs**: Create local or federated catalogs.
- **Federated**: Sync, status checks, and configuration for external catalogs.

## Access Control & Authentication
**Files**:
- `auth_middleware.rs`
- `oauth_handlers.rs`
- `token_handlers.rs`
- `permission_handlers.rs`
- `user_handlers.rs`
- `service_user_handlers.rs`
**Purpose**: Handles identity, authentication, and authorization.
- **OAuth**: Google/GitHub login flows.
- **Tokens**: JWT generation, rotation, and revocation.
- **Permissions**: Role-based access control (RBAC), granting/revoking roles.
- **Users**: User profiles and service accounts.

## Data Versioning (Git-for-Data)
**Files**:
- `pangolin_handlers.rs` (Partial)
- `merge_handlers.rs`
- `conflict_detector.rs`
**Purpose**: Implements Nessie-like branching and merging capabilities.
- **Branches**: Create, list, delete branches (`main`, `dev`, etc.).
- **Commits**: View commit history.
- **Tags**: Tag specific commits.
- **Merges**: Create merge operations, detect conflicts, resolving conflicts (3-way merge).

## Governance & Metadata
**Files**:
- `audit_handlers.rs`
- `business_metadata_handlers.rs`
- `signing_handlers.rs`
**Purpose**: Data governance, auditing, and secure access vending.
- **Audit**: Immutable log of all system actions.
- **Business Metadata**: Tagging, descriptions, and ownership metadata.
- **Signing**: Vending temporary AWS/Azure/GCP credentials for data access (S3/ADLS/GCS).

## Optimization & Search
**Files**:
- `optimization_handlers.rs`
- `dashboard_handlers.rs`
**Purpose**: High-performance endpoints for UI and data discovery.
- **Unified Search**: Multi-backend optimized search (Assets, Namespaces, Catalogs).
- **Dashboard**: Aggregated statistics for finding data faster.
- **Validation**: Name validation logic.

## System
**Files**:
- `system_config_handlers.rs`
- `cleanup_job.rs`
**Purpose**: Global system settings and background maintenance.
- **Config**: SMTP settings, default retention policies.
- **Cleanup**: Background jobs for expired tokens and orphan files.
