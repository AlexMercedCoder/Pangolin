# Merging & Conflict Resolution

Pangolin supports advanced branching and merging capabilities for data catalogs, similar to Git but specialized for data lake table formats like Iceberg. This document explains the merge logic, conflict detection, and resolution strategies.

## Overview

Pangolin's Enhanced Merge Conflict Resolution system provides intelligent conflict detection and resolution when merging branches. Unlike simple merge operations that fail on any conflict, this system:

- **Detects** conflicts automatically during merge operations
- **Categorizes** conflicts by type (schema, deletion, metadata, data overlap)
- **Tracks** merge operations with full lifecycle management
- **Enables** manual resolution with multiple strategies
- **Supports** automatic resolution for non-critical conflicts

## Merge Logic

Pangolin uses a **3-Way Merge** algorithm when a common ancestor (base commit) can be detected. If no common ancestor is found (e.g., disjoint histories), it falls back to a 2-way merge (Source overwrites Target).

### 1. Base Commit Detection
When you initiate a merge from `Feature Branch` to `Main`, Pangolin:
1.  Traverses the commit history of both branches.
2.  Finds the most recent common ancestor (Base Commit).
3.  Calculates changes:
    *   **Source Changes**: Base -> Source Head
    *   **Target Changes**: Base -> Target Head

### 2. Conflict Detection
Pangolin detects conflicts by comparing modifications to assets in Source and Target relative to the Base.

| Scenario | Source | Target | Result |
| :--- | :--- | :--- | :--- |
| **No Change** | Same as Base | Same as Base | No Change |
| **Fast-Forward** | Modified | Same as Base | Update to Source |
| **Incoming** | Same as Base | Modified | Keep Target |
| **Conflict** | Modified (A) | Modified (B) | **CONFLICT** |
| **Deletion** | Deleted | Modified | **CONFLICT** (Deletion Conflict) |
| **Resurrection**| Created | Created | **CONFLICT** (if different content) |

## Conflict Types

### 1. Schema Change Conflicts
Occur when the same table has different schemas in source and target branches.
- **Example**: Both branches added columns independently or changed column types.
- **Resolution**: Requires manual review to merge schema changes properly.

### 2. Deletion Conflicts
Occur when an asset is deleted in one branch but modified in another.
- **Example**: Source deleted `temp_table` while Target added data to it.
- **Resolution**: Choose to keep (with modifications) or delete the asset.

### 3. Metadata Conflicts
Occur when asset properties (tags, description, properties) differ between branches.
- **Resolution**: Can often be auto-resolved for non-critical properties.

### 4. Data Overlap Conflicts
Occur when the same partitions of a table are modified in both branches.
- **Resolution**: Requires manual review to determine which version to keep.

## Merge Workflow

### 1. Initiate Merge
```http
POST /api/v1/branches/merge
{
  "catalog": "production",
  "source_branch": "feature-branch",
  "target_branch": "main"
}
```

**Response (Conflicts Detected):**
```json
{
  "status": "conflicted",
  "operation_id": "550e8400-e29b...",
  "conflicts": 3,
  "message": "Merge has 3 conflicts that need resolution"
}
```

### 2. List Conflicts
```http
GET /api/v1/merge-operations/{operation_id}/conflicts
```

### 3. Resolve Conflicts
```http
POST /api/v1/conflicts/{conflict_id}/resolve
{
  "strategy": "TakeSource",
  "resolved_value": null
}
```

**Resolution Strategies:**
- `TakeSource`: Use the source branch version.
- `TakeTarget`: Use the target branch version.
- `Manual`: Provide custom resolution (e.g., a merged schema) in `resolved_value`.
- `AutoMerge`: Automatically merge non-conflicting changes.

### 4. Complete Merge
After all conflicts are resolved:
```http
POST /api/v1/merge-operations/{operation_id}/complete
```

### 5. Abort Merge
If you decide not to proceed:
```http
POST /api/v1/merge-operations/{operation_id}/abort
```

## Merge Operation Lifecycle

```
Pending → Conflicted → Resolving → Ready → Completed
   ↓                                           ↑
   └─────────────→ Aborted ←──────────────────┘
```

## Managing Merge Operations

### List All Merge Operations
```http
GET /api/v1/catalogs/{catalog_name}/merge-operations
```

### Get Merge Operation Details
```http
GET /api/v1/merge-operations/{operation_id}
```

## Best Practices
- **Merge Frequently**: Minimize divergence between branches.
- **Review Thoroughly**: Schema and deletion conflicts can be destructive.
- **Test After Merge**: Always validate data and schema after a complex merge.
