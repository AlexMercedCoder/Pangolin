# Entity & Model Overview

Pangolin's data model is designed to support multi-tenancy and Git-like branching for lakehouse assets.

## Core Entities

### Tenant
Represents an isolated customer or organization.
- **ID**: UUID
- **Name**: String
- **Properties**: Key-Value map

### Warehouse
Represents a physical storage location for data.
- **ID**: UUID
- **Name**: String
- **Storage Config**: Configuration for S3, Azure, GCS, etc.
- **Tenant ID**: UUID of the owning tenant

### Catalog
A collection of namespaces.
- **Name**: String (e.g., `warehouse`, `lake`)
- **Properties**: Key-Value map

### Namespace
A logical grouping of assets, similar to a database or schema.
- **Name**: Multi-part identifier (e.g., `["marketing", "campaigns"]`)
- **Properties**: Key-Value map

### Asset
A tracked object in the lakehouse (Table, View, Model, etc.).
- **ID**: UUID
- **Name**: String
- **Kind**: `IcebergTable`, `DeltaTable`, `HudiTable`, `ParquetTable`, `View`, `MlModel`, etc.
- **Location**: Storage URI
- **Properties**: Key-Value map

### Branch
A named pointer to a specific commit history.
- **Name**: String (e.g., `main`, `dev`)
- **Type**: `Ingest` (production-ready) or `Experimental` (isolated testing)
- **Head Commit**: Pointer to the latest commit on this branch.
- **Assets**: List of asset names tracked by this branch (for partial branching).

### Commit
An immutable record of a state change.
- **ID**: UUID
- **Parent ID**: UUID of the previous commit (optional)
- **Timestamp**: Unix timestamp
- **Author**: User who made the change
- **Message**: Description of the change
- **Operations**: List of changes (`Put Asset` or `Delete Asset`)

### Merge Operation
Tracked process for integrating changes between branches.
- **ID**: UUID
- **Status**: `Pending`, `Conflicted`, `Resolving`, `Ready`, `Completed`, or `Aborted`.
- **Conflicts**: List of detected conflicts requiring resolution.
- **Resolution Strategy**: `AutoMerge`, `TakeSource`, `TakeTarget`, `Manual`, `ThreeWayMerge`.
