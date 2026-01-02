# User Commands

The `user` command group focuses on day-to-day data engineering tasks, such as exploring data, generating connection code, and version control operations.

## Usage

```bash
python -m pypangolin.cli.main user [COMMAND] [OPTIONS]
```

## Authentication

### `login`
Authenticates with the Pangolin server and stores the token in your active profile.

**Options:**
- `--username TEXT`: (Required)
- `--password TEXT`: (Required)
- `--tenant-id TEXT`: Optional Tenant ID to scope the login.

**Example:**
```bash
pypangolin user login --username "myuser" --password "mypass"
```

## Data Exploration

### `list-catalogs`
Lists the catalogs available to the current user.

### `search`
Searches for assets (tables, views) across the platform.

**Arguments:**
- `QUERY`: The search string.

**Example:**
```bash
pypangolin user search "sales_data"
```

### `generate-code`
Generates boilerplate code to connect to a specific table in various languages.

**Options:**
- `--language` (Required): One of `pyiceberg`, `pyspark`, `dremio`, `sql`.
- `--table` (Required): Full table identifier (`catalog.namespace.table`).

**Example:**
```bash
pypangolin user generate-code --language pyiceberg --table "gold_db.marketing.campaigns"
```

## Version Control (Iceberg)

### `list-branches`
Lists branches for a given catalog.

**Arguments:**
- `CATALOG`: Name of the catalog.

### `create-branch`
Create a new branch from a source.

**Arguments:**
- `CATALOG`: Catalog name.
- `NAME`: New branch name.
- `--from TEXT`: Source branch name (optional).

### `merge-branch`
Merges changes from one branch to another.

**Options:**
- `--catalog TEXT`: (Required) Catalog name.
- `--source TEXT`: (Required) Source branch.
- `--target TEXT`: (Required) Target branch.

## Tags

### `list-tags`
Lists tags in a catalog.

### `create-tag`
Creates a tag at a specific commit.

**Arguments:**
- `CATALOG`: Catalog name.
- `NAME`: Tag name.
- `COMMIT-ID`: The commit hash to tag.
