# CatalogStore Trait

## Overview
The `CatalogStore` trait is the fundamental abstraction layer for Pangolin's metadata and state persistence. It enforces a strict multi-tenant interface, ensuring that data isolation is handled at the storage-engine level.

**Source**: `pangolin_store/src/lib.rs`

---

## Core Infrastructure Operations

### 1. Tenant Management
Handles the lifecycle of the highest-level isolation units.
- `create_tenant(tenant: Tenant) -> Result<()>`
- `get_tenant(id: Uuid) -> Result<Option<Tenant>>`
- `list_tenants() -> Result<Vec<Tenant>>`
- `update_tenant(id: Uuid, updates: TenantUpdate) -> Result<Tenant>`
- `delete_tenant(id: Uuid) -> Result<()>`

### 2. Warehouse & Catalog Setup
Defines the storage and catalog configurations.
- `create_warehouse(tenant_id: Uuid, warehouse: Warehouse) -> Result<()>`
- `create_catalog(tenant_id: Uuid, catalog: Catalog) -> Result<()>`
- `get_catalog(tenant_id: Uuid, name: String) -> Result<Option<Catalog>>`
- `list_catalogs(tenant_id: Uuid) -> Result<Vec<Catalog>>`

---

## Data & Object Operations

### 3. Namespace & Asset Management
Pangolin uses hierarchical namespaces and versioned assets.
- `create_namespace(tenant_id: Uuid, catalog: &str, namespace: Namespace) -> Result<()>`
- `list_namespaces(tenant_id: Uuid, catalog: &str, parent: Option<String>) -> Result<Vec<Namespace>>`
- `create_asset(tenant_id: Uuid, catalog: &str, branch: Option<String>, namespace: Vec<String>, asset: Asset) -> Result<()>`
- `get_asset(tenant_id: Uuid, catalog: &str, ..., name: String) -> Result<Option<Asset>>`
- `get_asset_by_id(tenant_id: Uuid, asset_id: Uuid) -> Result<Option<(Asset, String, Vec<String>)>>`

### 4. Branching & Tagging
Git-like operations for data versioning.
- `create_branch(tenant_id: Uuid, catalog: &str, branch: Branch) -> Result<()>`
- `merge_branch(tenant_id: Uuid, catalog: &str, source: String, target: String) -> Result<()>`
- `create_tag(tenant_id: Uuid, catalog: &str, tag: Tag) -> Result<()>`
- `delete_tag(tenant_id: Uuid, catalog: &str, name: String) -> Result<()>`

---

## Governance & Security

### 5. IAM (Users, Roles, Permissions)
Granular access control management.
- `create_user(_user: User) -> Result<()>`
- `list_users(_tenant_id: Option<Uuid>) -> Result<Vec<User>>`
- `create_role(_role: Role) -> Result<()>`
- `assign_role(_user_role: UserRole) -> Result<()>`
- `list_user_permissions(_user_id: Uuid) -> Result<Vec<Permission>>`

### 6. Service Users
Programmatic access management.
- `create_service_user(_service_user: ServiceUser) -> Result<()>`
- `get_service_user_by_api_key_hash(_hash: &str) -> Result<Option<ServiceUser>>`

### 7. Audit Logging
System-wide activity tracking.
- `log_audit_event(tenant_id: Uuid, event: AuditLogEntry) -> Result<()>`
- `list_audit_events(tenant_id: Uuid, filter: Option<AuditLogFilter>) -> Result<Vec<AuditLogEntry>>`
- `count_audit_events(tenant_id: Uuid, filter: Option<AuditLogFilter>) -> Result<usize>`

---

## Maintenance & System

### 8. Table Maintenance
Low-level object storage cleanup.
- `expire_snapshots(tenant_id: Uuid, ..., table: String, retention_ms: i64) -> Result<()>`
- `remove_orphan_files(tenant_id: Uuid, ..., table: String, older_than_ms: i64) -> Result<()>`

### 9. System Configuration
- `get_system_settings(tenant_id: Uuid) -> Result<SystemSettings>`
- `update_system_settings(tenant_id: Uuid, settings: SystemSettings) -> Result<SystemSettings>`

### 10. Federated Catalog Operations
- `sync_federated_catalog(tenant_id: Uuid, catalog_name: &str) -> Result<()>`
- `get_federated_catalog_stats(tenant_id: Uuid, catalog_name: &str) -> Result<SyncStats>`

### 11. Token Lifecycle
- `revoke_token(token_id: Uuid, expires_at: DateTime<Utc>, reason: Option<String>) -> Result<()>`
- `is_token_revoked(token_id: Uuid) -> Result<bool>`
- `cleanup_expired_tokens() -> Result<usize>`
- `list_active_tokens(tenant_id: Uuid, user_id: Option<Uuid>, pagination: Option<PaginationParams>) -> Result<Vec<TokenInfo>>`
- `store_token(token: TokenInfo) -> Result<()>`
- `validate_token(token: &str) -> Result<Option<TokenInfo>>`

### 12. Permissions & Roles
- `create_permission(permission: Permission) -> Result<()>`
- `revoke_permission(permission_id: Uuid) -> Result<()>`
- `list_permissions(tenant_id: Uuid, pagination: Option<PaginationParams>) -> Result<Vec<Permission>>`
- `list_user_permissions(user_id: Uuid, pagination: Option<PaginationParams>) -> Result<Vec<Permission>>`

---

## Implementation Requirements
Every `CatalogStore` implementation must:
1.  **Enforce Multi-tenancy**: Every method (where applicable) requires a `tenant_id`. Implementations must never leak data across tenants.
2.  **Thread Safety**: Derive `Send + Sync` to support concurrent API requests.
3.  **Async/Await**: Use `async_trait` for all persistence logic.
4.  **Error Handling**: Return `anyhow::Result` for consistent error propagation up to the API layer.
