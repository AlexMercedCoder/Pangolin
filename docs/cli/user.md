# Pangolin User CLI (`pangolin-user`)

The `pangolin-user` tool is for data engineers and analysts to discover data, manage branches, and request access.

## Usage Modes

### Interactive Mode (REPL)
```bash
pangolin-user
pangolin-user --profile dev
```

### Non-Interactive Mode
```bash
pangolin-user list-catalogs
pangolin-user search "sales_data"
```

## Data Discovery
- `list-catalogs`: View available catalogs.
- `search <query>`: Search for tables and views.
- `generate-code --language <pyiceberg|pyspark|dremio|sql> --table <table_name>`: Generate connection snippets.
- `get-token [--description <text>] [--expires-in <days>]`: Generate personal API key.

## Data Engineering
### Branching
- `list-branches --catalog <catalog>`: View branch history.
- `create-branch --catalog <catalog> <name> [--from <branch>] [--branch-type <type>]`: Create new fork.
- `merge-branch --catalog <catalog> --source <src> --target <tgt>`: Promote/Integrate changes.

### Tags
- `list-tags --catalog <catalog>`: View immutable release/snapshot tags.
- `create-tag --catalog <catalog> --name <name> --commit-id <id>`: Tag a specific version.

## Access Management
- `request-access --resource <res> --role <role> --reason <text>`: Request permission to an asset.
- `list-requests`: View status of your pending/historical requests.
