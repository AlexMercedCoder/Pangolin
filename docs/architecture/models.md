# Pangolin Core Models

This document details the core data structures (Structs) defined in `pangolin_core`.

## Identity & Storage

### Tenant
The root multi-tenancy isolation boundary.
- `id`: `Uuid`
- `name`: `String`
- `properties`: `HashMap<String, String>`

### Warehouse
Physical storage backend configuration (S3, GCS, ADLS).
- `id`: `Uuid`
- `name`: `String` (Unique per tenant)
- `tenant_id`: `Uuid`
- `storage_config`: `HashMap<String, String>`
- `vending_strategy`: `Option<VendingStrategy>`
- `use_sts`: `bool` (Legacy flag)

### Catalog
Logical grouping for assets, maps to an Iceberg Catalog.
- `id`: `Uuid` (For granular permissions)
- `name`: `String`
- `catalog_type`: `CatalogType` (Local or Federated)
- `warehouse_name`: `Option<String>`
- `storage_location`: `Option<String>`
- `federated_config`: `Option<FederatedCatalogConfig>`
- `properties`: `HashMap<String, String>`

---

## Asset Management

### Namespace
Hierarchical organization unit (`db.schema`).
- `name`: `Vec<String>` (Path parts)
- `properties`: `HashMap<String, String>`

### Asset
Managed data artifact.
- `id`: `Uuid`
- `name`: `String`
- `kind`: `AssetType`
- `location`: `String` (Object store URI)
- `properties`: `HashMap<String, String>`

---

## Version Control

### Branch
Pointer to a commit history chain.
- `name`: `String`
- `head_commit_id`: `Option<Uuid>`
- `branch_type`: `BranchType`
- `assets`: `Vec<String>` (Visible asset names)

### Tag
Named immutable reference to a commit.
- `name`: `String`
- `commit_id`: `Uuid`

### Commit
Immutable record of atomic changes.
- `id`: `Uuid`
- `parent_id`: `Option<Uuid>`
- `timestamp`: `i64` (Epoch)
- `author`: `String`
- `message`: `String`
- `operations`: `Vec<CommitOperation>` (Put or Delete)

---

## Security & Identity

### User
Human user profile.
- `id`: `Uuid`
- `username`: `String`
- `email`: `String`
- `password_hash`: `Option<String>` (BCrypt)
- `oauth_provider`: `Option<OAuthProvider>`
- `tenant_id`: `Option<Uuid>`
- `role`: `UserRole`
- `active`: `bool`

### ServiceUser
Machine account for API key access.
- `id`: `Uuid`
- `name`: `String`
- `tenant_id`: `Uuid`
- `api_key_hash`: `String`
- `role`: `UserRole`
- `last_used`: `Option<DateTime<Utc>>`
- `active`: `bool`

### Role
A named group of permissions within a tenant.
- `id`: `Uuid`
- `name`: `String`
- `tenant_id`: `Uuid`
- `permissions`: `Vec<PermissionGrant>`

### Permission
Directly assigned permission grant.
- `id`: `Uuid`
- `user_id`: `Uuid`
- `scope`: `PermissionScope`
- `actions`: `HashSet<Action>`

### PermissionGrant
Permission definition stored within a Role.
- `scope`: `PermissionScope`
- `actions`: `HashSet<Action>`

---

## Merge & Discovery

### MergeOperation
State tracking for branch reconciliation.
- `id`: `Uuid`
- `source_branch`: `String`
- `target_branch`: `String`
- `base_commit_id`: `Option<Uuid>`
- `status`: `MergeStatus`
- `conflicts`: `Vec<Uuid>`

### BusinessMetadata
Governance metadata for an asset.
- `asset_id`: `Uuid`
- `owner`: `String`
- `tags`: `Vec<String>`
- `description`: `Option<String>`

### AccessRequest
Workflow record for requesting asset access.
- `id`: `Uuid`
- `asset_id`: `Uuid`
- `requester_id`: `Uuid`
- `status`: `String` (Pending/Approved/Rejected)

---

## System

### SystemSettings
Global configuration for a tenant.
- `allow_public_signup`: `Option<bool>`
- `default_warehouse_bucket`: `Option<String>`
- `default_retention_days`: `Option<i32>`
- `smtp_host`: `Option<String>`, etc.

### SyncStats
Performance metrics for federated catalogs.
- `last_synced_at`: `Option<DateTime<Utc>>`
- `sync_status`: `String`
- `tables_synced`: `i32`
