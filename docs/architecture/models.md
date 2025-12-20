# Pangolin Data Models

## Overview
This document serves as the technical reference for all core data models used within Pangolin. These models are defined in the `pangolin_core` crate and are consistently implemented across all storage backends.

---

## Core Infrastructure (`model.rs`)

### Tenant
The root unit of multi-tenancy.
```rust
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub properties: HashMap<String, String>,
}
```

### Warehouse
Configurations for underlying object storage.
```rust
pub struct Warehouse {
    pub id: Uuid,
    pub name: String,
    pub tenant_id: Uuid,
    pub storage_config: HashMap<String, String>,
    pub use_sts: bool, // Deprecated in favor of vending_strategy
    pub vending_strategy: Option<VendingStrategy>,
}
```

### VendingStrategy
Defines how Pangolin provides temporary data access credentials to clients.
- `AwsSts`: AWS Role Assumption.
- `AwsStatic`: Direct access keys.
- `AzureSas`: Shared Access Signatures.
- `GcpDownscoped`: Service Account impersonation.
- `None`: No credentials provided (server-side only or client-managed).

### Catalog
Standard Iceberg or Federated proxy catalog.
```rust
pub struct Catalog {
    pub id: Uuid,
    pub name: String,
    pub catalog_type: CatalogType, // Local | Federated
    pub warehouse_name: Option<String>,
    pub storage_location: Option<String>,
    pub federated_config: Option<FederatedCatalogConfig>,
    pub properties: HashMap<String, String>,
}
```

---

## Data Lifecycle & Assets (`model.rs`)

### Asset
A unified representation of data resources.
```rust
pub struct Asset {
    pub id: Uuid,
    pub name: String,
    pub kind: AssetType, // IcebergTable, View, MlModel, etc.
    pub location: String,
    pub properties: HashMap<String, String>,
}
```

### Branch & Tag
References to commit chains for versioning.
```rust
pub struct Branch {
    pub name: String,
    pub head_commit_id: Option<Uuid>,
    pub branch_type: BranchType, // Ingest | Experimental
    pub assets: Vec<String>, // Assets strictly tracked by this branch
}

pub struct Tag {
    pub name: String,
    pub commit_id: Uuid,
}
```

### Commit
Immutable snapshots of state.
```rust
pub struct Commit {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub timestamp: i64,
    pub author: String,
    pub operations: Vec<CommitOperation>, // Put { asset } | Delete { name }
}
```

---

## Identity & Access (`user.rs`, `permission.rs`)

### User
```rust
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub tenant_id: Option<Uuid>, // None for Root
    pub role: UserRole, // Root | TenantAdmin | TenantUser
    pub active: bool,
    pub oauth_provider: Option<OAuthProvider>,
}
```

### ServiceUser
Programmatic identities for automation.
```rust
pub struct ServiceUser {
    pub id: Uuid,
    pub name: String,
    pub tenant_id: Uuid,
    pub api_key_hash: String,
    pub role: UserRole,
    pub expires_at: Option<DateTime<Utc>>,
    pub active: bool,
}
```

### Permission Scope
Defines the target range for a grant.
```rust
pub enum PermissionScope {
    Tenant,
    Catalog { catalog_id: Uuid },
    Namespace { catalog_id: Uuid, namespace: String },
    Asset { catalog_id: Uuid, namespace: String, asset_id: Uuid },
    Tag { tag_name: String }, // TBAC
}
```

### Action
Atomic operations allowed on a scope.
- `Read`, `Write`, `Delete`, `Create`, `Update`, `List`, `All`.
- `IngestBranching`: Can merge changes back to parent.
- `ExperimentalBranching`: Isolated dev-only branches.
- `ManageDiscovery`: Permission to edit business metadata.

---

## Business & Discovery (`business_metadata.rs`, `audit.rs`)

### BusinessMetadata
Descriptive layers for data assets.
```rust
pub struct BusinessMetadata {
    pub asset_id: Uuid,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub properties: serde_json::Value,
    pub discoverable: bool,
}
```

### AuditLogEntry
Comprehensive tracking of every system action.
```rust
pub struct AuditLogEntry {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub username: String,
    pub action: AuditAction,
    pub resource_type: ResourceType,
    pub resource_name: String,
    pub result: AuditResult, // Success | Failure
    pub timestamp: DateTime<Utc>,
}
```

---

## System State (`model.rs`)

### SystemSettings
Tenant-specific system configurations.
- `allow_public_signup`: Boolean toggle.
- `default_warehouse_bucket`: Fallback storage.
- `default_retention_days`: Maintenance policy.

### SyncStats
Real-time metrics for Federated catalogs.
- `last_synced_at`: Timestamp.
- `sync_status`: Status indicator.
- `tables_synced`: Count.
