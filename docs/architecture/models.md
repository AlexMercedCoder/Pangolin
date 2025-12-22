# Pangolin Core Models

This document details the core data structures (Structs) defined in `pangolin_core`.

## Identity & Storage

### Tenant
Top-level isolation boundary for multitenancy.
- **id**: `Uuid` - Unique identifier.
- **name**: `String` - Tenant name (e.g., "acme-corp").
- **properties**: `HashMap<String, String>` - Custom metadata.

### Warehouse
Defines the connection to physical object storage (S3, GCS, Azure).
- **id**: `Uuid`
- **name**: `String` - Unique name within a tenant.
- **tenant_id**: `Uuid` - Owner tenant.
- **storage_config**: `HashMap<String, String>` - Endpoint, region, bucket.
- **vending_strategy**: `Option<VendingStrategy>` - Logic for vending temporary credentials.

### Catalog
A logical grouping of assets, maps to an Iceberg Catalog.
- **id**: `Uuid`
- **name**: `String`
- **catalog_type**: `CatalogType` (Local or Federated)
- **warehouse_name**: `Option<String>` - Link to the storage backend.
- **storage_location**: `Option<String>` - Base path prefix.
- **federated_config**: `Option<FederatedCatalogConfig>` - Upstream catalog details.

---

## Asset Management

### Namespace
Hierarchical container for assets (equivalent to Schema/Database).
- **name**: `Vec<String>` - Parts of the path (e.g., `["accounting", "finance"]`).
- **properties**: `HashMap<String, String>`.

### Asset
Represents a managed data artifact (Table, View, Model).
- **id**: `Uuid`
- **name**: `String`
- **kind**: `AssetType` (e.g., IcebergTable, View).
- **location**: `String` - Absolute object store path.
- **properties**: `HashMap<String, String>`.

---

## Version Control

### Branch
A pointer to a specific commit history, enabling Git-like data versioning.
- **name**: `String` (e.g., "main", "dev").
- **head_commit_id**: `Option<Uuid>` - Latest commit on this branch.
- **branch_type**: `BranchType`.
- **assets**: `Vec<String>` - List of asset names currently tracked on this branch.

### Tag
A named reference to a specific commit (immutable).
- **name**: `String` (e.g., "v1.0").
- **commit_id**: `Uuid`.

### Commit
An immutable record of a state change.
- **id**: `Uuid`
- **parent_id**: `Option<Uuid>` - Previous commit.
- **timestamp**: `i64` - Epoch time.
- **author**: `String` - User who made the change.
- **operations**: `Vec<CommitOperation>` - List of atomic changes (Put/Delete).

---

## Merge Operations

### MergeOperation
Tracks the state of merging one branch into another.
- **id**: `Uuid`
- **source_branch**: `String`.
- **target_branch**: `String`.
- **base_commit_id**: `Option<Uuid>` - Common ancestor.
- **status**: `MergeStatus`.
- **conflicts**: `Vec<Uuid>` - List of `MergeConflict` IDs to resolve.
- **result_commit_id**: `Option<Uuid>` - The final commit ID if successful.

### MergeConflict
Represents a specific issue preventing an automatic merge.
- **id**: `Uuid`
- **conflict_type**: `ConflictType`.
- **description**: `String` - Human-readable detail.
- **resolution**: `Option<ConflictResolution>` - Records how it was fixed.

### ConflictResolution
Details on how a conflict was resolved.
- **strategy**: `ResolutionStrategy` (e.g., TakeSource).
- **resolved_by**: `Uuid` - User ID.
- **resolved_value**: `Option<serde_json::Value>` - The final data/schema used.

---

## System Entities

### Credentials
Temporary access tokens returned to clients.
- **Aws**: AccessKey, SecretKey, SessionToken.
- **Azure**: SAS Token, Account Name.
- **Gcp**: Access Token.

### AuditLogEntry
Record of system activity.
- **id**: `Uuid`.
- **actor_id**: `Uuid` - User ID.
- **action**: `String` - (e.g., "create_table").
- **resource**: `String` - (e.g., "namespace/table").
- **status**: `String` ("Success", "Failed").
