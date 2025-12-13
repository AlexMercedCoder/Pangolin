# Azure Blob Storage Configuration

Pangolin supports Azure Blob Storage (ADLS Gen2) as a storage backend for Iceberg tables.

## Prerequisites

- Azure Storage Account with ADLS Gen2 enabled
- Storage account name and access key
- Container created for Iceberg tables

## Warehouse Configuration

### Create Azure Warehouse

```bash
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "Content-Type: application/json" \
  -d '{
    "name": "azure_warehouse",
    "use_sts": false,
    "storage_config": {
      "type": "azure",
      "account_name": "mystorageaccount",
      "account_key": "your-account-key-here",
      "container": "iceberg-tables",
      "endpoint": "https://mystorageaccount.dfs.core.windows.net"
    }
  }'
```

### Storage Config Fields

| Field | Required | Description |
|-------|----------|-------------|
| `type` | Yes | Must be `"azure"` |
| `account_name` | Yes | Azure storage account name |
| `account_key` | Yes | Azure storage account access key |
| `container` | Yes | Azure blob container name |
| `endpoint` | No | Custom endpoint (defaults to `<account>.dfs.core.windows.net`) |

## PyIceberg Integration

### With Credential Vending

```python
from pyiceberg.catalog import load_catalog

# Catalog will automatically receive Azure credentials from warehouse
catalog = load_catalog('pangolin', **{
    'uri': 'http://localhost:8080',
    'prefix': 'my_catalog',  # Catalog linked to Azure warehouse
})

# Create table - credentials vended automatically
table = catalog.create_table(
    'my_namespace.my_table',
    schema=schema
)

# Write data - uses vended Azure credentials
table.append(data)
```

### With Client-Provided Credentials

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog('pangolin', **{
    'uri': 'http://localhost:8080',
    'prefix': 'my_catalog',
    'adls.account-name': 'mystorageaccount',
    'adls.account-key': 'your-account-key-here',
})
```

## Credential Vending

When a catalog is linked to an Azure warehouse, Pangolin automatically vends credentials in the correct format for PyIceberg:

**Vended Properties**:
- `adls.account-name` - Azure storage account name
- `adls.account-key` - Azure storage account access key
- `adls.endpoint` - Azure ADLS endpoint (if configured)

**Storage Location Format**:
```
abfss://<container>@<account>.dfs.core.windows.net/
```

## OAuth2 Support (Future)

For OAuth2/Azure AD authentication, set `use_sts: true`:

```json
{
  "name": "azure_oauth_warehouse",
  "use_sts": true,
  "storage_config": {
    "type": "azure",
    "tenant_id": "your-tenant-id",
    "client_id": "your-client-id",
    "client_secret": "your-client-secret",
    "container": "iceberg-tables"
  }
}
```

> [!NOTE]
> OAuth2 token generation is not yet implemented. Currently returns placeholder tokens.

## Example: Complete Workflow

```bash
# 1. Create Azure warehouse
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "Content-Type: application/json" \
  -d '{
    "name": "azure_prod",
    "use_sts": false,
    "storage_config": {
      "type": "azure",
      "account_name": "prodaccount",
      "account_key": "prod-key",
      "container": "iceberg-prod"
    }
  }'

# 2. Create catalog linked to warehouse
curl -X POST http://localhost:8080/api/v1/catalogs \
  -H "Content-Type: application/json" \
  -d '{
    "name": "production",
    "warehouse_name": "azure_prod",
    "storage_location": "abfss://iceberg-prod@prodaccount.dfs.core.windows.net/production/"
  }'

# 3. Use with PyIceberg
python3 << EOF
from pyiceberg.catalog import load_catalog

catalog = load_catalog('pangolin', **{
    'uri': 'http://localhost:8080',
    'prefix': 'production',
})

# Credentials automatically vended!
table = catalog.load_table('my_namespace.my_table')
df = table.scan().to_pandas()
print(f"Read {len(df)} rows from Azure")
EOF
```

## Troubleshooting

### Connection Issues

**Error**: `Unable to connect to Azure storage`

**Solution**: Verify account name and key are correct:
```bash
# Test Azure credentials
az storage container list \
  --account-name mystorageaccount \
  --account-key your-account-key
```

### Permission Issues

**Error**: `Access denied`

**Solution**: Ensure the account key has proper permissions on the container.

### PyIceberg Configuration

PyIceberg expects these property names:
- `adls.account-name` (with hyphens)
- `adls.account-key`
- `adls.endpoint`

Pangolin automatically formats credentials correctly.

## See Also

- [Storage: S3](storage_s3.md)
- [Storage: GCS](storage_gcs.md)
- [Warehouse Management](warehouse_management.md)
- [PyIceberg Testing](pyiceberg_testing.md)
