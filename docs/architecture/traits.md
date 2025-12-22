# Pangolin Traits Documentation

This document details the core traits used in the Pangolin architecture to define storage and signing interfaces.

## CatalogStore Trait

**Source**: `pangolin_store/src/lib.rs`

The `CatalogStore` trait is the central interface for all metadata storage backends (Memory, SQLite, PostgreSQL, MongoDB). It extends `Send`, `Sync`, and `Signer`.

### Tenant Operations
- `create_tenant(tenant: Tenant) -> Result<()>`: Creates a new tenant.
- `get_tenant(tenant_id: Uuid) -> Result<Option<Tenant>>`: Retrieves a tenant by ID.
- `list_tenants() -> Result<Vec<Tenant>>`: Lists all tenants.
- `update_tenant(tenant_id: Uuid, updates: TenantUpdate) -> Result<Tenant>`: Updates tenant properties.
- `delete_tenant(tenant_id: Uuid) -> Result<()>`: Deletes a tenant.

### Warehouse Operations
- `create_warehouse(tenant_id: Uuid, warehouse: Warehouse) -> Result<()>`: Creates a new warehouse.
- `get_warehouse(tenant_id: Uuid, name: String) -> Result<Option<Warehouse>>`: Retrieves a warehouse by name.
- `list_warehouses(tenant_id: Uuid) -> Result<Vec<Warehouse>>`: Lists warehouses for a tenant.
- `update_warehouse(tenant_id: Uuid, name: String, updates: WarehouseUpdate) -> Result<Warehouse>`: Updates a warehouse.
- `delete_warehouse(tenant_id: Uuid, name: String) -> Result<()>`: Deletes a warehouse.

### Catalog Operations
- `create_catalog(tenant_id: Uuid, catalog: Catalog) -> Result<()>`: Creates a new catalog.
- `get_catalog(tenant_id: Uuid, name: String) -> Result<Option<Catalog>>`: Retrieves a catalog by name.
- `update_catalog(tenant_id: Uuid, name: String, updates: CatalogUpdate) -> Result<Catalog>`: Updates a catalog.
- `delete_catalog(tenant_id: Uuid, name: String) -> Result<()>`: Deletes a catalog.
- `list_catalogs(tenant_id: Uuid) -> Result<Vec<Catalog>>`: Lists catalogs for a tenant.

### Namespace Operations
- `create_namespace(tenant_id: Uuid, catalog_name: &str, namespace: Namespace) -> Result<()>`: Creates a namespace.
- `list_namespaces(tenant_id: Uuid, catalog_name: &str, parent: Option<String>) -> Result<Vec<Namespace>>`: Lists namespaces.
- `get_namespace(tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<Option<Namespace>>`: Retrieves a namespace.
- `delete_namespace(tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<()>`: Deletes a namespace.
- `update_namespace_properties(...)`: Updates namespace properties.

### Asset Operations
- `create_asset(...)`: Creates a new asset (Table, View, etc.).
- `get_asset(...)`: Retrieves an asset by name/location.
- `get_asset_by_id(...)`: Retrieves an asset by UUID.
- `list_assets(...)`: Lists assets in a namespace.
- `delete_asset(...)`: Deletes an asset.
- `rename_asset(...)`: Renames an asset.
- `search_assets(...)`: Searches for assets across the store.

### Branch & Tag Operations
- `create_branch(...)`: Creates a new branch.
- `get_branch(...)`: Retrieves a branch.
- `list_branches(...)`: Lists branches in a catalog.
- `merge_branch(...)`: Merges a source branch into a target branch.
- `create_tag(...)`: Creates a tag for a commit.
- `list_tags(...)`: Lists tags.

### Merge & Conflict Operations
- `find_conflicting_assets(...)`: Identifies assets modified in both branches during a merge.
- `create_merge_operation(...)`: Starts a new merge operation.
- `list_merge_operations(...)`: Lists merge operations.
- `get_merge_conflict(...)`: Retrieves a specific conflict.
- `resolve_merge_conflict(...)`: Resolves a conflict with a chosen strategy.

### Metadata & File IO
- `get_metadata_location(...)`: Gets the current metadata file location for a table.
- `update_metadata_location(...)`: Updates the metadata pointer (commit).
- `read_file(location: &str) -> Result<Vec<u8>>`: Reads a file from the underlying object store.
- `write_file(location: &str, content: Vec<u8>) -> Result<()>`: Writes a file to the underlying object store.

### Security & Access
- `log_audit_event(...)`: Logs an audit entry.
- `list_audit_events(...)`: Lists audit logs.
- `revoke_token(...)`: Revokes an API token.
- `is_token_revoked(...)`: Checks token validity.

---

## Signer Trait

**Source**: `pangolin_store/src/signer.rs`

The `Signer` trait defines the interface for vending credential and presigning URLs for data access.

### Methods
- `get_table_credentials(&self, location: &str) -> Result<Credentials>`: Returns temporary credentials (AWS STS, Azure SAS, GCP Token) for a specific table location.
- `presign_get(&self, location: &str) -> Result<String>`: Generates a presigned HTTP GET URL for a specific object location.

### Types
- `Credentials`: Enum representing provider-specific credentials (`Aws`, `Azure`, `Gcp`).
