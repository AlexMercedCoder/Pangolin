# Branching & Merging (Git-for-Data)

Pangolin implements a "Git-for-Data" model similar to Project Nessie, allowing users to isolate changes, experiment with data, and safely merge updates.

## Core Concepts

### 1. Branches
A **Branch** is a named pointer to a specific commit hash in the history.
- **main**: The default branch (protected).
- **dev / feature-***: Temporary branches for ingestion or experimentation.

Just like Git, creating a branch is cheap (O(1)) as it is just a reference copy.

### 2. Commits
A **Commit** is an immutable record of a state change.
- **Operations**: Each commit contains a list of `Put` (add/update table) or `Delete` (remove table) operations.
- **Parent**: Links to the previous commit, forming a Directed Acyclic Graph (DAG).
- **Snapshot**: In Iceberg terms, a commit points to a specific `metadata.json` location for a table.

### 3. Tags
A **Tag** is an immutable reference to a specific commit. Useful for marking releases (e.g., `Q1_REPORT_FINAL`).

---

## Merge Lifecycle

Merging is managed via the `MergeOperation` model, which tracks the transition from initiation to completion.

### 1. Initiation
A merge is started between a `source` and `target` branch. Pangolin identifies the **Base Commit** (common ancestor) to perform a 3-way analysis.

### 2. Conflict Detection
Pangolin automatically detects:
- **Schema Conflict**: Incompatible evolution on both branches.
- **Data Conflict**: Concurrent writes to overlapping partitions.
- **Metadata Conflict**: Conflicting table property changes.

### 3. Resolution
- **Auto-Merge**: Orthogonal changes are applied automatically, and the operation moves to `Completed`.
- **Manual Intervention**: If conflicts are found, the operation enters `Conflicted` status. Users must use the API to resolve each conflict using a `ResolutionStrategy` (`TakeSource`, `TakeTarget`, or `ThreeWayMerge`).

### 4. Completion
Once all conflicts are resolved, the merge is finalized, creating a new commit on the target branch.
