# Entity & Model Overview

Pangolin's data model is designed to support multi-tenancy and Git-like branching for lakehouse assets.

## Core Entities

### Tenant
Represents an isolated customer or organization.
- **ID**: UUID
- **Name**: String
- **Properties**: Key-Value map

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
- **Name**: String
- **Kind**: `IcebergTable`, `DeltaTable`, `View`, `MlModel`, etc.
- **Location**: Storage URI
- **Properties**: Key-Value map

### Branch
A named pointer to a specific commit history.
- **Name**: String (e.g., `main`, `dev`, `feature-1`)
- **Type**: `Ingest` (write-only) or `Experimental`
- **Head Commit**: Pointer to the latest commit on this branch.

### Commit
An immutable record of a state change.
- **ID**: UUID
- **Parent ID**: UUID of the previous commit
- **Timestamp**: Epoch time
- **Author**: User who made the change
- **Operations**: List of changes (Put Asset, Delete Asset)
