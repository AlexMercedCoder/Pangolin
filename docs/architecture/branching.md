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

## Workflows

### Isolation (Experimentation)
1. User creates branch `dev/experiment` from `main`.
2. Changes made in `dev/experiment` are isolated. `main` users continue to see the old data.
3. If the experiment fails, the branch is deleted. No impact on production.

### Zero-Copy Ingestion
1. ETL job writes data to `etl-job-123` branch.
2. Data quality checks run against this branch.
3. If checks pass, `etl-job-123` is merged into `main` atomically.
4. If checks fail, the branch is discarded.

## Merge Logic

Merging involves taking changes from a `source` branch and applying them to a `target` branch.

### 3-Way Merge Strategy
Pangolin uses a 3-way merge algorithm to detect conflicts:
1. Identify the **Base Commit** (common ancestor).
2. Compare **Source** vs **Base** to find changes.
3. Compare **Target** vs **Base** to find changes.
4. Detect conflicts where both Source and Target modified the same asset.

### Conflict Types
- **Schema Conflict**: Table schema evolved incompatibly in both branches.
- **Data Conflict**: Both branches wrote to the same partitions.
- **Metadata Conflict**: Table properties changed incompatibly.

### Resolution
- **Auto-Merge**: If changes are orthogonal (e.g., Branch A touched Table X, Branch B touched Table Y), they are merged automatically.
- **Manual**: If a conflict is detected, the merge operation pauses (`Conflicted` status) and requires API intervention to choose a winner (`TakeSource` or `TakeTarget`).
