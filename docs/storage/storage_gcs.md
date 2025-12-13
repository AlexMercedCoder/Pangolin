# Google Cloud Storage Configuration

Pangolin supports Google Cloud Storage (GCS) as a storage backend for Iceberg tables.

## Prerequisites

- Google Cloud Platform account
- GCS bucket created
- Service account with Storage Admin permissions
- Service account JSON key file

## Warehouse Configuration

### Create GCS Warehouse

```bash
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "Content-Type: application/json" \
  -d '{
    "name": "gcs_warehouse",
    "use_sts": false,
    "storage_config": {
      "type": "gcs",
      "project_id": "my-gcp-project",
      "service_account_key": "{\"type\":\"service_account\",\"project_id\":\"my-project\",...}",
      "bucket": "iceberg-tables"
    }
  }'
```

### Storage Config Fields

| Field | Required | Description |
|-------|----------|-------------|
| `type` | Yes | Must be `"gcs"` |
| `project_id` | Yes | GCP project ID |
| `service_account_key` | Yes | Service account JSON key (as string) |
| `bucket` | Yes | GCS bucket name |
| `endpoint` | No | Custom endpoint (for GCS emulator) |

## Service Account Setup

### Create Service Account

```bash
# Create service account
gcloud iam service-accounts create iceberg-catalog \
  --display-name="Iceberg Catalog Service Account"

# Grant Storage Admin role
gcloud projects add-iam-policy-binding my-gcp-project \
  --member="serviceAccount:iceberg-catalog@my-gcp-project.iam.gserviceaccount.com" \
  --role="roles/storage.admin"

# Create and download key
gcloud iam service-accounts keys create key.json \
  --iam-account=iceberg-catalog@my-gcp-project.iam.gserviceaccount.com
```

### Use Key in Warehouse Config

```bash
# Read key file and escape for JSON
KEY_JSON=$(cat key.json | jq -c .)

curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"gcs_warehouse\",
    \"use_sts\": false,
    \"storage_config\": {
      \"type\": \"gcs\",
      \"project_id\": \"my-gcp-project\",
      \"service_account_key\": $KEY_JSON,
      \"bucket\": \"iceberg-tables\"
    }
  }"
```

## PyIceberg Integration

### With Credential Vending

```python
from pyiceberg.catalog import load_catalog

# Catalog will automatically receive GCS credentials from warehouse
catalog = load_catalog('pangolin', **{
    'uri': 'http://localhost:8080',
    'prefix': 'my_catalog',  # Catalog linked to GCS warehouse
})

# Create table - credentials vended automatically
table = catalog.create_table(
    'my_namespace.my_table',
    schema=schema
)

# Write data - uses vended GCS credentials
table.append(data)
```

### With Client-Provided Credentials

```python
from pyiceberg.catalog import load_catalog
import json

# Load service account key
with open('key.json') as f:
    sa_key = json.load(f)

catalog = load_catalog('pangolin', **{
    'uri': 'http://localhost:8080',
    'prefix': 'my_catalog',
    'gcs.project-id': 'my-gcp-project',
    'gcs.service-account-key': json.dumps(sa_key),
})
```

## Credential Vending

When a catalog is linked to a GCS warehouse, Pangolin automatically vends credentials in the correct format for PyIceberg:

**Vended Properties**:
- `gcs.project-id` - GCP project ID
- `gcs.service-account-key` - Service account JSON key
- `gcs.endpoint` - Custom endpoint (if configured)

**Storage Location Format**:
```
gs://<bucket>/
```

## OAuth2 Support (Future)

For OAuth2 token-based authentication, set `use_sts: true`:

```json
{
  "name": "gcs_oauth_warehouse",
  "use_sts": true,
  "storage_config": {
    "type": "gcs",
    "project_id": "my-gcp-project",
    "bucket": "iceberg-tables"
  }
}
```

> [!NOTE]
> OAuth2 token generation is not yet implemented. Currently returns placeholder tokens.

## Example: Complete Workflow

```bash
# 1. Create GCS warehouse
KEY_JSON=$(cat key.json | jq -c .)

curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"gcs_prod\",
    \"use_sts\": false,
    \"storage_config\": {
      \"type\": \"gcs\",
      \"project_id\": \"my-prod-project\",
      \"service_account_key\": $KEY_JSON,
      \"bucket\": \"iceberg-prod\"
    }
  }"

# 2. Create catalog linked to warehouse
curl -X POST http://localhost:8080/api/v1/catalogs \
  -H "Content-Type: application/json" \
  -d '{
    "name": "production",
    "warehouse_name": "gcs_prod",
    "storage_location": "gs://iceberg-prod/production/"
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
print(f"Read {len(df)} rows from GCS")
EOF
```

## GCS Emulator (Testing)

For local testing, use the GCS emulator:

```bash
# Start GCS emulator
docker run -p 4443:4443 fsouza/fake-gcs-server -scheme http

# Create warehouse with emulator endpoint
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "Content-Type: application/json" \
  -d '{
    "name": "gcs_local",
    "use_sts": false,
    "storage_config": {
      "type": "gcs",
      "project_id": "test-project",
      "service_account_key": "{\"type\":\"service_account\"}",
      "bucket": "test-bucket",
      "endpoint": "http://localhost:4443"
    }
  }'
```

## Troubleshooting

### Authentication Issues

**Error**: `Invalid service account key`

**Solution**: Verify JSON key is valid:
```bash
# Validate JSON
cat key.json | jq .

# Test authentication
gcloud auth activate-service-account \
  --key-file=key.json
```

### Permission Issues

**Error**: `Access denied`

**Solution**: Ensure service account has Storage Admin role:
```bash
gcloud projects get-iam-policy my-gcp-project \
  --flatten="bindings[].members" \
  --filter="bindings.members:serviceAccount:iceberg-catalog@*"
```

### PyIceberg Configuration

PyIceberg expects these property names:
- `gcs.project-id` (with hyphens)
- `gcs.service-account-key`
- `gcs.endpoint`

Pangolin automatically formats credentials correctly.

## See Also

- [Storage: S3](storage_s3.md)
- [Storage: Azure](storage_azure.md)
- [Warehouse Management](warehouse_management.md)
- [PyIceberg Testing](pyiceberg_testing.md)
