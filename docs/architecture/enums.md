# Pangolin Enums & Models

This document details the core data structures and enumerations defined in `pangolin_core`.

## Enums

### AssetType
Defines the type of data asset being managed.
- `IcebergTable`: Apache Iceberg table format.
- `DeltaTable`: Delta Lake table format.
- `HudiTable`: Apache Hudi table format.
- `ParquetTable`: Raw Parquet file table.
- `CsvTable`: CSV file table.
- `JsonTable`: JSON file table.
- `View`: Virtual view definition.
- `MlModel`: Machine Learning model artifact.

### CatalogType
Defines the nature of a catalog.
- `Local`: Managed natively by Pangolin.
- `Federated`: Proxy to an external Iceberg REST catalog.

### BranchType
Defines the purpose of a branch.
- `Ingest`: For data ingestion pipelines.
- `Experimental`: For testing and development.
- (Default): Standard branch.

### VendingStrategy
Defines how credentials are vended for a warehouse.
- `AwsSts`: AWS Security Token Service (AssumeRole).
- `AwsStatic`: Static IAM Access Keys.
- `AzureSas`: Azure Shared Access Signatures.
- `GcpDownscoped`: GCP Downscoped Token.
- `None`: No credential vending (proxy only).

### ConflictType
Types of conflicts that can occur during a merge.
- `SchemaChange`: Incompatible schema evolution.
- `DataOverlap`: Concurrent writes to the same partitions.
- `MetadataConflict`: Conflicting table properties.
- `DeletionConflict`: Asset deleted in one branch but modified in another.

### ResolutionStrategy
Strategies for resolving merge conflicts.
- `AutoMerge`: Automatically apply non-conflicting changes.
- `TakeSource`: Overwrite with source branch version.
- `TakeTarget`: Keep target branch version.
- `Manual`: Require human intervention.
- `ThreeWayMerge`: Use base commit to reconcile changes.

### MergeStatus
Status lifecycle of a merge operation.
- `Pending` -> `Conflicted` -> `Resolving` -> `Ready` -> `Completed`
- `Aborted` (Terminal state)

---

## Core Models

### Tenant
Top-level isolation boundary.
- `id`: UUID
- `name`: String
- `properties`: Map<String, String>

### Warehouse
Physical storage configuration.
- `id`: UUID
- `tenant_id`: UUID
- `storage_config`: Map containing bucket/endpoint details.
- `vending_strategy`: Configuration for credential vending.

### Catalog
Logical grouping of namespaces and tables.
- `id`: UUID
- `name`: String
- `catalog_type`: `CatalogType`
- `warehouse_name`: Link to physical storage.
- `federated_config`: Config for external proxy (if Federated).

### Namespace
Hierarchical organization unit (e.g., `db.schema`).
- `name`: List<String> parts.
- `properties`: Map<String, String>.

### Asset
The primary unit of management (Table, View).
- `id`: UUID
- `name`: String
- `kind`: `AssetType`
- `location`: s3:// path.
- `properties`: Key-value pairs.

### Branch
Nessie-like branching structure.
- `name`: String
- `head_commit_id`: Pointer to the latest commit.
- `branch_type`: `BranchType`
- `assets`: List of assets visible in this branch.

### Commit
Immutable record of changes.
- `id`: UUID
- `parent_id`: Previous commit UUID.
- `operations`: List of `Put` or `Delete` actions.

### MergeOperation
State tracking for a branch merge.
- `source_branch`: Branch being merged.
- `target_branch`: Destination branch.
- `conflicts`: List of `MergeConflict` IDs.
- `status`: Current `MergeStatus`.

### MergeConflict
Detail record for a specific conflict.
- `conflict_type`: `ConflictType`.
- `resolution`: `ConflictResolution` (if resolved).
- `description`: Human-readable explanation.
