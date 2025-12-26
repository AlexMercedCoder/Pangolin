# Pangolin Enums

This document details the core enumerations defined in `pangolin_core`. These define the discrete states and types used across the system.

## Data & Storage

### AssetType
Defines the format and nature of a data asset.
- `SCREAMING_SNAKE_CASE` serialization.
- `IcebergTable`, `DeltaTable`, `HudiTable`, `ParquetTable`, `CsvTable`, `JsonTable`.
- `View`, `MlModel`, `ApachePaimon`, `Vortex`, `Lance`, `Nimble`.
- `Directory`, `VideoFile`, `ImageFile`, `DbConnString`, `Other`.

### CatalogType
Defines the nature of a catalog backend.
- `Local`: Native Pangolin catalog with full versioning.
- `Federated`: Proxy to an external Iceberg REST catalog.

### BranchType
Defines the purpose and behavior of a branch.
- `Ingest`: For transient data ingestion; can be merged.
- `Experimental`: For sandbox testing; limited merge capabilities.

### VendingStrategy
Configuration for how cloud credentials are vended.
- `AwsSts`: AWS STS AssumeRole with external ID support.
- `AwsStatic`: Static IAM credentials.
- `AzureSas`: Azure Blob SAS tokens.
- `GcpDownscoped`: GCP Downscoped access tokens.
- `None`: No vending; client provides own credentials.

---

## Identity & Access

### UserRole
Defines the high-level authorization tier for a user.
- `Root`: Global system administrator.
- `TenantAdmin`: Administrative control over a specific tenant.
- `TenantUser`: Standard user within a tenant with granular permissions.

### OAuthProvider
Supported identity providers for OIDC login.
- `Google`, `Microsoft`, `GitHub`, `Okta`.

### PermissionScope
Defines the target boundary for a permission grant.
- `Tenant`: Applies to all assets in the tenant.
- `Catalog`: Scoped to a specific catalog ID.
- `Namespace`: Scoped to a catalog + namespace path.
- `Asset`: Scoped to a specific asset ID.
- `Tag`: Applies to all assets with a matching business tag.

### Action
Atomic actions that can be permitted.
- `Read`, `Write`, `Delete`, `Create`, `Update`, `List`, `All`.
- `IngestBranching`, `ExperimentalBranching`.
- `ManageDiscovery`: Ability to edit business metadata.

---

## Conflict & Versioning

### ConflictType
Reasons for merge failure.
- `SchemaChange`, `DataOverlap`, `MetadataConflict`, `DeletionConflict`.

### ResolutionStrategy
Strategies for resolving detected conflicts.
- `AutoMerge`, `TakeSource`, `TakeTarget`, `Manual`, `ThreeWayMerge`.

### MergeStatus
Lifecycle of a merge operation.
- `Pending` (Detection) -> `Conflicted` / `Ready` -> `Resolving` -> `Completed` / `Aborted`.
