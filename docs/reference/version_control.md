# Version Control Reference

Pangolin operates like Git for data. You can Branch, Tag, and Merge data across namespaces.

## 1. API

### Create Branch
*   **Method**: `POST`
*   **Path**: `/api/v1/branches`
*   **Body**:
    ```json
    {
      "catalog": "analytics",
      "name": "feature-branch",
      "source_ref": "main"
    }
    ```

### Create Merge Operation
*   **Method**: `POST`
*   **Path**: `/api/v1/merges`
*   **Body**:
    ```json
    {
      "catalog": "analytics",
      "source": "feature-branch",
      "target": "main"
    }
    ```

### List Conflicts
*   **Method**: `GET`
*   **Path**: `/api/v1/merges/{merge_id}/conflicts`

### Resolve Conflict
*   **Method**: `POST`
*   **Path**: `/api/v1/merges/conflicts/{conflict_id}/resolve`
*   **Body**: `{"strategy": "KeepSource"}` (or `KeepTarget`)

### Complete Merge
*   **Method**: `POST`
*   **Path**: `/api/v1/merges/{merge_id}/complete`

---

## 2. CLI

### Branching
```bash
pangolin-admin create-branch \
  --catalog analytics \
  --name dev \
  --from main

pangolin-admin list-branches --catalog analytics
```

### Merging
```bash
# Start Merge
pangolin-admin create-merge \
  --catalog analytics \
  --source dev \
  --target main

# Check Conflicts
pangolin-admin list-merge-conflicts --id <merge-id>

# Resolve
pangolin-admin resolve-merge-conflict \
  --conflict-id <uuid> \
  --strategy KeepSource

# Complete
pangolin-admin complete-merge --id <merge-id>
```

---

## 3. Python SDK (`pypangolin`)

### Branching
```python
client.branches.create(
    catalog="analytics",
    name="experiment",
    source="main"
)
```

### Merging
```python
# 1. Start
merge_op = client.merge_operations.create(
    catalog="analytics",
    source="experiment",
    target="main"
)

# 2. Check Conflicts
if merge_op.conflicts:
    conflicts = client.merge_operations.list_conflicts(merge_op.id)
    for c in conflicts:
        client.merge_operations.resolve_conflict(c.id, "KeepSource")

# 3. Complete
client.merge_operations.complete(merge_op.id)
```

---

## 4. UI

1.  **Branches**:
    *   Navigate to **Catalogs**.
    *   Select a catalog.
    *   Go to **Branches** tab.
    *   Click **"New Branch"**.
2.  **Merging**:
    *   Go to **Merge Requests** (or similar tab in Catalog).
    *   Click **"New Merge"**.
    *   Select Source/Target.
    *   **Review**: UI shows diffs/conflicts.
    *   **Resolve**: Use UI buttons ("Keep Source" / "Keep Target") for each conflict.
    *   **Complete**: Click **Merge**.
