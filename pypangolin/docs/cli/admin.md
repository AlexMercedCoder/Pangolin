# Admin Commands

The `admin` command group contains administrative operations for managing the Pangolin platform. These commands typically require a user with `Root` or `TenantAdmin` roles.

## Usage

```bash
python -m pypangolin.cli.main admin [COMMAND] [OPTIONS]
```

## Tenants

### `create-tenant`
Creates a new tenant.

**Arguments:**
- `--name TEXT`: (Required) The name of the tenant.

**Example:**
```bash
pypangolin admin create-tenant --name "my-analytics-tenant"
```

### `list-tenants`
Lists all available tenants.

**Example:**
```bash
pypangolin admin list-tenants
```

### `delete-tenant`
Deletes a tenant by ID.

**Arguments:**
- `TENANT_ID`: (Required) The UUID of the tenant to delete.

**Example:**
```bash
pypangolin admin delete-tenant 550e8400-e29b-41d4-a716-446655440000
```

## Users

### `create-user`
Creates a new user within a tenant or as a root user.

**Options:**
- `--username TEXT`: (Required) Username.
- `--password TEXT`: (Required) Password.
- `--email TEXT`: (Required) Email address.
- `--role TEXT`: Role to assign (`TenantUser`, `TenantAdmin`, or `Root`). Default: `TenantUser`.
- `--tenant-id TEXT`: ID of the tenant to assign the user to.

**Example:**
```bash
pypangolin admin create-user \
  --username "data_eng_1" \
  --password "strong_password" \
  --email "de@example.com" \
  --role "TenantUser" \
  --tenant-id "TENANT_UUID"
```

### `list-users`
Lists all users.

**Example:**
```bash
pypangolin admin list-users
```

## Warehouses

### `create-warehouse`
confgiures a storage warehouse (currently supports S3-compatible storage).

**Options:**
- `--name TEXT`: (Required) Warehouse name.
- `--type TEXT`: Storage type (default: `s3`).
- `--bucket TEXT`: S3 bucket name.
- `--region TEXT`: AWS Region (default: `us-east-1`).
- `--endpoint TEXT`: Custom S3 endpoint (e.g., for MinIO).
- `--access-key TEXT`: AWS Access Key ID.
- `--secret-key TEXT`: AWS Secret Access Key.

**Example:**
```bash
pypangolin admin create-warehouse \
  --name "data-lake" \
  --bucket "my-bucket" \
  --endpoint "http://localhost:9000" \
  --access-key "minioadmin" \
  --secret-key "minioadmin"
```

### `list-warehouses`
Lists all configured warehouses.

## Catalogs

### `create-catalog`
Creates a catalog attached to a warehouse.

**Options:**
- `--name TEXT`: (Required) Catalog name (namespace in the API).
- `--warehouse TEXT`: (Required) Name of the warehouse to attach.
- `--type TEXT`: Catalog type (`Local` or `Federated`). Default: `Local`.

**Example:**
```bash
pypangolin admin create-catalog --name "gold_db" --warehouse "data-lake"
```

### `list-catalogs`
Lists all catalogs.

## Governance

### `grant-permission`
Grants a permission to a user.

**Options:**
- `--username TEXT`: (Required) User to receive permission.
- `--action TEXT`: (Required) Action to allow (e.g., `Read`, `Write`, `Create`).
- `--resource TEXT`: (Required) Resource identifier.

**Example:**
```bash
pypangolin admin grant-permission --username "data_eng_1" --action "Read" --resource "catalog:gold_db"
```

### `list-audit-events`
Lists recent audit log events.

**Options:**
- `--limit INTEGER`: Number of events to show (default: 100).
