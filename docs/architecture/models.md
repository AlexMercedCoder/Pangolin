# Pangolin Data Models

## Overview

This document catalogs all core data models used throughout the Pangolin project. Models are organized by domain and defined in the `pangolin_core` crate.

## Model Organization

Models are organized into the following modules:
- **`model.rs`**: Core Iceberg and infrastructure models
- **`user.rs`**: User authentication and authorization models
- **`permission.rs`**: Permission and role-based access control models
- **`business_metadata.rs`**: Business metadata and discovery models
- **`audit.rs`**: Audit logging models

---

## Core Models (`pangolin_core/src/model.rs`)

### Tenant

Multi-tenancy isolation unit.

```rust
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub properties: HashMap<String, String>,
}
```

**Fields**:
- `id`: Unique tenant identifier
- `name`: Tenant name (unique)
- `properties`: Custom key-value properties

**Usage**: Root-level organization unit for complete data isolation

### Warehouse

Storage configuration for a tenant.

```rust
pub struct Warehouse {
    pub id: Uuid,
    pub name: String,
    pub tenant_id: Uuid,
    pub storage_config: HashMap<String, String>,
    pub use_sts: bool,
    pub vending_strategy: Option<VendingStrategy>,
}
```

**Fields**:
- `id`: Unique warehouse identifier
- `name`: Warehouse name
- `tenant_id`: Parent tenant
- `storage_config`: S3/Azure/GCS configuration
- `use_sts`: Legacy STS flag (deprecated)
- `vending_strategy`: Credential vending strategy

**Usage**: Defines where and how data is stored

### VendingStrategy

Credential vending configuration.

```rust
pub enum VendingStrategy {
    AwsSts {
        role_arn: String,
        external_id: Option<String>,
    },
    AwsStatic {
        access_key_id: String,
        secret_access_key: String,
    },
    AzureSas {
        account_name: String,
        account_key: String,
    },
    GcpDownscoped {
        service_account_email: String,
        private_key: String,
    },
    None,
}
```

**Variants**:
- `AwsSts`: AWS STS role assumption
- `AwsStatic`: Static AWS credentials
- `AzureSas`: Azure SAS tokens
- `GcpDownscoped`: GCP service account impersonation
- `None`: No credential vending (client-provided only)

### Catalog

Iceberg catalog configuration.

```rust
pub struct Catalog {
    pub id: Uuid,
    pub name: String,
    pub warehouse_id: Uuid,
    pub tenant_id: Uuid,
    pub properties: HashMap<String, String>,
    pub catalog_type: CatalogType,
    pub federated_config: Option<FederatedCatalogConfig>,
}
```

**Fields**:
- `id`: Unique catalog identifier
- `name`: Catalog name
- `warehouse_id`: Parent warehouse
- `tenant_id`: Parent tenant
- `properties`: Custom properties
- `catalog_type`: Local or Federated
- `federated_config`: Configuration for federated catalogs

### CatalogType

```rust
pub enum CatalogType {
    Local,
    Federated,
}
```

### FederatedCatalogConfig

Configuration for federated (remote) catalogs.

```rust
pub struct FederatedCatalogConfig {
    pub base_url: String,
    pub auth_type: FederatedAuthType,
    pub credentials: Option<FederatedCredentials>,
    pub timeout_seconds: u64,
}
```

### Namespace

Iceberg namespace (database/schema).

```rust
pub struct Namespace {
    pub id: Uuid,
    pub name: Vec<String>,
    pub properties: HashMap<String, String>,
    pub catalog_id: Uuid,
}
```

**Fields**:
- `id`: Unique namespace identifier
- `name`: Multi-level namespace (e.g., ["db", "schema"])
- `properties`: Custom properties
- `catalog_id`: Parent catalog

### Asset

Iceberg table or view.

```rust
pub struct Asset {
    pub id: Uuid,
    pub name: String,
    pub kind: String, // "table" or "view"
    pub namespace_id: Uuid,
    pub location: String,
    pub metadata_location: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Fields**:
- `id`: Unique asset identifier
- `name`: Table/view name
- `kind`: "table" or "view"
- `namespace_id`: Parent namespace
- `location`: S3/Azure/GCS base location
- `metadata_location`: Current metadata file location
- `created_at`: Creation timestamp
- `updated_at`: Last update timestamp

### Branch

Git-like branch for table versioning.

```rust
pub struct Branch {
    pub id: Uuid,
    pub name: String,
    pub catalog_id: Uuid,
    pub branch_type: BranchType,
    pub head_commit_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}
```

**Fields**:
- `id`: Unique branch identifier
- `name`: Branch name
- `catalog_id`: Parent catalog
- `branch_type`: Main, Ingest, or Experimental
- `head_commit_id`: Latest commit on this branch
- `created_at`: Creation timestamp

### BranchType

```rust
pub enum BranchType {
    Main,
    Ingest,
    Experimental,
}
```

### Tag

Named reference to a specific commit.

```rust
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub catalog_id: Uuid,
    pub commit_id: Uuid,
    pub created_at: DateTime<Utc>,
}
```

### Commit

Snapshot of table state.

```rust
pub struct Commit {
    pub id: Uuid,
    pub catalog_id: Uuid,
    pub asset_id: Uuid,
    pub parent_commit_id: Option<Uuid>,
    pub metadata_location: String,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}
```

### Merge Models

#### MergeOperation

```rust
pub struct MergeOperation {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub catalog_name: String,
    pub source_branch: String,
    pub target_branch: String,
    pub base_commit_id: Option<Uuid>,
    pub status: MergeStatus,
    pub conflicts: Vec<Uuid>,
    pub initiated_by: Uuid,
    pub initiated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result_commit_id: Option<Uuid>,
}
```

#### MergeStatus

```rust
pub enum MergeStatus {
    Pending,
    Conflicted,
    Resolving,
    Ready,
    Completed,
    Aborted,
}
```

#### MergeConflict

```rust
pub struct MergeConflict {
    pub id: Uuid,
    pub merge_operation_id: Uuid,
    pub conflict_type: ConflictType,
    pub asset_id: Option<Uuid>,
    pub description: String,
    pub resolution: Option<ConflictResolution>,
    pub created_at: DateTime<Utc>,
}
```

#### ConflictType

```rust
pub enum ConflictType {
    SchemaChange {
        asset_name: String,
        source_schema: serde_json::Value,
        target_schema: serde_json::Value,
    },
    DataOverlap {
        asset_name: String,
        overlapping_partitions: Vec<String>,
    },
    MetadataConflict {
        asset_name: String,
        conflicting_properties: Vec<String>,
    },
    DeletionConflict {
        asset_name: String,
        deleted_in: String,
        modified_in: String,
    },
}
```

---

## User Models (`pangolin_core/src/user.rs`)

### User

```rust
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub tenant_id: Option<Uuid>,
    pub role: UserRole,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub oauth_provider: Option<OAuthProvider>,
    pub oauth_id: Option<String>,
}
```

### UserRole

```rust
pub enum UserRole {
    Root,
    TenantAdmin,
    User,
}
```

### UserSession

```rust
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub tenant_id: Option<Uuid>,
    pub role: UserRole,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
```

### ServiceUser

```rust
pub struct ServiceUser {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub tenant_id: Uuid,
    pub role: UserRole,
    pub api_key_hash: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}
```

### OAuthProvider

```rust
pub enum OAuthProvider {
    Google,
    GitHub,
    Microsoft,
}
```

---

## Permission Models (`pangolin_core/src/permission.rs`)

### Permission

```rust
pub struct Permission {
    pub id: Uuid,
    pub user_id: Uuid,
    pub scope: PermissionScope,
    pub actions: Vec<Action>,
    pub granted_by: Uuid,
    pub granted_at: DateTime<Utc>,
}
```

### PermissionScope

```rust
pub enum PermissionScope {
    Tenant { tenant_id: Uuid },
    Warehouse { warehouse_id: Uuid },
    Catalog { catalog_id: Uuid },
    Namespace { namespace_id: Uuid },
    Asset { asset_id: Uuid },
}
```

### Action

```rust
pub enum Action {
    Read,
    Write,
    Delete,
    Admin,
    ManageDiscovery,
}
```

### Role

```rust
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub tenant_id: Uuid,
    pub permissions: Vec<PermissionGrant>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

---

## Business Metadata Models (`pangolin_core/src/business_metadata.rs`)

### BusinessMetadata

```rust
pub struct BusinessMetadata {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub properties: HashMap<String, String>,
    pub discoverable: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_by: Uuid,
    pub updated_at: DateTime<Utc>,
}
```

### AccessRequest

```rust
pub struct AccessRequest {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub asset_id: Uuid,
    pub reason: Option<String>,
    pub requested_at: DateTime<Utc>,
    pub status: RequestStatus,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_comment: Option<String>,
}
```

### RequestStatus

```rust
pub enum RequestStatus {
    Pending,
    Approved,
    Rejected,
}
```

---

## Audit Models (`pangolin_core/src/audit.rs`)

### AuditLogEntry

```rust
pub struct AuditLogEntry {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user: String,
    pub action: String,
    pub resource: String,
    pub details: Option<String>,
    pub timestamp: DateTime<Utc>,
}
```

---

## Update Models

### TenantUpdate

```rust
pub struct TenantUpdate {
    pub name: Option<String>,
    pub properties: Option<HashMap<String, String>>,
}
```

### WarehouseUpdate

```rust
pub struct WarehouseUpdate {
    pub storage_config: Option<HashMap<String, String>>,
    pub vending_strategy: Option<VendingStrategy>,
}
```

### CatalogUpdate

```rust
pub struct CatalogUpdate {
    pub properties: Option<HashMap<String, String>>,
}
```

---

## Common Patterns

### UUID Identifiers
All models use `Uuid` for unique identification, ensuring global uniqueness and preventing ID collisions.

### Timestamps
Models use `chrono::DateTime<Utc>` for all timestamps, ensuring consistent timezone handling.

### Optional Fields
Many fields are `Option<T>` to support partial updates and nullable values.

### HashMap Properties
Custom properties use `HashMap<String, String>` for flexibility.

## Serialization

All models derive `Serialize` and `Deserialize` from `serde`, enabling:
- JSON API responses
- Database storage
- Configuration files
- OpenAPI schema generation (with `ToSchema`)

## See Also

- [CatalogStore Trait](./catalog-store-trait.md)
- [Signer Trait](./signer-trait.md)
- [API Documentation](../api/README.md)
