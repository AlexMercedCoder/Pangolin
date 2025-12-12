# Branch Management

Pangolin brings Git-like semantics to your data catalog, allowing you to isolate changes, experiment safely, and manage data lifecycles effectively.

## Core Concepts

### Branches
A branch is a named pointer to a specific state of your catalog.
- **main**: The default branch, typically representing the "production" state.
- **Feature Branches**: Short-lived branches for testing new schemas or data loads (e.g., `dev`, `feature/new-schema`).

### Partial Branching
Unlike Git, where a branch copies the entire repository, Pangolin supports **Partial Branching**. You can create a branch that only tracks a subset of tables.
- **Use Case**: You want to test a schema change on `users` table without duplicating the state of 1,000 other tables in your warehouse.
- **Behavior**: When you create a branch with specific assets, only those assets are "forked". Other assets are not visible or accessible on that branch.

## Workflows

### 1. Creating a Branch

**Full Branch (Copy Everything)**
*Currently, full catalog branching copies all asset pointers.*
```bash
curl -X POST /api/v1/branches -d '{
  "name": "audit-audit",
  "from_branch": "main"
}'
```

**Partial Branch (Specific Assets)**
```bash
curl -X POST /api/v1/branches -d '{
  "name": "dev",
  "from_branch": "main",
  "assets": ["data_team.users", "sales.orders"]
}'
```

### 2. Working on a Branch
To read or write to a specific branch, append the `@branchName` suffix to the table or namespace in your Iceberg client or API call.

**API Example:**
`POST /v1/.../tables/users?branch=dev`

**SQL Example (if supported by engine):**
`SELECT * FROM pangolin.data_team.users.branch_dev`

### 3. Merging a Branch
Merging applies changes from a source branch to a target branch.

**Endpoint:** `POST /api/v1/branches/merge`

**Payload:**
```json
{
  "source_branch": "dev",
  "target_branch": "main"
}
```

**Merge Logic:**
1.  Identify assets tracked by `source_branch`.
2.  For each asset, check if it has changed compared to `target_branch`.
3.  Update the `target_branch` to point to the new metadata location of the asset.
4.  If the asset is new, it is added to `target_branch`.

## Best Practices
- **Isolation**: Always use a new branch for schema evolution or bulk data ingestion.
- **Cleanup**: Delete experimental branches after dpc.
- **Naming**: Use descriptive names like `user/feature-x` or `ingest/batch-123`.
