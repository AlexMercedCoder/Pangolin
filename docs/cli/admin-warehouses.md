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
Configure a new storage location. The CLI may prompt for additional configuration depending on the type.

**Syntax**:
```bash
pangolin-admin create-warehouse --name <name> --type <type>
```

**Supported Types**:
- `s3`: Amazon S3 or MinIO
- `gcs`: Google Cloud Storage
- `azure`: Azure Blob Storage
- `local`: Local filesystem

**Example**:
```bash
pangolin-admin create-warehouse --name main_lake --type s3
# Prompts for Bucket, Region, Access Key, Secret Key will follow...
```

### Delete Warehouse
Remove a warehouse configuration. Does not delete the actual data in storage, only the reference.

**Syntax**:
```bash
pangolin-admin delete-warehouse <name>
```

**Example**:
```bash
pangolin-admin delete-warehouse old_bucket
```
