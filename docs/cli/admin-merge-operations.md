# Admin CLI - Merge Operations

This guide covers the complete merge workflow for branch management in Pangolin.

---

## Overview

Merge operations allow you to merge changes from one branch into another. The workflow includes:
1. List merge operations
2. Check for conflicts
3. Resolve conflicts
4. Complete or abort the merge

---

## List Merge Operations

List all merge operations in the current catalog.

### Syntax
```bash
pangolin-admin list-merge-operations
```

### Parameters
None

### Example
```bash
pangolin-admin list-merge-operations
```

### Output
```
+--------------------------------------+----------------+---------------+---------+---------------------+
| ID                                   | Source Branch  | Target Branch | Status  | Created At          |
+--------------------------------------+----------------+---------------+---------+---------------------+
| merge-uuid-1                         | feature-branch | main          | pending | 2025-12-18T10:30:00 |
+--------------------------------------+----------------+---------------+---------+---------------------+
```

### Notes
- Shows all merge operations for the current catalog
- Status can be: pending, in-progress, completed, aborted
- Use merge ID for subsequent operations

---

## Get Merge Operation

Get detailed information about a specific merge operation.

### Syntax
```bash
pangolin-admin get-merge-operation --id <MERGE_ID>
```

### Parameters
- `--id` - UUID of the merge operation (required)

### Example
```bash
pangolin-admin get-merge-operation --id "merge-uuid"
```

### Output
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Merge Operation Details
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ID: merge-uuid
Source Branch: feature-branch
Target Branch: main
Status: pending
Created At: 2025-12-18T10:30:00
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### Notes
- Provides full details of the merge operation
- Includes timestamps and status
- Shows completed timestamp if merge is finished

---

## List Conflicts

List all conflicts in a merge operation.

### Syntax
```bash
pangolin-admin list-conflicts --merge-id <MERGE_ID>
```

### Parameters
- `--merge-id` - UUID of the merge operation (required)

### Example
```bash
pangolin-admin list-conflicts --merge-id "merge-uuid"
```

### Output
```
+--------------------------------------+----------+---------------+----------+
| ID                                   | Asset ID | Conflict Type | Resolved |
+--------------------------------------+----------+---------------+----------+
| conflict-uuid-1                      | table-1  | schema        | No       |
+--------------------------------------+----------+---------------+----------+
| conflict-uuid-2                      | table-2  | data          | Yes      |
+--------------------------------------+----------+---------------+----------+
```

### Notes
- Shows all conflicts that need resolution
- Conflict types: schema, data, metadata
- Resolved status indicates if conflict has been addressed

---

## Resolve Conflict

Resolve a specific merge conflict.

### Syntax
```bash
pangolin-admin resolve-conflict \
  --merge-id <MERGE_ID> \
  --conflict-id <CONFLICT_ID> \
  --resolution <RESOLUTION_STRATEGY>
```

### Parameters
- `--merge-id` - UUID of the merge operation (required)
- `--conflict-id` - UUID of the conflict (required)
- `--resolution` - Resolution strategy (required)

### Resolution Strategies
- `use-source` - Use changes from source branch
- `use-target` - Keep target branch version
- `manual` - Manual resolution (requires additional steps)

### Examples
```bash
# Use source branch changes
pangolin-admin resolve-conflict \
  --merge-id "merge-uuid" \
  --conflict-id "conflict-uuid" \
  --resolution "use-source"

# Keep target branch version
pangolin-admin resolve-conflict \
  --merge-id "merge-uuid" \
  --conflict-id "conflict-uuid" \
  --resolution "use-target"
```

### Output
```
✅ Conflict resolved successfully!
```

### Notes
- Must resolve all conflicts before completing merge
- Resolution is permanent once applied
- Use `list-conflicts` to verify all are resolved

---

## Complete Merge

Complete a merge operation after all conflicts are resolved.

### Syntax
```bash
pangolin-admin complete-merge --id <MERGE_ID>
```

### Parameters
- `--id` - UUID of the merge operation (required)

### Example
```bash
pangolin-admin complete-merge --id "merge-uuid"
```

### Output
```
✅ Merge completed successfully!
```

### Notes
- All conflicts must be resolved first
- Finalizes the merge into target branch
- Cannot be undone after completion
- Target branch will contain merged changes

---

## Abort Merge

Abort a merge operation without applying changes.

### Syntax
```bash
pangolin-admin abort-merge --id <MERGE_ID>
```

### Parameters
- `--id` - UUID of the merge operation (required)

### Example
```bash
pangolin-admin abort-merge --id "merge-uuid"
```

### Output
```
✅ Merge aborted successfully!
```

### Notes
- Cancels the merge operation
- No changes applied to target branch
- Useful if conflicts are too complex
- Can start a new merge later

---

## Complete Merge Workflow

### 1. Initiate Merge
```bash
# Create merge operation (via merge-branch command)
pangolin-admin merge-branch \
  --source "feature-branch" \
  --target "main"
```

### 2. Check Status
```bash
# List merge operations
pangolin-admin list-merge-operations

# Get specific merge details
pangolin-admin get-merge-operation --id "merge-uuid"
```

### 3. Handle Conflicts
```bash
# List conflicts
pangolin-admin list-conflicts --merge-id "merge-uuid"

# Resolve each conflict
pangolin-admin resolve-conflict \
  --merge-id "merge-uuid" \
  --conflict-id "conflict-1" \
  --resolution "use-source"

pangolin-admin resolve-conflict \
  --merge-id "merge-uuid" \
  --conflict-id "conflict-2" \
  --resolution "use-target"
```

### 4. Complete or Abort
```bash
# If all conflicts resolved, complete
pangolin-admin complete-merge --id "merge-uuid"

# Or abort if needed
pangolin-admin abort-merge --id "merge-uuid"
```

---

## Advanced Workflows

### Automated Merge Script
```bash
#!/bin/bash
MERGE_ID="$1"

# Check for conflicts
CONFLICTS=$(pangolin-admin list-conflicts --merge-id "$MERGE_ID" | grep "No" | wc -l)

if [ "$CONFLICTS" -eq 0 ]; then
  # No unresolved conflicts, complete merge
  pangolin-admin complete-merge --id "$MERGE_ID"
else
  echo "⚠️  $CONFLICTS unresolved conflicts. Manual resolution required."
  pangolin-admin list-conflicts --merge-id "$MERGE_ID"
fi
```

### Conflict Resolution Strategy
```bash
#!/bin/bash
# Always use source for schema conflicts, target for data conflicts

MERGE_ID="$1"

# Get conflicts
pangolin-admin list-conflicts --merge-id "$MERGE_ID" > /tmp/conflicts.txt

# Parse and resolve
while read -r line; do
  CONFLICT_ID=$(echo "$line" | awk '{print $1}')
  CONFLICT_TYPE=$(echo "$line" | awk '{print $3}')
  
  if [ "$CONFLICT_TYPE" == "schema" ]; then
    pangolin-admin resolve-conflict \
      --merge-id "$MERGE_ID" \
      --conflict-id "$CONFLICT_ID" \
      --resolution "use-source"
  else
    pangolin-admin resolve-conflict \
      --merge-id "$MERGE_ID" \
      --conflict-id "$CONFLICT_ID" \
      --resolution "use-target"
  fi
done < /tmp/conflicts.txt
```

---

## Error Handling

### Common Errors

**Merge Not Found**:
```
Error: Failed to get merge operation (404): Merge operation not found
```
- Solution: Verify merge ID is correct

**Conflicts Not Resolved**:
```
Error: Failed to complete merge (400): Unresolved conflicts remain
```
- Solution: Resolve all conflicts first

**Invalid Resolution**:
```
Error: Failed to resolve conflict (400): Invalid resolution strategy
```
- Solution: Use valid strategy (use-source, use-target, manual)

---

## Best Practices

1. **Check Conflicts First**: Always list conflicts before resolving
2. **Resolve Systematically**: Resolve conflicts one by one
3. **Verify Resolution**: List conflicts again to confirm all resolved
4. **Test Before Complete**: Verify changes before completing merge
5. **Document Decisions**: Keep notes on resolution strategies
6. **Use Abort Wisely**: Don't hesitate to abort complex merges

---

## Related Commands

- `merge-branch` - Initiate a merge operation
- `create-branch` - Create a new branch
- `list-branches` - List all branches
- `list-commits` - View commit history

---

## Conflict Types

### Schema Conflicts
- Column additions/removals
- Data type changes
- Constraint modifications
- **Recommendation**: Usually use source (newer schema)

### Data Conflicts
- Overlapping data changes
- Concurrent updates
- **Recommendation**: Case-by-case basis

### Metadata Conflicts
- Property changes
- Tag modifications
- **Recommendation**: Usually use source

---

## Merge States

```
Initiated → Pending → In Progress → Completed/Aborted
                ↓
            Conflicts?
                ↓
            Resolved → Complete
```

---

## Tips

- **Small Merges**: Merge frequently to avoid large conflicts
- **Communication**: Coordinate with team before merging
- **Backup**: Consider backing up before complex merges
- **Testing**: Test merged changes in development first
- **Documentation**: Document merge decisions for audit trail
