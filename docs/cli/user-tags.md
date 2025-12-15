# User Tag Management

Tags are immutable references to specific commits in the catalog's history. They are useful for marking releases, snapshots, or important points in time.

## Commands

### List Tags
View all tags in a catalog.

**Syntax**:
```bash
pangolin-user list-tags --catalog <catalog>
```

**Example**:
```bash
pangolin-user list-tags --catalog sales
```

### Create Tag
Create a tag pointing to a specific commit.

**Syntax**:
```bash
pangolin-user create-tag --catalog <catalog> --name <tag_name> --commit-id <commit_sha>
```

**Example**:
```bash
pangolin-user create-tag --catalog sales --name v1.0.0 --commit-id 8a08734268e3...
```
