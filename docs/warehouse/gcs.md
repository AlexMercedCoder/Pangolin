# Google Cloud Storage Warehouse Configuration

Configure Google Cloud Storage (GCS) as a warehouse for storing Iceberg table data in Pangolin.

## Overview

Google Cloud Storage provides:
- Multi-regional and dual-regional storage
- Strong consistency
- Integration with Google Cloud ecosystem
- Automatic encryption at rest
- Cost-effective storage classes

## Prerequisites

- GCS bucket created
- Service account with appropriate permissions
- Service account key JSON file

## Warehouse Configuration

### Option 1: With Service Account Key

```bash
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "X-Pangolin-Tenant: my-tenant" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "gcs-prod",
    "storage_type": "gcs",
    "bucket": "my-iceberg-tables",
    "project_id": "my-gcp-project",
    "use_sts": false,
    "credentials": {
      "service_account_key": "{\"type\":\"service_account\",\"project_id\":\"my-project\",...}"
    }
  }'
```

### Option 2: With Workload Identity (GKE - Recommended)

```bash
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "X-Pangolin-Tenant: my-tenant" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "gcs-prod",
    "storage_type": "gcs",
    "bucket": "my-iceberg-tables",
    "project_id": "my-gcp-project",
    "use_sts": true,
    "workload_identity": "pangolin-sa@my-project.iam.gserviceaccount.com"
  }'
```

## Client Configuration

### Scenario 1: Catalog WITH Warehouse (Recommended)

Pangolin vends credentials automatically.

#### PyIceberg

```python
from pyiceberg.catalog import load_catalog

# Pangolin vends GCS credentials automatically
catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080/api/v1/catalogs/analytics",
        "warehouse": "gs://my-iceberg-tables/analytics/"
    }
)

# Create table - Pangolin handles GCS access
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, IntegerType

schema = Schema(
    NestedField(1, "id", IntegerType(), required=True),
    NestedField(2, "name", StringType(), required=True)
)

table = catalog.create_table("db.users", schema=schema)

# Write data
import pyarrow as pa
data = pa.table({"id": [1, 2, 3], "name": ["Alice", "Bob", "Charlie"]})
table.append(data)

# Read data
df = table.scan().to_pandas()
print(df)
```

#### PySpark

```python
from pyspark.sql import SparkSession

# Pangolin vends credentials automatically
spark = SparkSession.builder \
    .appName("Pangolin Iceberg GCS") \
    .config("spark.sql.catalog.pangolin", "org.apache.iceberg.spark.SparkCatalog") \
    .config("spark.sql.catalog.pangolin.catalog-impl", "org.apache.iceberg.rest.RESTCatalog") \
    .config("spark.sql.catalog.pangolin.uri", "http://localhost:8080/api/v1/catalogs/analytics") \
    .config("spark.sql.catalog.pangolin.warehouse", "gs://my-iceberg-tables/analytics/") \
    .getOrCreate()

# Create table
spark.sql("""
    CREATE TABLE pangolin.db.users (
        id INT,
        name STRING
    ) USING iceberg
""")

# Write data
data = [(1, "Alice"), (2, "Bob"), (3, "Charlie")]
df = spark.createDataFrame(data, ["id", "name"])
df.writeTo("pangolin.db.users").append()

# Read data
spark.table("pangolin.db.users").show()
```

### Scenario 2: Catalog WITHOUT Warehouse

Clients configure GCS access themselves.

#### PyIceberg

```python
from pyiceberg.catalog import load_catalog
import os

# Set GCS credentials
os.environ["GOOGLE_APPLICATION_CREDENTIALS"] = "/path/to/service-account-key.json"

catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080/api/v1/catalogs/analytics",
        "warehouse": "gs://my-iceberg-tables/analytics/",
        # Client-side GCS configuration
        "gcs.project-id": "my-gcp-project",
        "gcs.service-account-file": "/path/to/service-account-key.json"
    }
)

# Use catalog normally
table = catalog.load_table("db.users")
df = table.scan().to_pandas()
```

#### PySpark

```python
from pyspark.sql import SparkSession

spark = SparkSession.builder \
    .appName("Pangolin Iceberg GCS") \
    .config("spark.sql.catalog.pangolin", "org.apache.iceberg.spark.SparkCatalog") \
    .config("spark.sql.catalog.pangolin.catalog-impl", "org.apache.iceberg.rest.RESTCatalog") \
    .config("spark.sql.catalog.pangolin.uri", "http://localhost:8080/api/v1/catalogs/analytics") \
    .config("spark.sql.catalog.pangolin.warehouse", "gs://my-iceberg-tables/analytics/") \
    .config("spark.hadoop.google.cloud.auth.service.account.enable", "true") \
    .config("spark.hadoop.google.cloud.auth.service.account.json.keyfile", "/path/to/service-account-key.json") \
    .getOrCreate()

# Use Spark normally
spark.table("pangolin.db.users").show()
```

## GCS Best Practices

### Performance
1. **Use Regional buckets** for best performance
2. **Use Multi-regional** for global access
3. **Enable Turbo Replication** for faster replication
4. **Collocate compute and storage** in same region

### Security
1. **Use Workload Identity** (GKE) instead of service account keys
2. **Enable Uniform Bucket-Level Access**
3. **Use Customer-Managed Encryption Keys** (CMEK) if required
4. **Enable Audit Logging**
5. **Use IAM Conditions** for fine-grained access

### Cost Optimization
1. **Use Lifecycle Policies** to transition to Nearline/Coldline/Archive
2. **Enable Object Versioning** selectively
3. **Use Committed Use Discounts**
4. **Monitor with Cloud Billing**

## IAM Permissions

Service account needs these permissions:

```json
{
  "roles": [
    "roles/storage.objectAdmin"
  ],
  "resources": [
    "projects/my-project/buckets/my-iceberg-tables"
  ]
}
```

Or custom role with specific permissions:
- `storage.objects.create`
- `storage.objects.delete`
- `storage.objects.get`
- `storage.objects.list`
- `storage.buckets.get`

## Troubleshooting

### Access Denied

```
Error: 403 Forbidden
```

**Solutions**:
1. Check service account has correct permissions
2. Verify bucket name is correct
3. Check project ID
4. Ensure service account key is valid

### Connection Issues

**Solutions**:
1. Verify bucket exists
2. Check network connectivity
3. Ensure firewall rules allow access
4. Check service account key format

## Additional Resources

- [Google Cloud Storage Documentation](https://cloud.google.com/storage/docs)
- [GCS Best Practices](https://cloud.google.com/storage/docs/best-practices)
- [Iceberg GCS Configuration](https://iceberg.apache.org/docs/latest/gcp/)

## Next Steps

- [S3 Warehouse](s3.md)
- [Azure Warehouse](azure.md)
- [Warehouse Concept](README.md)
