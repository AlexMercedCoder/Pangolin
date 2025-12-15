# Pangolin Admin CLI (`pangolin-admin`)

The `pangolin-admin` tool is designed for system administrators to manage Tenants, Users, Warehouses, Catalogs, and Governance policies.

## Usage Modes

### Interactive Mode (REPL)
Run the binary without a command to enter the interactive shell.
```bash
pangolin-admin
# Optional: Specify server URL and Profile
pangolin-admin --url http://localhost:8080 --profile prod
```
In this mode, the session is persistent. You can login once and run multiple commands.

### Non-Interactive Mode
Run a single command directly from the shell. Useful for scripts.
```bash
pangolin-admin login --username admin
pangolin-admin list-tenants
pangolin-admin --profile prod create-user --username newuser
```

## Authentication
- **Login**: `login --username <user>`
- **Logout**: Exit the REPL or simply don't use the session. Session tokens are stored in `~/.config/pangolin/cli/config.json`.

## Core Management
### Tenants
- `create-tenant --name <name>`: Create a new tenant.
- `list-tenants`: List all tenants.
- `delete-tenant <id>`: Delete a tenant.

### Users
- `create-user --username <user> --role <role>`: Create a new user. Role defaults to `user`.
- `list-users`: List all registered users.
- `delete-user <username>`: Delete a user.

### Warehouses
- `create-warehouse --name <name> --type <s3|gcs|azure>`: Create a storage warehouse.
- `list-warehouses`: List defined warehouses.
- `delete-warehouse <name>`: Delete a warehouse.

### Catalogs
- `create-catalog --name <name> --warehouse <warehouse_name>`: Create a catalog linked to a warehouse.
- `list-catalogs`: List all catalogs.
- `delete-catalog <name>`: Delete a catalog.

## Governance
### Permissions
- `list-permissions --role <role> --user <user>`: View active permissions.
- `grant-permission <role> <action> <resource>`: Grant a permission.
- `revoke-permission <role> <action> <resource>`: Revoke a permission.

### Metadata
- `get-metadata --entity-type <type> --entity-id <id>`: detailed JSON metadata.
- `set-metadata --entity-type <type> --entity-id <id> <key> <value>`: Attach key-value metadata.
