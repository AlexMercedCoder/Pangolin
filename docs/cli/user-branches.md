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
pangolin-user create-branch --catalog <catalog> <name> [--from <source_branch>] [--branch-type <type>] [--assets <asset_list>]
```

**Parameters**:
- `--catalog` (required): Catalog name
- `name` (required): Name of the new branch
- `--from` (optional): Source branch to branch from (default: `main`)
- `--branch-type` (optional): Branch type - `main`, `feature`, or `experiment`
- `--assets` (optional): Comma-separated list of assets for partial branching

**Examples**:

Basic branch creation:
```bash
# Create a feature branch from main
pangolin-user create-branch --catalog sales feature/q3_adjustments

# Create a branch from an existing branch
pangolin-user create-branch --catalog sales experimental --from dev
```

**Enhanced Branching** (with branch types):
```bash
# Create a feature branch with explicit type
pangolin-user create-branch --catalog sales feature-x \
  --branch-type feature

# Create an experiment branch
pangolin-user create-branch --catalog sales experiment-1 \
  --branch-type experiment
```

**Partial Branching** (specific assets only):
```bash
# Create a branch with only specific tables
pangolin-user create-branch --catalog sales feature-subset \
  --branch-type feature \
  --assets "customers,orders,products"

# Create an experiment branch with a single table
pangolin-user create-branch --catalog sales test-pricing \
  --branch-type experiment \
  --assets "pricing_model"
```

**Use Cases**:

1. **Feature Development**: Create feature branches to develop new data transformations
2. **Experimentation**: Create experiment branches to test data changes without affecting production
3. **Partial Branching**: Create branches with only the tables you need to work with, reducing overhead
4. **Data Isolation**: Isolate changes to specific datasets while keeping others unchanged


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
