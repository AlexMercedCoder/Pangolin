# Backend Gap Analysis & Branch Reading Investigation

## Executive Summary
This document details the findings from the investigation into:
1.  Unimplemented backend methods causing "Operation not supported" errors.
2.  The root cause of PyIceberg `table@branch` reading failures (404 errors).

**Key Findings**:
*   **MemoryStore** is missing the `list_merge_conflicts` implementation.
*   **All Backends** (Memory, Postgres, Mongo, SQLite) lack "Asset Propagation" logic in `create_branch`. When a branch is created, it starts empty (no assets), causing subsequent reads to fail even if the branch metadata exists.
*   **PyIceberg Parsing** is correct; the 404 was a legitimate "Asset Not Found" response from the backend, not a parsing error.

---

## 1. Backend Method Audit

The following table summarizes the status of critical methods across backends, focusing on the recent regressions.

| Feature Area | Method | Memory | Postgres | Mongo | SQLite | Status |
| :--- | :--- | :---: | :---: | :---: | :---: | :--- |
| **Merge Conficts** | `list_merge_conflicts` | ❌ **Missing** | ✅ Implemented | ✅ Implemented | ✅ Implemented | **Blocking** |
| **Merge Conficts** | `resolve_merge_conflict` | ✅ Implemented | ✅ Implemented | ✅ Implemented | ✅ Implemented | OK |
| **Assets** | `create_branch` (Asset Copy) | ❌ **Missing** | ❌ **Missing** | ❌ **Missing** | ❌ **Missing** | **Blocking** |

### Detailed Findings

#### MemoryStore
*   **Missing**: `list_merge_conflicts_internal` is not defined in `merge.rs`, causing the trait implementation to fallback to the default error.
*   **Impact**: Verification scripts running against MemoryStore (fast tests) fail on merge validation.

#### PostgresStore
*   **Implemented**: `list_merge_conflicts` is correctly implemented in `main.rs`.
*   **Note**: If verification failed here, it was likely due to the test environment using defaults (MemoryStore) rather than Postgres.

---

## 2. Branch Reading Investigation (PyIceberg 404)

### The Symptom
The verification script failed with `404 Table Not Found` when attempting to load a table from a newly created branch:
```python
client.branches.create("dev", from_branch="main")
# ...
table_dev = cat.load_table("namespace.table@dev") # -> 404 Not Found
```

### The Root Cause
The issue is **not** in PyIceberg's request construction or the API's URL parsing.
*   `pangolin_api/src/iceberg/types.rs` correctly verifies `parse_table_identifier` handles `table@branch`.
*   `tables.rs` correctly extracts the branch.

**The Failure**:
1.  **Naive Implementations**: The current backend `create_branch` implementations (Memory, Postgres, etc.) simply insert a `Branch` metadata record.
2.  **No Asset Copy**: They do **not** copy the asset references from the source branch (e.g., `main`) to the new branch.
3.  **Isolation Model**: Pangolin's current storage model associates assets explicitly with a branch. If `dev` branch has no entries in the `assets` table/map, `get_asset(..., branch="dev")` returns `None`.

### Solution: Asset Propagation
To support correct branching behavior in these store implementations, `create_branch` logic must change to:
1.  **Lookup Source Branch**: Retrieve the source branch (e.g., 'main').
2.  **List Source Assets**: Identify all assets currently linked to the source branch.
3.  **Duplicate Entries**: Insert new asset records for the `target_branch` pointing to the same metadata locations/IDs as the source assets.
    *   *Memory*: Insert new keys into `self.assets` map.
    *   *Postgres/SQL*: Execute `INSERT INTO assets ... SELECT ... FROM assets WHERE branch = 'source'`.

---

## 3. Recommended Remediation Plan

### Phase 1: Fix MemoryStore Parity
1.  **Implement `list_merge_conflicts`**: Add `list_merge_conflicts_internal` to `pangolin_store/src/memory/merge.rs` and update `pangolin_store/src/memory/mod.rs` to call it.

### Phase 2: Fix Branch Isolation (All Backends)
1.  **Update `create_branch` Signature**: (Internal only) Ensure it has context to know the source branch assets.
2.  **MemoryStore Update**: Modify `create_branch_internal` to copy assets.
3.  **Postgres/SQLite/Mongo Update**: Modify `create_branch` to perform a "Copy-on-Create" operation for assets.

### Impact Analysis
*   **API**: No changes required.
*   **CLI**: No changes required.
*   **PyPangolin**: No changes required.
*   **UI**: Will implicitly benefit from working branch visualization.
