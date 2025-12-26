# Pangolin Traits Documentation

This document details the core traits used in the Pangolin architecture to define storage and signing interfaces.

## CatalogStore Trait

**Source**: `pangolin_store/src/lib.rs`

The `CatalogStore` trait is the central interface for all metadata storage backends (Memory, SQLite, PostgreSQL, MongoDB). It extends `Send`, `Sync`, and `Signer`.

### Tenant Operations
- `create_tenant(tenant: Tenant) -> Result<()>`
- `get_tenant(tenant_id: Uuid) -> Result<Option<Tenant>>`
- `list_tenants() -> Result<Vec<Tenant>>`
- `update_tenant(tenant_id: Uuid, updates: TenantUpdate) -> Result<Tenant>`
- `delete_tenant(tenant_id: Uuid) -> Result<()>`

### Warehouse Operations
- `create_warehouse(tenant_id: Uuid, warehouse: Warehouse) -> Result<()>`
- `get_warehouse(tenant_id: Uuid, name: String) -> Result<Option<Warehouse>>`
- `list_warehouses(tenant_id: Uuid) -> Result<Vec<Warehouse>>`
- `update_warehouse(tenant_id: Uuid, name: String, updates: WarehouseUpdate) -> Result<Warehouse>`
- `delete_warehouse(tenant_id: Uuid, name: String) -> Result<()>`

### Catalog Operations
- `create_catalog(tenant_id: Uuid, catalog: Catalog) -> Result<()>`
- `get_catalog(tenant_id: Uuid, name: String) -> Result<Option<Catalog>>`
- `update_catalog(tenant_id: Uuid, name: String, updates: CatalogUpdate) -> Result<Catalog>`
- `delete_catalog(tenant_id: Uuid, name: String) -> Result<()>`
- `list_catalogs(tenant_id: Uuid) -> Result<Vec<Catalog>>`

### Namespace Operations
- `create_namespace(tenant_id: Uuid, catalog_name: &str, namespace: Namespace) -> Result<()>`
- `list_namespaces(tenant_id: Uuid, catalog_name: &str, parent: Option<String>) -> Result<Vec<Namespace>>`
- `get_namespace(tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<Option<Namespace>>`
- `delete_namespace(tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<()>`
- `update_namespace_properties(tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>, properties: HashMap<String, String>) -> Result<()>`
- `count_namespaces(tenant_id: Uuid) -> Result<usize>`

### Asset Operations
- `create_asset(tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, asset: Asset) -> Result<()>`
- `get_asset(tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<Option<Asset>>`
- `get_asset_by_id(tenant_id: Uuid, asset_id: Uuid) -> Result<Option<(Asset, String, Vec<String>)>>`
- `list_assets(tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>) -> Result<Vec<Asset>>`
- `delete_asset(tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<()>`
- `rename_asset(tenant_id, catalog_name, branch, source_namespace, source_name, dest_namespace, dest_name) -> Result<()>`
- `count_assets(tenant_id: Uuid) -> Result<usize>`

### Branch & Tag Operations
- `create_branch(tenant_id: Uuid, catalog_name: &str, branch: Branch) -> Result<()>`
- `get_branch(tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Branch>>`
- `list_branches(tenant_id: Uuid, catalog_name: &str) -> Result<Vec<Branch>>`
- `delete_branch(tenant_id: Uuid, catalog_name: &str, name: String) -> Result<()>`
- `merge_branch(tenant_id, catalog_name, source_branch, target_branch) -> Result<()>`
- `create_tag(tenant_id: Uuid, catalog_name: &str, tag: Tag) -> Result<()>`
- `get_tag(tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Tag>>`
- `list_tags(tenant_id: Uuid, catalog_name: &str) -> Result<Vec<Tag>>`
- `delete_tag(tenant_id: Uuid, catalog_name: &str, name: String) -> Result<()>`

### Merge & Conflict Operations
- `find_conflicting_assets(tenant_id, catalog_name, source_branch, target_branch) -> Result<Vec<(Asset, Asset)>>`
- `create_merge_operation(operation: MergeOperation) -> Result<()>`
- `get_merge_operation(id: Uuid) -> Result<Option<MergeOperation>>`
- `list_merge_operations(tenant_id: Uuid, catalog_name: &str) -> Result<Vec<MergeOperation>>`
- `update_merge_operation_status(id: Uuid, status: MergeStatus) -> Result<()>`
- `complete_merge_operation(id: Uuid, result_commit_id: Uuid) -> Result<()>`
- `abort_merge_operation(id: Uuid) -> Result<()>`
- `create_merge_conflict(conflict: MergeConflict) -> Result<()>`
- `get_merge_conflict(id: Uuid) -> Result<Option<MergeConflict>>`
- `list_merge_conflicts(operation_id: Uuid) -> Result<Vec<MergeConflict>>`
- `resolve_merge_conflict(id: Uuid, resolution: ConflictResolution) -> Result<()>`

### Security & Identity
- **Users**: `create_user`, `get_user`, `get_user_by_username`, `list_users`, `update_user`, `delete_user`.
- **Roles**: `create_role`, `get_role`, `list_roles`, `assign_role`, `revoke_role`, `get_user_roles`, `update_role`, `delete_role`.
- **Service Users**: `create_service_user`, `get_service_user`, `get_service_user_by_api_key_hash`, `list_service_users`, `update_service_user`, `delete_service_user`, `update_service_user_last_used`.
- **Tokens**: `revoke_token`, `is_token_revoked`, `cleanup_expired_tokens`, `list_active_tokens`, `store_token`, `validate_token`.

### Business Metadata & Discovery
- `upsert_business_metadata(metadata: BusinessMetadata) -> Result<()>`
- `get_business_metadata(asset_id: Uuid) -> Result<Option<BusinessMetadata>>`
- `delete_business_metadata(asset_id: Uuid) -> Result<()>`
- `search_assets(tenant_id, query, tags) -> Result<Vec<(Asset, Option<BusinessMetadata>, String, Vec<String>)>>`
- `search_catalogs(tenant_id, query) -> Result<Vec<Catalog>>`
- `search_namespaces(tenant_id, query) -> Result<Vec<(Namespace, String)>>`
- `search_branches(tenant_id, query) -> Result<Vec<(Branch, String)>>`
- `create_access_request(request: AccessRequest) -> Result<()>`
- `get_access_request(id: Uuid) -> Result<Option<AccessRequest>>`
- `list_access_requests(tenant_id: Uuid) -> Result<Vec<AccessRequest>>`
- `update_access_request(request: AccessRequest) -> Result<()>`

### System & Maintenance
- `get_system_settings(tenant_id: Uuid) -> Result<SystemSettings>`
- `update_system_settings(tenant_id, settings) -> Result<SystemSettings>`
- `sync_federated_catalog(tenant_id, catalog_name) -> Result<()>`
- `get_federated_catalog_stats(tenant_id, catalog_name) -> Result<SyncStats>`
- `expire_snapshots(...)`, `remove_orphan_files(...)`.

---

## Signer Trait

**Source**: `pangolin_store/src/signer.rs`

The `Signer` trait defines the interface for vending credentials and presigning URLs for data access.

### Methods
- `get_table_credentials(&self, location: &str) -> Result<Credentials>`: Returns temporary credentials (AWS STS, Azure SAS, GCP Token) for a specific table location.
- `presign_get(&self, location: &str) -> Result<String>`: Generates a presigned HTTP GET URL for a specific object location.
