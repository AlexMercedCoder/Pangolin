# Merge Principles & Conflict Management

In a Data Lakehouse, merging branches is more complex than merging code because the underlying data is massive and mutable. Pangolin implements specific strategies to handle these updates safely.

> [!NOTE]
> For the step-by-step API guide and technical details on the 3-Way Merge algorithm, see the **[Merge Operations](./merge_operations.md)** guide.

## Types of Changes

Pangolin tracks three types of changes to assets (tables/views) when merging a branch:

1.  **Append (New Data)**: Adding new files/rows to a table.
2.  **Schema Update**: Changing columns (add, drop, rename, type change).
3.  **Rewrite (Optimize/Update/Delete)**: Rewriting existing data files (e.g., compaction or row-level updates).

## When Do Conflicts Occur?

A **Merge Conflict** occurs when the *same asset* has been modified on both the **Source Branch** (the one you are merging) and the **Target Branch** (usually `main`) since they diverged, in a way that cannot be automatically reconciled.

| Change Type A (Target) | Change Type B (Source) | Conflict? | Resolution |
| :--- | :--- | :--- | :--- |
| **Append** | **Append** | **No** | Pangolin automatically combines both sets of new files. |
| **Schema Change** | **Append** | **Maybe** | If schema change breaks new data (e.g., deleted a column present in new data), it conflicts. |
| **Rewrite** | **Append** | **No** | Generally safe; new data is added to the rewritten table state. |
| **Rewrite** | **Rewrite** | **YES** | **Concurrent Update Conflict**. Both branches modified the same partition/files. |
| **Schema Change** | **Schema Change** | **YES** | Divergent schemas can rarely be auto-merged. |

### The "Concurrent Update" Problem
If Branch A compacts `partition_1` and Branch B deletes rows from `partition_1`, merging them is impossible without losing data. One operation must be discarded or re-played.

## Best Practices to Avoid Conflicts

### 1. Short-Lived Branches
The longer a branch exists, the more likely `main` has diverged. Merge frequently.

### 2. Isolate Scope (Partial Branching)
Use Pangolin's **Partial Branching** feature to only include the specific tables you are working on.
*   *Bad*: Branching the massive `sales_fact` table just to update the `dim_users` table.
*   *Good*: Create a branch containing only `dim_users`.

### 3. Coordinate Compaction
Schedule maintenance jobs (Compaction/Vacuum) on the `main` branch during windows when other write branches are not active, or coordinate with your team.

### 4. Rebase/Sync Often
If your branch is long-lived, periodically **Rebase** (merge `main` into your feature branch) to resolve conflicts early in your isolated environment rather than at the final merge.
