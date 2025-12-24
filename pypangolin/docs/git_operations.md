# Git-like Operations

Starting with version 0.2.0, `pypangolin` supports Git-like semantics for managing table versions, branches, tags, and merges. These features allow for powerful data versioning workflows directly from Python.

## Branching

Create and manage branches to isolate changes or experiment with data.

```python
from pypangolin import PangolinClient

client = PangolinClient(...)

# Create a new branch 'dev' from 'main'
# Branch creation is scoped to a catalog
dev_branch = client.branches.create("dev", from_branch="main", catalog_name="my_catalog")
print(f"Created branch {dev_branch.name} at commit {dev_branch.head_commit_id}")

# List active branches
branches = client.branches.list(catalog_name="my_catalog")
for b in branches:
    print(f"- {b.name} ({b.branch_type})")

# Get branch details
branch = client.branches.get("dev")
```

## Commits & History

Inspect the history of changes on a branch.

```python
# List commits on the 'main' branch
commits = client.branches.list_commits("main", catalog_name="my_catalog")

for commit in commits:
    print(f"[{commit.id[:8]}] {commit.message} - {commit.timestamp}")
```

## Merging

Merge changes from one branch into another. Pangolin handles conflict detection for schemas and data.

```python
# Merge 'dev' into 'main'
try:
    operation = client.branches.merge(
        source_branch="dev", 
        target_branch="main", 
        catalog_name="my_catalog"
    )
    
    if operation.status == "merged":
        print("Merge successful!")
        
    elif operation.status == "conflicted":
        print(f"Merge conflict detected! Operation ID: {operation.id}")
        
        # List conflicts
        conflicts = client.merge_operations.list_conflicts(operation.id)
        for c in conflicts:
            print(f"Conflict in {c.asset_name}: {c.conflict_type}")
            
        # Resolve conflicts (Example: 'source' wins)
        for c in conflicts:
            client.merge_operations.resolve_conflict(c.id, resolution="source")
            
        # Complete the merge
        client.merge_operations.complete(operation.id)
        print("Merge completed after resolution.")

except Exception as e:
    print(f"Merge failed: {e}")
```

## Tagging

Tag specific points in history for releases or snapshots.

```python
# Create a tag on the current head of 'main'
main_branch = client.branches.get("main")
if main_branch.head_commit_id:
    tag = client.tags.create(
        "v1.0.0", 
        commit_id=main_branch.head_commit_id, 
        catalog_name="my_catalog"
    )
    print(f"Tagged release {tag.name}")

# List tags
tags = client.tags.list(catalog_name="my_catalog")
```
