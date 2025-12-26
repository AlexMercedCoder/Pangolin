# Pangolin API Handlers

This document lists the handler modules responsible for the API implementation, grouped by their domain/functionality.

## Iceberg Core
**File**: `iceberg_handlers.rs` (Refactor to `iceberg/` in progress)
**Purpose**: Implements the Apache Iceberg REST Catalog Specification.
- `get_iceberg_catalog_config_handler`: Catalog configuration endpoint.
- `list_namespaces` / `create_namespace` / `delete_namespace`: Namespace management.
- `list_tables` / `create_table`: Table lifecycle.
- `load_table` / `update_table` / `delete_table`: Table operations.
- `report_metrics`: Metrics reporting.
- **Planned Refactor**: See [Iceberg Modularization Plan](../../planning/modularization_plan_iceberg.md).

## Tenant & Storage Management
**Files**: 
- `tenant_handlers.rs`: Create, list, details, delete tenants.
- `warehouse_handlers.rs`: Configure S3/GCS/Azure storage buckets and vending strategies.
- `asset_handlers.rs`: Generic asset management (Tables, Views, etc.) outside the Iceberg scope.

## Catalog Management
**Files**: 
- `pangolin_handlers.rs`: Core catalog, branch, and tag management.
- `federated_catalog_handlers.rs`: Sync, status, and config for external catalogs.
- `federated_proxy.rs`: Proxy logic for forwarding requests to federated backends.

## Access Control & Authentication
**Files**:
- `auth_middleware.rs`: JWT and API key validation layer.
- `oauth_handlers.rs`: Google/GitHub/Microsoft OIDC flows.
- `token_handlers.rs`: Token generation, rotation, and revocation.
- `permission_handlers.rs`: RBAC management (Role CRUD, assignments, grants).
- `user_handlers.rs`: User profile management and basic login.
- `service_user_handlers.rs`: Machine-to-machine account management and API key rotation.

## Data Versioning (Git-for-Data)
**Files**:
- `merge_handlers.rs`: Merge operation lifecycle (Start, Complete, Abort).
- `conflict_detector.rs`: Logic for detecting schema and data conflicts between branches.
- `pangolin_handlers.rs` (Versioning parts): Branching and tagging logic.

## Governance & Metadata
**Files**:
- `audit_handlers.rs`: Enhanced audit logging with filtering and count endpoints (Isolated by user/resource).
- `business_metadata_handlers.rs`: Business tagging, descriptions, access requests, and ownership.
- `signing_handlers.rs`: Endpoint for vending temporary cloud credentials.

## Optimization & Search
**Files**:
- `optimization_handlers.rs`: Unified search, bulk operations, and name validation.
- `dashboard_handlers.rs`: Aggregated statistics for the UI.

## System & Background
**Files**:
- `system_config_handlers.rs`: Global settings (SMTP, defaults).
- `cleanup_job.rs`: Background worker for expired tokens.

## CLI Admin Handlers
**File**: `pangolin_cli_admin/src/handlers.rs` (Refactor to `handlers/` in progress)
- **Planned Refactor**: See [CLI Modularization Plan](../../planning/modularization_plan_cli.md).
