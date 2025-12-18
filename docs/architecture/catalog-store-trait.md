# CatalogStore Trait

## Overview

The `CatalogStore` trait is the core abstraction for all storage backends in Pangolin. It defines a comprehensive interface for managing tenants, warehouses, catalogs, users, permissions, and all Iceberg-related entities.

**Location**: `pangolin_store/src/lib.rs`

## Purpose

The `CatalogStore` trait provides:
- **Storage Backend Abstraction**: Allows multiple storage implementations (Memory, PostgreSQL, MongoDB, SQLite)
- **Async Operations**: All methods are async for non-blocking I/O
- **Error Handling**: Consistent error handling across all storage operations
- **Multi-tenancy**: Built-in tenant isolation for all operations

## Implementations

- **MemoryStore** (`pangolin_store/src/memory.rs`) - In-memory storage for testing and development
- **PostgresStore** (`pangolin_store/src/postgres.rs`) - PostgreSQL backend for production
- **MongoStore** (`pangolin_store/src/mongo.rs`) - MongoDB backend (alternative)
- **SqliteStore** (`pangolin_store/src/sqlite.rs`) - SQLite backend for embedded deployments

## Trait Definition

```rust
#[async_trait]
pub trait CatalogStore: Send + Sync {
    // Tenant Management
    // Warehouse Management
    // Catalog Management
    // Namespace Management
    // Table Management
    // User Management
    // Permission Management
    // Token Management
    // Branch/Tag Management
    // Merge Operations
    // Business Metadata
    // Federated Catalogs
    // Service Users
    // Audit Logging
}
```

## Method Categories

### 1. Tenant Management

#### `create_tenant`
```rust
async fn create_tenant(&self, tenant: Tenant) -> Result<(), StoreError>;
```
Creates a new tenant in the system.

**Parameters**:
- `tenant`: Tenant struct with id, name, and properties

**Returns**: `Result<(), StoreError>`

**Usage**:
```rust
let tenant = Tenant {
    id: Uuid::new_v4(),
    name: "acme-corp".to_string(),
    properties: HashMap::new(),
};
store.create_tenant(tenant).await?;
```

#### `list_tenants`
```rust
async fn list_tenants(&self) -> Result<Vec<Tenant>, StoreError>;
```
Lists all tenants in the system.

#### `get_tenant`
```rust
async fn get_tenant(&self, tenant_id: Uuid) -> Result<Option<Tenant>, StoreError>;
```
Retrieves a specific tenant by ID.

#### `update_tenant`
```rust
async fn update_tenant(&self, tenant_id: Uuid, update: TenantUpdate) -> Result<(), StoreError>;
```
Updates tenant properties.

#### `delete_tenant`
```rust
async fn delete_tenant(&self, tenant_id: Uuid) -> Result<(), StoreError>;
```
Deletes a tenant and all associated data.

### 2. Warehouse Management

#### `create_warehouse`
```rust
async fn create_warehouse(&self, warehouse: Warehouse) -> Result<(), StoreError>;
```
Creates a new warehouse for a tenant.

#### `list_warehouses`
```rust
async fn list_warehouses(&self, tenant_id: Uuid) -> Result<Vec<Warehouse>, StoreError>;
```
Lists all warehouses for a tenant.

#### `get_warehouse`
```rust
async fn get_warehouse(&self, tenant_id: Uuid, warehouse_id: Uuid) -> Result<Option<Warehouse>, StoreError>;
```
Retrieves a specific warehouse.

#### `update_warehouse`
```rust
async fn update_warehouse(&self, tenant_id: Uuid, warehouse_id: Uuid, update: WarehouseUpdate) -> Result<(), StoreError>;
```
Updates warehouse configuration.

#### `delete_warehouse`
```rust
async fn delete_warehouse(&self, tenant_id: Uuid, warehouse_id: Uuid) -> Result<(), StoreError>;
```
Deletes a warehouse.

### 3. Catalog Management

#### `create_catalog`
```rust
async fn create_catalog(&self, catalog: Catalog) -> Result<(), StoreError>;
```
Creates a new catalog within a warehouse.

#### `list_catalogs`
```rust
async fn list_catalogs(&self, tenant_id: Uuid) -> Result<Vec<Catalog>, StoreError>;
```
Lists all catalogs for a tenant.

#### `get_catalog`
```rust
async fn get_catalog(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Option<Catalog>, StoreError>;
```
Retrieves a catalog by name.

#### `update_catalog`
```rust
async fn update_catalog(&self, tenant_id: Uuid, catalog_name: &str, update: CatalogUpdate) -> Result<(), StoreError>;
```
Updates catalog configuration.

#### `delete_catalog`
```rust
async fn delete_catalog(&self, tenant_id: Uuid, catalog_name: &str) -> Result<(), StoreError>;
```
Deletes a catalog.

### 4. Namespace Management

#### `create_namespace`
```rust
async fn create_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Namespace) -> Result<(), StoreError>;
```
Creates a new namespace in a catalog.

#### `list_namespaces`
```rust
async fn list_namespaces(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<Namespace>, StoreError>;
```
Lists all namespaces in a catalog.

#### `get_namespace`
```rust
async fn get_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: &str) -> Result<Option<Namespace>, StoreError>;
```
Retrieves a specific namespace.

#### `update_namespace_properties`
```rust
async fn update_namespace_properties(&self, tenant_id: Uuid, catalog_name: &str, namespace: &str, updates: HashMap<String, String>, removals: Vec<String>) -> Result<(), StoreError>;
```
Updates namespace properties.

#### `delete_namespace`
```rust
async fn delete_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: &str) -> Result<(), StoreError>;
```
Deletes a namespace.

### 5. Table Management

#### `create_table`
```rust
async fn create_table(&self, tenant_id: Uuid, catalog_name: &str, namespace: &str, table_name: &str, metadata: serde_json::Value) -> Result<Asset, StoreError>;
```
Creates a new Iceberg table.

#### `load_table`
```rust
async fn load_table(&self, tenant_id: Uuid, catalog_name: &str, namespace: &str, table_name: &str) -> Result<Option<Asset>, StoreError>;
```
Loads table metadata.

#### `update_table`
```rust
async fn update_table(&self, tenant_id: Uuid, catalog_name: &str, namespace: &str, table_name: &str, metadata: serde_json::Value) -> Result<Asset, StoreError>;
```
Updates table metadata.

#### `delete_table`
```rust
async fn delete_table(&self, tenant_id: Uuid, catalog_name: &str, namespace: &str, table_name: &str) -> Result<(), StoreError>;
```
Deletes a table.

#### `table_exists`
```rust
async fn table_exists(&self, tenant_id: Uuid, catalog_name: &str, namespace: &str, table_name: &str) -> Result<bool, StoreError>;
```
Checks if a table exists.

### 6. User Management

#### `create_user`
```rust
async fn create_user(&self, user: User) -> Result<(), StoreError>;
```
Creates a new user.

#### `get_user_by_username`
```rust
async fn get_user_by_username(&self, tenant_id: Uuid, username: &str) -> Result<Option<User>, StoreError>;
```
Retrieves a user by username.

#### `get_user_by_id`
```rust
async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>, StoreError>;
```
Retrieves a user by ID.

#### `list_users`
```rust
async fn list_users(&self, tenant_id: Uuid) -> Result<Vec<User>, StoreError>;
```
Lists all users for a tenant.

#### `update_user`
```rust
async fn update_user(&self, user_id: Uuid, username: Option<String>, password_hash: Option<String>, active: Option<bool>) -> Result<(), StoreError>;
```
Updates user information.

#### `delete_user`
```rust
async fn delete_user(&self, user_id: Uuid) -> Result<(), StoreError>;
```
Deletes a user.

### 7. Permission Management

#### `grant_permission`
```rust
async fn grant_permission(&self, permission: Permission) -> Result<(), StoreError>;
```
Grants a permission to a user.

#### `revoke_permission`
```rust
async fn revoke_permission(&self, permission_id: Uuid) -> Result<(), StoreError>;
```
Revokes a permission.

#### `list_user_permissions`
```rust
async fn list_user_permissions(&self, user_id: Uuid) -> Result<Vec<Permission>, StoreError>;
```
Lists all permissions for a user.

#### `create_role`
```rust
async fn create_role(&self, role: Role) -> Result<(), StoreError>;
```
Creates a new role.

#### `list_roles`
```rust
async fn list_roles(&self, tenant_id: Uuid) -> Result<Vec<Role>, StoreError>;
```
Lists all roles for a tenant.

### 8. Token Management

#### `create_token`
```rust
async fn create_token(&self, token: UserSession) -> Result<(), StoreError>;
```
Creates a new authentication token.

#### `get_token`
```rust
async fn get_token(&self, token: &str) -> Result<Option<UserSession>, StoreError>;
```
Retrieves a token by its value.

#### `revoke_token`
```rust
async fn revoke_token(&self, token: &str) -> Result<(), StoreError>;
```
Revokes a token.

#### `cleanup_expired_tokens`
```rust
async fn cleanup_expired_tokens(&self) -> Result<usize, StoreError>;
```
Removes expired tokens from storage.

### 9. Branch & Tag Management

#### `create_branch`
```rust
async fn create_branch(&self, tenant_id: Uuid, catalog_name: &str, branch: Branch) -> Result<(), StoreError>;
```
Creates a new branch.

#### `list_branches`
```rust
async fn list_branches(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<Branch>, StoreError>;
```
Lists all branches in a catalog.

#### `get_branch`
```rust
async fn get_branch(&self, tenant_id: Uuid, catalog_name: &str, branch_name: &str) -> Result<Option<Branch>, StoreError>;
```
Retrieves a specific branch.

#### `create_tag`
```rust
async fn create_tag(&self, tenant_id: Uuid, catalog_name: &str, tag: Tag) -> Result<(), StoreError>;
```
Creates a new tag.

#### `list_tags`
```rust
async fn list_tags(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<Tag>, StoreError>;
```
Lists all tags in a catalog.

### 10. Merge Operations

#### `create_merge_operation`
```rust
async fn create_merge_operation(&self, operation: MergeOperation) -> Result<(), StoreError>;
```
Initiates a merge operation.

#### `get_merge_operation`
```rust
async fn get_merge_operation(&self, operation_id: Uuid) -> Result<Option<MergeOperation>, StoreError>;
```
Retrieves a merge operation.

#### `list_merge_conflicts`
```rust
async fn list_merge_conflicts(&self, operation_id: Uuid) -> Result<Vec<MergeConflict>, StoreError>;
```
Lists conflicts for a merge operation.

#### `resolve_merge_conflict`
```rust
async fn resolve_merge_conflict(&self, conflict_id: Uuid, resolution: ConflictResolution) -> Result<(), StoreError>;
```
Resolves a merge conflict.

### 11. Business Metadata

#### `upsert_business_metadata`
```rust
async fn upsert_business_metadata(&self, metadata: BusinessMetadata) -> Result<(), StoreError>;
```
Creates or updates business metadata for an asset.

#### `get_business_metadata`
```rust
async fn get_business_metadata(&self, asset_id: Uuid) -> Result<Option<BusinessMetadata>, StoreError>;
```
Retrieves business metadata for an asset.

#### `search_assets`
```rust
async fn search_assets(&self, tenant_id: Uuid, query: &str, tags: Option<Vec<String>>) -> Result<Vec<(Asset, Option<BusinessMetadata>)>, StoreError>;
```
Searches for assets by query and tags.

### 12. Federated Catalogs

#### `create_federated_catalog`
```rust
async fn create_federated_catalog(&self, catalog: Catalog) -> Result<(), StoreError>;
```
Creates a federated catalog configuration.

#### `get_federated_catalog_config`
```rust
async fn get_federated_catalog_config(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Option<FederatedCatalogConfig>, StoreError>;
```
Retrieves federated catalog configuration.

### 13. Service Users

#### `create_service_user`
```rust
async fn create_service_user(&self, service_user: ServiceUser) -> Result<(), StoreError>;
```
Creates a service user for API access.

#### `list_service_users`
```rust
async fn list_service_users(&self, tenant_id: Uuid) -> Result<Vec<ServiceUser>, StoreError>;
```
Lists all service users for a tenant.

### 14. Audit Logging

#### `log_audit_event`
```rust
async fn log_audit_event(&self, tenant_id: Uuid, event: AuditLogEntry) -> Result<(), StoreError>;
```
Logs an audit event.

#### `list_audit_events`
```rust
async fn list_audit_events(&self, tenant_id: Uuid, limit: Option<usize>) -> Result<Vec<AuditLogEntry>, StoreError>;
```
Lists audit events for a tenant.

## Error Handling

All methods return `Result<T, StoreError>` where `StoreError` can be:

```rust
pub enum StoreError {
    NotFound,
    AlreadyExists,
    InvalidInput(String),
    DatabaseError(String),
    SerializationError(String),
    Other(String),
}
```

## Usage Example

```rust
use pangolin_store::{CatalogStore, MemoryStore};
use pangolin_core::model::Tenant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a store instance
    let store = MemoryStore::new();
    
    // Create a tenant
    let tenant = Tenant {
        id: Uuid::new_v4(),
        name: "my-tenant".to_string(),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant.clone()).await?;
    
    // List tenants
    let tenants = store.list_tenants().await?;
    println!("Tenants: {:?}", tenants);
    
    Ok(())
}
```

## Thread Safety

The trait requires `Send + Sync`, making all implementations thread-safe and suitable for use in async contexts with multiple concurrent requests.

## Best Practices

1. **Always handle errors**: Check `Result` types and handle errors appropriately
2. **Use tenant isolation**: Always pass the correct `tenant_id` to ensure data isolation
3. **Cleanup resources**: Use `delete_*` methods to remove unused resources
4. **Audit important operations**: Use `log_audit_event` for security-sensitive operations
5. **Check existence**: Use `*_exists` methods before operations when appropriate

## See Also

- [Models Documentation](./models.md)
- [Storage Implementations](./storage-implementations.md)
- [Error Handling](./error-handling.md)
