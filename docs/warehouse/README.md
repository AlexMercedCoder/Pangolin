# Warehouse Storage

Warehouses in Pangolin define where your **actual data files** are stored (Parquet, Avro, ORC files), separate from the catalog metadata.

## Warehouse vs Backend Storage

It's important to understand the distinction:

| Component | Purpose | Storage | Examples |
|-----------|---------|---------|----------|
| **Backend Storage** | Catalog metadata | PostgreSQL, MongoDB, SQLite | Table schemas, partitions, snapshots |
| **Warehouse Storage** | Actual data files | S3, Azure Blob, GCS | Parquet files, metadata.json |

```
┌─────────────────────────────────────┐
│   Pangolin Catalog (Backend)       │
│   - Table schemas                   │
│   - Partition info                  │
│   - Snapshot metadata               │
│   Stored in: PostgreSQL/Mongo/SQLite│
└──────────────┬──────────────────────┘
               │ Points to
               ▼
┌─────────────────────────────────────┐
│   Warehouse (Object Storage)        │
│   - Parquet data files              │
│   - Iceberg metadata files          │
│   - Manifest files                  │
│   Stored in: S3/Azure/GCS           │
└─────────────────────────────────────┘
```

## Warehouse Concept

A **warehouse** in Pangolin is a named configuration that specifies:
- **Storage type**: S3, Azure Blob Storage, or Google Cloud Storage
- **Location**: Bucket/container and path prefix
- **Credentials**: How to authenticate (static credentials or STS/IAM roles)
- **Region**: Geographic location of storage

> [!TIP]
> **Flat Key Support**: As of v0.1.0, `storage_config` supports flat keys (e.g., `"s3.bucket": "mybucket"`) as an alternative to nested objects. This is often easier to pass via CLI or environment-driven scripts.

### Example Warehouse (Flat Keys)

```json
{
  "name": "production-s3",
  "storage_config": {
    "s3.bucket": "my-company-datalake",
    "s3.region": "us-east-1"
  },
  "vending_strategy": {
    "type": "AwsSts",
    "role_arn": "arn:aws:iam::123456789:role/PangolinDataAccess"
  }
}
```

## Warehouse Patterns

### Pattern 1: Warehouse Attached to Catalog

The catalog configuration includes a warehouse reference:

```json
{
  "name": "analytics",
  "type": "local",
  "warehouse": "production-s3",
  "properties": {}
}
```

**Benefits**:
- Centralized credential management
- Consistent storage configuration
- Automatic credential vending to clients
- Easier to manage and audit

**Client Configuration**: Minimal - Pangolin vends credentials automatically

### Pattern 2: Catalog Without Warehouse

The catalog has no warehouse attached:

```json
{
  "name": "analytics",
  "type": "local",
  "warehouse": null,
  "properties": {}
}
```

**Benefits**:
- Clients control their own storage access
- Flexible for multi-cloud scenarios
- Useful when clients have their own credentials

**Client Configuration**: Clients must configure storage themselves

## Authentication Methods

### VendingStrategy Enum

Pangolin uses the `vending_strategy` field to configure credential vending. The `use_sts` field is deprecated but kept for backward compatibility.

**Available Strategies**:

#### 1. AwsSts - AWS STS Temporary Credentials (Recommended)

```json
{
  "name": "prod-s3",
  "storage_config": {
    "bucket": "my-datalake",
    "region": "us-east-1"
  },
  "vending_strategy": {
    "type": "AwsSts",
    "role_arn": "arn:aws:iam::123456789:role/PangolinDataAccess",
    "external_id": null
  }
}
```

**Pros**: Most secure, credentials expire, fine-grained permissions  
**Cons**: Requires IAM role setup

#### 2. AwsStatic - AWS Static Credentials

```json
{
  "name": "dev-s3",
  "storage_config": {
    "bucket": "my-dev-datalake",
    "region": "us-east-1"
  },
  "vending_strategy": {
    "type": "AwsStatic",
    "access_key_id": "AKIAIOSFODNN7EXAMPLE",
    "secret_access_key": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
  }
}
```

**Pros**: Simple, works everywhere  
**Cons**: Less secure, credentials don't expire

#### 3. AzureSas - Azure SAS Tokens

```json
{
  "name": "prod-azure",
  "storage_config": {
    "account_name": "mystorageaccount",
    "container": "datalake"
  },
  "vending_strategy": {
    "type": "AzureSas",
    "account_name": "mystorageaccount",
    "account_key": "your-account-key"
  }
}
```

#### 4. GcpDownscoped - GCP Service Account

```json
{
  "name": "prod-gcs",
  "storage_config": {
    "bucket": "my-datalake",
    "project_id": "my-project"
  },
  "vending_strategy": {
    "type": "GcpDownscoped",
    "service_account_email": "iceberg@my-project.iam.gserviceaccount.com",
    "private_key": "-----BEGIN PRIVATE KEY-----\n..."
  }
}
```

#### 5. None - No Credential Vending

```json
{
  "name": "client-provided",
  "storage_config": {
    "bucket": "my-datalake"
  },
  "vending_strategy": {
    "type": "None"
  }
}
```

Clients must provide their own credentials.

### Deprecated: use_sts Field

The `use_sts` boolean field is deprecated. Use `vending_strategy` instead.

**Old Format** (Deprecated):
```json
{
  "use_sts": true,
  "role_arn": "arn:aws:iam::123:role/Access"
}
```

**New Format** (Current):
```json
{
  "vending_strategy": {
    "type": "AwsSts",
    "role_arn": "arn:aws:iam::123:role/Access",
    "external_id": null
  }
}
```

## Supported Storage Types

| Storage | Status | Best For |
|---------|--------|----------|
| [AWS S3](s3.md) | ✅ Production | Most common, excellent performance |
| [Azure Blob](azure.md) | ✅ Production | Azure-native deployments |
| [Google Cloud Storage](gcs.md) | ✅ Production | GCP-native deployments |
| [Local Filesystem](local.md) | ⚠️ Dev/Test | Local development & testing |

## Quick Start

### 1. Create a Warehouse

```bash
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "X-Pangolin-Tenant: my-tenant" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "production-s3",
    "storage_config": {
      "bucket": "my-datalake",
      "region": "us-east-1"
    },
    "vending_strategy": {
      "type": "AwsSts",
      "role_arn": "arn:aws:iam::123456789:role/DataAccess",
      "external_id": null
    }
  }'
```

### 2. Create a Catalog with Warehouse

```bash
curl -X POST http://localhost:8080/api/v1/catalogs \
  -H "X-Pangolin-Tenant: my-tenant" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "analytics",
    "type": "local",
    "warehouse": "production-s3"
  }'
```

### 3. Use from PyIceberg

```python
from pyiceberg.catalog import load_catalog

# Pangolin vends credentials automatically
catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080/api/v1/catalogs/analytics",
        "warehouse": "s3://my-datalake/analytics/"
    }
)

# Create table - Pangolin handles storage access
catalog.create_table(
    "db.table",
    schema=schema
)
```

## Client Configuration

### With Warehouse (Recommended)

When a catalog has a warehouse attached, Pangolin automatically vends credentials to clients via the `X-Iceberg-Access-Delegation` header.

**PyIceberg**: No storage configuration needed
**PySpark**: No storage configuration needed

### Without Warehouse

When a catalog has no warehouse, clients must configure storage themselves.

**PyIceberg**: Configure S3/Azure/GCS credentials
**PySpark**: Configure Hadoop filesystem properties

See individual storage guides for details:
- [S3 Client Configuration](s3.md)
- [Azure Client Configuration](azure.md)
- [GCS Client Configuration](gcs.md)

## Best Practices

### Security
1. **Use STS/IAM Roles**: Prefer temporary credentials over static keys
2. **Least Privilege**: Grant minimum required permissions
3. **Separate Warehouses**: Use different warehouses for dev/staging/prod
4. **Audit Access**: Enable CloudTrail/Azure Monitor/GCS audit logs

### Performance
1. **Regional Colocation**: Place warehouse in same region as compute
2. **Bucket Naming**: Use descriptive, hierarchical names
3. **Lifecycle Policies**: Archive old data to cheaper storage tiers
4. **Compression**: Use Snappy or Zstd for Parquet files

### Organization
1. **Naming Convention**: `{environment}-{region}-{purpose}`
   - Examples: `prod-us-east-1-analytics`, `dev-eu-west-1-ml`
2. **Path Structure**: `s3://bucket/{catalog}/{namespace}/{table}/`
3. **Multi-Tenant**: Use separate buckets or prefixes per tenant

## Troubleshooting

### Permission Denied

```
Error: Access Denied to s3://my-bucket/path/
```

**Solutions**:
1. Check IAM role permissions
2. Verify bucket policy
3. Check STS assume role permissions
4. Verify warehouse configuration

### Credential Vending Not Working

```
Error: No credentials provided
```

**Solutions**:
1. Ensure catalog has warehouse attached
2. Check warehouse `use_sts` setting
3. Verify IAM role ARN
4. Check Pangolin server has permission to assume role

### Slow Performance

**Solutions**:
1. Check region - ensure compute and storage are colocated
2. Enable S3 Transfer Acceleration
3. Use larger instance types for compute
4. Check network bandwidth

## Next Steps

- [S3 Warehouse Configuration](s3.md)
- [Azure Blob Warehouse Configuration](azure.md)
- [GCS Warehouse Configuration](gcs.md)
- [Backend Storage Options](../backend_storage/README.md)
