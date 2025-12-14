# Enhanced Merge Conflict Resolution

## Overview

Pangolin's Enhanced Merge Conflict Resolution system provides intelligent conflict detection and resolution when merging branches. Unlike simple merge operations that fail on any conflict, this system:

- **Detects** conflicts automatically during merge operations
- **Categorizes** conflicts by type (schema, deletion, metadata, data overlap)
- **Tracks** merge operations with full lifecycle management
- **Enables** manual resolution with multiple strategies
- **Supports** automatic resolution for non-critical conflicts

## Conflict Types

### 1. Schema Change Conflicts

Occur when the same table has different schemas in source and target branches.

**Example:**
- Source branch: Added column `email` to `users` table
- Target branch: Added column `phone` to `users` table
- **Conflict**: Both branches modified the schema independently

**Resolution**: Requires manual review to merge schema changes properly.

### 2. Deletion Conflicts

Occur when an asset is deleted in one branch but modified in another.

**Example:**
- Source branch: Deleted `temp_table`
- Target branch: Modified `temp_table` (added data)
- **Conflict**: Cannot delete a table that has been modified

**Resolution**: Choose to keep (with modifications) or delete the asset.

### 3. Metadata Conflicts

Occur when asset properties differ between branches.

**Example:**
- Source branch: Set `description = "Production data"`
- Target branch: Set `description = "Staging data"`
- **Conflict**: Same property, different values

**Resolution**: Can often be auto-resolved for non-critical properties.

### 4. Data Overlap Conflicts

Occur when the same partitions are modified in both branches.

**Example:**
- Source branch: Updated partition `date=2024-01-01`
- Target branch: Updated partition `date=2024-01-01`
- **Conflict**: Same data modified in both branches

**Resolution**: Requires manual review to determine which version to keep.

## Merge Workflow

### 1. Initiate Merge

```bash
POST /api/v1/branches/merge
Authorization: Bearer <token>
Content-Type: application/json

{
  "catalog": "production",
  "source_branch": "feature-branch",
  "target_branch": "main"
}
```

**Response (No Conflicts):**
```json
{
  "status": "merged",
  "operation_id": "550e8400-e29b-41d4-a716-446655440000",
  "commit_id": "660e8400-e29b-41d4-a716-446655440111"
}
```

**Response (Conflicts Detected):**
```json
{
  "status": "conflicted",
  "operation_id": "550e8400-e29b-41d4-a716-446655440000",
  "conflicts": 3,
  "message": "Merge has 3 conflicts that need resolution"
}
```

### 2. List Conflicts

```bash
GET /api/v1/merge-operations/{operation_id}/conflicts
Authorization: Bearer <token>
```

**Response:**
```json
[
  {
    "id": "770e8400-e29b-41d4-a716-446655440222",
    "conflict_type": {
      "SchemaChange": {
        "asset_name": "users",
        "source_schema": {...},
        "target_schema": {...}
      }
    },
    "description": "Schema conflict detected for asset 'users'",
    "resolution": null,
    "created_at": "2024-01-15T10:30:00Z"
  }
]
```

### 3. Resolve Conflicts

```bash
POST /api/v1/conflicts/{conflict_id}/resolve
Authorization: Bearer <token>
Content-Type: application/json

{
  "strategy": "TakeSource",
  "resolved_value": null
}
```

**Resolution Strategies:**
- `TakeSource`: Use the source branch version
- `TakeTarget`: Use the target branch version  
- `Manual`: Provide custom resolution in `resolved_value`
- `AutoMerge`: Automatically merge non-conflicting changes
- `ThreeWayMerge`: Use base commit as reference (advanced)

### 4. Complete Merge

After all conflicts are resolved:

```bash
POST /api/v1/merge-operations/{operation_id}/complete
Authorization: Bearer <token>
```

**Response:**
```json
{
  "status": "completed",
  "operation_id": "550e8400-e29b-41d4-a716-446655440000",
  "commit_id": "880e8400-e29b-41d4-a716-446655440333"
}
```

### 5. Abort Merge (Optional)

If you decide not to proceed:

```bash
POST /api/v1/merge-operations/{operation_id}/abort
Authorization: Bearer <token>
```

## Merge Operation Lifecycle

```
Pending → Conflicted → Resolving → Ready → Completed
   ↓                                           ↑
   └─────────────→ Aborted ←──────────────────┘
```

**States:**
- **Pending**: Merge initiated, detecting conflicts
- **Conflicted**: Conflicts detected, awaiting resolution
- **Resolving**: Manual resolution in progress
- **Ready**: All conflicts resolved, ready to complete
- **Completed**: Merge successfully completed
- **Aborted**: Merge cancelled by user

## Managing Merge Operations

### List All Merge Operations

```bash
GET /api/v1/catalogs/{catalog_name}/merge-operations
Authorization: Bearer <token>
```

### Get Merge Operation Details

```bash
GET /api/v1/merge-operations/{operation_id}
Authorization: Bearer <token>
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "tenant_id": "...",
  "catalog_name": "production",
  "source_branch": "feature-branch",
  "target_branch": "main",
  "status": "Conflicted",
  "conflicts": ["770e8400...", "880e8400..."],
  "initiated_by": "990e8400-e29b-41d4-a716-446655440444",
  "initiated_at": "2024-01-15T10:00:00Z",
  "completed_at": null
}
```

## Best Practices

### 1. **Merge Frequently**
Merge feature branches into main regularly to minimize conflicts.

### 2. **Review Conflicts Carefully**
Schema and deletion conflicts can have significant impacts. Review thoroughly before resolving.

### 3. **Use Descriptive Branch Names**
Clear branch names help identify the purpose of changes when resolving conflicts.

### 4. **Test After Merge**
Always validate data and schema after completing a merge with conflicts.

### 5. **Document Resolutions**
Keep notes on why you chose specific resolution strategies for complex conflicts.

## Resolution Strategy Guide

| Conflict Type | Recommended Strategy | Notes |
|--------------|---------------------|-------|
| Schema Change | Manual | Requires careful review to merge schemas |
| Deletion (asset deleted in source) | TakeSource or TakeTarget | Decide if deletion should proceed |
| Deletion (asset deleted in target) | TakeSource or TakeTarget | Decide if new changes should be kept |
| Metadata (non-critical properties) | AutoMerge | Safe for descriptions, tags, etc. |
| Metadata (critical properties) | Manual | Review partition specs, schemas |
| Data Overlap | Manual | Requires data-level analysis |

## Automatic Resolution

The system can automatically resolve certain conflicts:

**Auto-Resolvable:**
- Metadata conflicts in non-critical properties (description, tags)
- Non-overlapping changes to different properties

**Requires Manual Resolution:**
- Schema changes
- Deletion conflicts
- Data overlap conflicts
- Conflicts in critical properties (schema, partitioning)

## Example Workflow

### Scenario: Feature Branch with Schema Changes

1. **Developer creates feature branch:**
   ```bash
   POST /api/v1/branches
   {"name": "add-email-column", "catalog": "production"}
   ```

2. **Developer adds email column to users table**
   - Modifies schema on `add-email-column` branch

3. **Meanwhile, another developer adds phone column on main**
   - Schema diverges between branches

4. **Attempt to merge:**
   ```bash
   POST /api/v1/branches/merge
   {
     "source_branch": "add-email-column",
     "target_branch": "main"
   }
   ```

5. **System detects schema conflict:**
   ```json
   {
     "status": "conflicted",
     "operation_id": "...",
     "conflicts": 1
   }
   ```

6. **Review conflict:**
   ```bash
   GET /api/v1/merge-operations/{operation_id}/conflicts
   ```

7. **Resolve by merging both columns:**
   ```bash
   POST /api/v1/conflicts/{conflict_id}/resolve
   {
     "strategy": "Manual",
     "resolved_value": {
       "schema": {
         "columns": [
           {"name": "id", "type": "int"},
           {"name": "name", "type": "string"},
           {"name": "email", "type": "string"},
           {"name": "phone", "type": "string"}
         ]
       }
     }
   }
   ```

8. **Complete merge:**
   ```bash
   POST /api/v1/merge-operations/{operation_id}/complete
   ```

## Troubleshooting

### "Cannot complete merge: conflicts not resolved"
- **Cause**: Some conflicts still have `resolution: null`
- **Solution**: Resolve all conflicts before calling `/complete`

### "Merge operation not found"
- **Cause**: Invalid operation ID or operation was aborted
- **Solution**: List merge operations to find the correct ID

### "Operation not ready"
- **Cause**: Merge status is not `Ready` or `Pending`
- **Solution**: Check operation status and resolve conflicts

### Conflict detection seems incomplete
- **Cause**: Only compares assets in existing namespaces
- **Solution**: Ensure both branches have been properly committed

## API Reference Summary

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/v1/branches/merge` | POST | Initiate merge with conflict detection |
| `/api/v1/catalogs/:catalog/merge-operations` | GET | List merge operations |
| `/api/v1/merge-operations/:id` | GET | Get operation details |
| `/api/v1/merge-operations/:id/conflicts` | GET | List conflicts |
| `/api/v1/conflicts/:id/resolve` | POST | Resolve a conflict |
| `/api/v1/merge-operations/:id/complete` | POST | Complete merge |
| `/api/v1/merge-operations/:id/abort` | POST | Abort merge |

## Implementation Details

### Conflict Detection Algorithm

1. **Fetch assets** from source and target branches
2. **Compare schemas** for matching assets
3. **Detect deletions** (assets in one branch but not the other)
4. **Compare metadata** for property conflicts
5. **Create MergeConflict** records for each issue
6. **Store conflicts** and link to MergeOperation

### Data Models

```rust
pub struct MergeOperation {
    pub id: Uuid,
    pub status: MergeStatus,
    pub source_branch: String,
    pub target_branch: String,
    pub conflicts: Vec<Uuid>,
    // ...
}

pub struct MergeConflict {
    pub id: Uuid,
    pub conflict_type: ConflictType,
    pub resolution: Option<ConflictResolution>,
    // ...
}
```

## Related Documentation

- [Branch Management](./features/branch_management.md)
- [Time Travel](./features/time_travel.md)
- [API Reference](./api/api_overview.md)
