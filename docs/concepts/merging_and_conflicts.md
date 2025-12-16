# Merging & Conflict Resolution

Pangolin supports advanced branching and merging capabilities for data catalogs, similar to Git but specialized for data lake table formats like Iceberg. This document explains the merge logic, conflict detection, and resolution strategies.

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

### 3. Conflict Types

*   **SchemaChange**: Both branches modified the schema of the same table in incompatible ways (e.g., adding columns with same name but different types).
*   **DataOverlap**: Both branches added data files to the same partition (optimistic concurrency failure).
*   **MetadataConflict**: Conflicting updates to table properties or snapshots.
*   **DeletionConflict**: One branch deleted an asset while the other modified it.

## Manual Conflict Resolution

When a merge operation enters the `Conflicted` state, it pauses. You must resolve each conflict before the merge can complete.

### Resolution Strategies

1.  **TakeSource**: Discard target changes and apply source version.
    *   *Use case*: The feature branch is the source of truth.
2.  **TakeTarget**: Discard source changes and keep target version.
    *   *Use case*: The main branch has critical hotfixes that supersede the feature branch.
3.  **Manual (JSON Patch)**: Supply a specific JSON value to serve as the resolved state.
    *   *Use case*: Merging two schemas manually (e.g., keeping columns from both).

### Resolution Workflow (API)

1.  **List Conflicts**:
    ```http
    GET /api/v1/merge-operations/{operation_id}/conflicts
    ```

2.  **Resolve Each Conflict**:
    ```http
    POST /api/v1/conflicts/{conflict_id}/resolve
    {
      "strategy": "TakeSource"
    }
    ```
    OR
    ```http
    POST /api/v1/conflicts/{conflict_id}/resolve
    {
      "strategy": "Manual",
      "resolved_value": { ...merged schema... }
    }
    ```

3.  **Complete Merge**:
    Once all conflicts are resolved:
    ```http
    POST /api/v1/merge-operations/{operation_id}/complete
    ```

## Automated Merging

If no conflicts are detected during the initial check, Pangolin automatically:
1.  Applies all Source changes to the Target branch.
2.  Creates a new Merge Commit on the Target branch.
3.  Updates the Target Branch head.
