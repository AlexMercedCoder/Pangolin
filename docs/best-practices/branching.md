# Branch Management Best Practices

## Overview

This guide covers best practices for using Pangolin's Git-like branching features for safe data experimentation and development.

## Branching Strategy

### Branch Types

**Main/Production Branch**
- Default branch for production data
- Protected from direct writes
- All changes via merge from development branches

**Development Branches**
- Short-lived feature branches
- Experimentation and testing
- Merged back to main after validation

**Long-Running Branches**
- Staging/QA environments
- Regional variants
- Historical snapshots

### Naming Conventions

```bash
# Feature branches
feature/customer-segmentation
feature/new-metrics-2025

# Environment branches
staging
qa
production

# User branches
dev/alice/experiment
dev/bob/analysis

# Release branches
release/2025-q4
release/v2.0
```

## Branch Lifecycle

### 1. Create Branch

```bash
# Create from main
pangolin-user create-branch \
  --catalog analytics \
  --name dev/alice/customer-analysis \
  --from main

# Create from specific snapshot
pangolin-user create-branch \
  --catalog analytics \
  --name experiment/pricing-model \
  --from main \
  --snapshot-id abc123
```

**Best Practices**
- Always branch from known good state
- Use descriptive names
- Document branch purpose

### 2. Development

```python
# Work on branch
from pyiceberg.catalog import load_catalog

catalog = load_catalog("pangolin", ...)

# Specify branch in table reference
table = catalog.load_table("analytics.sales@dev/alice/customer-analysis")

# Make changes
table.append(new_data)
table.update(...)
```

**Isolation Benefits**
- Experiment without affecting production
- Test schema changes safely
- Validate data transformations

### 3. Testing & Validation

```python
# Compare branch to main
main_table = catalog.load_table("analytics.sales@main")
branch_table = catalog.load_table("analytics.sales@dev/alice/customer-analysis")

# Validate row counts
assert branch_table.scan().to_arrow().num_rows > main_table.scan().to_arrow().num_rows

# Validate schema compatibility
assert branch_table.schema() == main_table.schema()

# Run data quality checks
run_data_quality_tests(branch_table)
```

### 4. Merge

```bash
# Merge back to main
pangolin-user merge-branch \
  --catalog analytics \
  --source dev/alice/customer-analysis \
  --target main \
  --strategy fast-forward

# Or create merge commit
pangolin-user merge-branch \
  --catalog analytics \
  --source dev/alice/customer-analysis \
  --target main \
  --strategy merge-commit \
  --message "Add customer segmentation analysis"
```

### 5. Cleanup

```bash
# Delete merged branch
pangolin-admin delete-branch \
  --catalog analytics \
  --name dev/alice/customer-analysis

# Or keep for historical reference
pangolin-user create-tag \
  --catalog analytics \
  --name customer-analysis-v1 \
  --branch dev/alice/customer-analysis
```

## Merge Strategies

### Fast-Forward Merge

**When to Use**
- Linear history desired
- No conflicts expected
- Branch is ahead of target

```bash
# Fast-forward merge
pangolin-user merge-branch \
  --source feature/new-metrics \
  --target main \
  --strategy fast-forward
```

**Result**
```
Before:
main:    A---B
              \
feature:       C---D

After:
main:    A---B---C---D
```

### 3-Way Merge

**When to Use**
- Parallel development
- Both branches have changes
- Need to preserve history

```bash
# 3-way merge
pangolin-user merge-branch \
  --source feature/schema-update \
  --target main \
  --strategy merge-commit
```

**Result**
```
Before:
main:    A---B---E
              \
feature:       C---D

After:
main:    A---B---E---M
              \     /
feature:       C---D
```

## Conflict Resolution

### Detecting Conflicts

```bash
# Attempt merge
pangolin-user merge-branch \
  --source feature/update \
  --target main

# If conflicts detected
Error: Merge conflicts detected
- Table: sales.orders
  - Conflict: Schema mismatch
  - Base: schema v1
  - Source: schema v2
  - Target: schema v1.1
```

### Resolution Strategies

**1. Manual Resolution**
```bash
# List conflicts
pangolin-admin list-conflicts --merge-id <uuid>

# Resolve each conflict
pangolin-admin resolve-conflict \
  --merge-id <uuid> \
  --conflict-id <uuid> \
  --resolution use-source  # or use-target, or manual

# Complete merge
pangolin-admin complete-merge --merge-id <uuid>
```

**2. Abort and Retry**
```bash
# Abort merge
pangolin-admin abort-merge --merge-id <uuid>

# Rebase branch on target
# Resolve conflicts in branch
# Retry merge
```

## Common Patterns

### Feature Development

```bash
# 1. Create feature branch
pangolin-user create-branch --name feature/add-customer-ltv --from main

# 2. Develop and test
# ... make changes ...

# 3. Validate
run_tests()

# 4. Merge to main
pangolin-user merge-branch --source feature/add-customer-ltv --target main

# 5. Clean up
pangolin-admin delete-branch --name feature/add-customer-ltv
```

### Environment Promotion

```bash
# Dev → Staging → Production

# 1. Develop in dev branch
pangolin-user create-branch --name dev --from main

# 2. Promote to staging
pangolin-user merge-branch --source dev --target staging

# 3. Test in staging
run_integration_tests()

# 4. Promote to production
pangolin-user merge-branch --source staging --target main
```

### A/B Testing

```bash
# Create variant branches
pangolin-user create-branch --name variant-a --from main
pangolin-user create-branch --name variant-b --from main

# Apply different transformations
apply_transformation_a(variant-a)
apply_transformation_b(variant-b)

# Run experiments
results_a = run_experiment(variant-a)
results_b = run_experiment(variant-b)

# Merge winning variant
if results_a > results_b:
    pangolin-user merge-branch --source variant-a --target main
else:
    pangolin-user merge-branch --source variant-b --target main
```

### Time Travel & Rollback

```bash
# Create branch from historical snapshot
pangolin-user create-branch \
  --name rollback/before-incident \
  --from main \
  --snapshot-id <snapshot-before-issue>

# Validate data is correct
validate_data(rollback/before-incident)

# Merge to restore
pangolin-user merge-branch \
  --source rollback/before-incident \
  --target main \
  --message "Rollback to state before data quality issue"
```

## Branch Protection

### Protected Branches

**Main/Production**
```yaml
# Branch protection rules
branches:
  main:
    protected: true
    require_merge: true
    allowed_mergers:
      - tenant-admin
      - data-engineer
    require_tests: true
    require_approval: true
```

**Enforcement**
```bash
# Prevent direct writes to main
❌ table = catalog.load_table("analytics.sales@main")
❌ table.append(data)  # Error: Direct writes to protected branch not allowed

# Require merge workflow
✅ Create branch → Make changes → Merge to main
```

## Performance Considerations

### Branch Overhead

**Metadata Only**
- Branches are lightweight (metadata pointers)
- No data duplication until writes
- Fast branch creation

**Copy-on-Write**
```
main:     [snapshot-1] → [snapshot-2]
           ↓
branch:   [snapshot-1] → [snapshot-3] (only new data)
```

### Cleanup Strategy

**Delete Stale Branches**
```bash
# Find branches older than 30 days with no activity
pangolin-admin list-branches --catalog analytics --inactive-days 30

# Delete after review
pangolin-admin delete-branch --name old-feature-branch
```

**Archive Instead of Delete**
```bash
# Tag before deleting
pangolin-user create-tag --name archive/old-feature --branch old-feature-branch
pangolin-admin delete-branch --name old-feature-branch
```

## Monitoring

### Branch Metrics

```sql
-- Track branch count
SELECT catalog, COUNT(*) as branch_count
FROM branches
GROUP BY catalog;

-- Find long-running branches
SELECT name, created_at, DATEDIFF(NOW(), created_at) as age_days
FROM branches
WHERE age_days > 30
ORDER BY age_days DESC;
```

### Merge Activity

```sql
-- Merge frequency
SELECT 
    DATE_TRUNC('week', timestamp) as week,
    COUNT(*) as merge_count
FROM audit_logs
WHERE action = 'MERGE_BRANCH'
GROUP BY week
ORDER BY week DESC;
```

## Troubleshooting

### Merge Conflicts

**Common Causes**
- Schema evolution in both branches
- Concurrent data modifications
- Incompatible transformations

**Resolution**
1. Understand both changes
2. Choose appropriate resolution strategy
3. Test merged result
4. Document decision

### Branch Divergence

**Symptoms**
- Large number of conflicts
- Merge takes long time
- Unexpected results

**Prevention**
- Merge main into branch regularly
- Keep branches short-lived
- Communicate schema changes

## Checklist

### Before Creating Branch
- [ ] Clear purpose defined
- [ ] Appropriate base branch selected
- [ ] Naming convention followed
- [ ] Team notified if needed

### During Development
- [ ] Regular commits/snapshots
- [ ] Tests passing
- [ ] Documentation updated
- [ ] Sync with main periodically

### Before Merging
- [ ] All tests passing
- [ ] Data quality validated
- [ ] Schema compatibility verified
- [ ] Conflicts resolved
- [ ] Approval obtained (if required)

### After Merging
- [ ] Branch deleted or archived
- [ ] Tag created if needed
- [ ] Team notified
- [ ] Documentation updated

## Additional Resources

- [Branch Management Guide](../features/branch_management.md)
- [Merge Operations](../features/merge_operations.md)
- [Conflict Resolution](../features/merge_conflicts.md)
- [Git Operations (PyPangolin)](../../pypangolin/docs/git_operations.md)
