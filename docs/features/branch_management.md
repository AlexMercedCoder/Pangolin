# Branch Management

Pangolin brings Git-like semantics to your data catalog, allowing you to isolate changes, experiment safely, and manage data lifecycles with high precision.

---

## 💡 Core Concepts

### 1. Partial Branching (The "Asset Table")
Unlike Git, which tracks an entire repository, a Pangolin branch acts as a "Filtered View" or "Asset Table." 
- When you create a branch, you specify a set of assets (tables/views) to "fork."
- Only the metadata pointers for those specific assets are copied.
- This allows you to work on `table_A` in isolation without involving `table_B` through `table_Z`.

### 2. Fork-on-Write Behavior
Pangolin supports an **Auto-Add** mechanism during asset creation:
- **New Assets**: If you create a new table using the `@branch` suffix (e.g., `new_table@dev`), Pangolin will automatically create the `dev` branch if it does not exist.
- **Existing Assets**: To work on an existing table in a new branch, the table must first be "added" to the branch metadata. This happens either during explicit branch creation (specifying `assets`) or when the table is first written to on that branch.

> [!WARNING]
> If you attempt to **Update** a table on a branch where it hasn't been added yet (e.g., `update my_table@new_branch`), the operation will return `NOT_FOUND`. You must first fork the table into the branch.

---

## 🛠️ Usage Guides

### 1. Via CLI
The `pangolin-user` tool provides the most direct way to manage branches.

**Create a Branch:**
```bash
# Create dev branch from main with specific tables
pangolin-user create-branch dev --from main --assets data_team.users,sales.orders
```

**List Branches:**
```bash
pangolin-user list-branches --catalog production
```

**Merge a Branch:**
```bash
# Integrates dev into main
pangolin-user merge-branch dev main --catalog production
```

### 2. Via API (Iceberg Clients)
Most Iceberg clients support branching through the `@` syntax in the table identifier.

**Spark/PyIceberg Example:**
```python
# Create a new table on a branch (Auto-creates the branch)
df.writeTo("production.db.my_table@feature_x").create()

# Read from a branch
df = spark.read.table("production.db.my_table@feature_x")
```

### 3. Via Management UI
1. Navigate to the **Data Explorer**.
2. Select your **Catalog**.
3. Use the **Branch Selector** in the top-right to switch contexts.
4. To create a branch from the UI, click **Manage Branches** in the sidebar or the selector dropdown.

---

## 🧬 Branch Types & Permissions

| Type | Permission Required | Use Case |
| :--- | :--- | :--- |
| **Main** | `Write` on Catalog | The source of truth. |
| **Experimental** | `ExperimentalBranching` | Short-lived feature testing. |
| **Ingest** | `IngestBranching` | Staging area for bulk data loads before production merge. |

For detailed permission cascading, see the [Permissions System Guide](../permissions.md).

---

## 🚦 Best Practices
- **Atomic Changes**: Use branches for schema evolutions to ensure `main` never sees a broken state.
- **Audit Consistency**: All branch-specific operations are logged with the branch context, allowing you to trace `Who changed What on Which branch`.
- **Merge Often**: To avoid complex 3-way merge conflicts, keep feature branches short-lived and rebase frequently.
