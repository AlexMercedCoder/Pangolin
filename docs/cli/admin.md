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
- **Login**: `login --username <user> [--password <pass>] [--tenant-id <uuid>]`
  - Omit `--tenant-id` for Root user login
  - Include `--tenant-id` for tenant-scoped login (resolves duplicate usernames across tenants)
- **Logout**: Exit the REPL or simply don't use the session. Session tokens are stored in `~/.config/pangolin/cli/config.json`.

### Examples

**Root Login**:
```bash
pangolin-admin login --username admin --password password
```

**Tenant-Scoped Login** (for users with duplicate usernames):
```bash
# Get tenant ID first
TENANT_ID=$(pangolin-admin list-tenants | grep "my-tenant" | awk '{print $1}')

# Login with tenant context
pangolin-admin login --username user --password pass123 --tenant-id $TENANT_ID
```

## Core Management
### Tenants
- `create-tenant --name <name> --admin-username <user> --admin-password <pass>`: Create a new tenant.
- `list-tenants`: List all platform tenants (Root only).
- `update-tenant --id <id> [--name <name>]`: Update tenant properties.
- `delete-tenant <id>`: Remove a tenant and its data.

### Users
- `create-user <username> --email <email> --role <role>`: Create a new user.
- `list-users`: List all registered users.
- `update-user --id <id> [--username <name>] [--email <email>] [--active <bool>]`: Modify user profile.
- `delete-user <username>`: Delete a user account.

### Warehouses
- `create-warehouse --name <name> --type <s3|gcs|azure|local> ...`: Configure physical storage.
- `list-warehouses`: List defined warehouses.
- `update-warehouse --id <id> [--name <name>]`: Rename a warehouse.
- `delete-warehouse <name>`: Detach a storage location.

### Catalogs
- `create-catalog <name> --warehouse <warehouse_name>`: provision an Iceberg catalog.
- `list-catalogs`: View logical catalogs.
- `update-catalog --id <id> [--name <name>]`: Rename a catalog.
- `delete-catalog <name>`: Remove a catalog reference.

## Governance
### Permissions
- `list-permissions --role <role> --user <user>`: View active access policies.
- `grant-permission <username> <action> <resource>`: Grant user a specific permission.
- `revoke-permission <role> <action> <resource>`: Revoke permission from a role.

### Metadata
See [Metadata Management Guide](./admin-metadata.md) for detailed attribution and explorer commands.
- `get-metadata --entity-type <type> --entity-id <id>`: Get entity properties.
- `set-metadata --entity-type <type> --entity-id <id> <key> <value>`: Set entity properties.
- `list-namespace-tree <catalog>`: Browse catalog structure.

## Update Operations

Update existing resources. See [Update Operations Guide](./admin-update-operations.md) for details.

### Commands
- `update-tenant --id <id> --name <name>`: Update tenant properties
- `update-user --id <id> [--username <name>] [--email <email>] [--active <bool>]`: Update user properties
- `update-warehouse --id <id> --name <name>`: Update warehouse properties
- `update-catalog --id <id> --name <name>`: Update catalog properties

## Token Management

Manage authentication tokens for security. See [Token Management Guide](./admin-token-management.md) for details.

### Commands
- `revoke-token`: Revoke your own token (logout)
- `revoke-token-by-id --id <token-id>`: Admin revoke any token

## Merge Operations

Complete merge workflow for branch management. See [Merge Operations Guide](./admin-merge-operations.md) for details.

### Commands
- `list-merge-operations`: List all merge operations
- `get-merge-operation --id <id>`: Get merge details
- `list-conflicts --merge-id <id>`: List merge conflicts
- `resolve-conflict --merge-id <id> --conflict-id <id> --resolution <strategy>`: Resolve conflict
- `complete-merge --id <id>`: Complete a merge
- `abort-merge --id <id>`: Abort a merge

## Business Metadata & Governance

Manage business metadata and access requests. See [Business Metadata Guide](./admin-business-metadata.md) for details.

### Commands
- `delete-metadata --asset-id <id>`: Delete business metadata
- `request-access --asset-id <id> --reason <reason>`: Request asset access
- `list-access-requests`: List all access requests
- `update-access-request --id <id> --status <status>`: Approve/deny access request
- `get-asset-details --id <id>`: Get asset details

## Service User Management

Service users provide API key authentication for machine-to-machine access.

### Commands

- `create-service-user --name <name> [--description <desc>] [--role <role>] [--expires-in-days <days>]`: Create a new service user
  - Default role: `tenant-user`
  - Returns API key (shown only once!)
  
- `list-service-users`: List all service users with status

- `get-service-user --id <id>`: View detailed service user information

- `update-service-user --id <id> [--name <name>] [--description <desc>] [--active <true|false>]`: Update service user properties

- `delete-service-user --id <id>`: Delete a service user

- `rotate-service-user-key --id <id>`: Rotate API key (invalidates old key immediately)

### Examples

**Create service user**:
```bash
pangolin-admin create-service-user \
  --name "ci-pipeline" \
  --description "CI/CD automation" \
  --role "tenant-user" \
  --expires-in-days 90
```

**List service users**:
```bash
pangolin-admin list-service-users
```

**Rotate API key**:
```bash
pangolin-admin rotate-service-user-key --id <uuid>
```

### Important Notes

- ⚠️ API keys are shown only once during creation/rotation - save them securely!
- Valid roles: `tenant-user`, `tenant-admin`, `root`
- Service users authenticate via `X-API-Key` header
- Rotating a key immediately invalidates the old key
