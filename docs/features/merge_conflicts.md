# Merge Principles & Conflict Management

In a Data Lakehouse, merging branches is more complex than merging code because the underlying data is massive and mutable. Pangolin implements specific strategies to handle these updates safely.

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


## Current Implementation: Fast-Forward Merging

In the current version of Pangolin, the default merge strategy is **Fast-Forward**.

### How it works
When you merge a `source` branch into a `target` branch:
1.  Pangolin updates the `target` branch's `head_commit_id` to match the `source` branch.
2.  The `target` branch effectively "jumps" to the state of the `source` branch.

### Implications
-   **No Conflicts**: Since this is a pointer update, "conflicts" (divergent histories) are not currently enforced. The Source branch state becomes the new Target state.
-   **Best Practice**: Ensure your source branch is up-to-date with the target before working to avoid accidentally reverting changes on the target (though in a data lake, this is less destructive than in code).

> **Roadmap Feature**: Advanced 3-way merge with Optimistic Concurrency Control (OCC) and row-level conflict detection is planned for a future release. The section below describes the *target* architecture for that release.

## Planned: Conflict Resolution (Future)

*The following describes the planned behavior for the upcoming Three-Way Merge support.*

Currently, Pangolin adopts a **Optimistic Concurrency Control** (OCC) model...

