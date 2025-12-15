# User Branch Management

Pangolin supports Git-like branching for your data. You can create branches to experiment, isolate changes, or manage data lifecycles.

## Commands

### List Branches
View all branches in a catalog.

**Syntax**:
```bash
pangolin-user list-branches --catalog <catalog_name>
```

**Example**:
```bash
pangolin-user list-branches --catalog sales
```

### Create Branch
Create a new branch from an existing one (defaulting to `main`).

**Syntax**:
```bash
pangolin-user create-branch --catalog <catalog> --name <new_branch> [--from <source_branch>]
```

**Example**:
```bash
# Create a feature branch from main
pangolin-user create-branch --catalog sales --name feature/q3_adjustments

# Create a branch from an existing branch
pangolin-user create-branch --catalog sales --name experimental --from dev
```

### Merge Branch
Merge changes from one branch into another (e.g., merging a feature branch back into main).

**Syntax**:
```bash
pangolin-user merge-branch --catalog <catalog> --source <src> --target <tgt>
```

**Example**:
```bash
pangolin-user merge-branch --catalog sales --source feature/q3_adjustments --target main
```
