# Admin Warehouse Management

Warehouses define the physical storage locations (S3, GCS, Azure, Local) where data is kept.

## Commands

### List Warehouses
View configured warehouses and their storage types.

**Syntax**:
```bash
pangolin-admin list-warehouses
```

### Create Warehouse
Configure a new storage location.

> [!IMPORTANT]
> This command requires **Tenant Admin** privileges. The Root User cannot create warehouses.

**Syntax (Full)**:
```bash
pangolin-admin create-warehouse --name <name> --type <type> --bucket <bucket> --access-key <key> --secret-key <secret> --region <region> [--endpoint <url>]
```

**Supported Types**: `s3`, `gcs`, `azure`, `local`.

**Example**:
```bash
pangolin-admin create-warehouse --name production --type s3 --bucket "my-data" --region "us-east-1" ...
```

### Update Warehouse
Modify a warehouse configuration.

**Syntax**:
```bash
pangolin-admin update-warehouse --id <uuid> [--name <new_name>]
```

**Example**:
```bash
pangolin-admin update-warehouse --id "warehouse-uuid" --name "legacy-storage"
```

### Delete Warehouse
Remove a warehouse configuration. Does not delete the actual data in storage.

**Syntax**:
```bash
pangolin-admin delete-warehouse <name>
```

**Example**:
```bash
pangolin-admin delete-warehouse old_bucket
```
