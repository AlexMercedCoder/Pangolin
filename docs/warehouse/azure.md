# Azure Blob Storage Warehouse Configuration

Configure Azure Blob Storage (ADLS Gen2) as a warehouse for storing Iceberg table data in Pangolin.

## Overview

Azure Blob Storage with ADLS Gen2 provides:
- Hierarchical namespace for better performance
- Integration with Azure ecosystem
- Enterprise-grade security
- Cost-effective storage tiers
- Global availability

## Prerequisites

- Azure Storage Account with ADLS Gen2 enabled
- Storage account name and access key or SAS token
- Container created for Iceberg tables

## Warehouse Configuration

### AzureSas - Storage Account Key

```bash
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "X-Pangolin-Tenant: my-tenant" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "azure-prod",
    "storage_config": {
      "account_name": "mystorageaccount",
      "container": "iceberg-tables"
    },
    "vending_strategy": {
      "type": "AzureSas",
      "account_name": "mystorageaccount",
      "account_key": "your-account-key-here=="
    }
  }'
```

### None - Client-Provided Credentials

For scenarios where clients provide their own Azure credentials:

```bash
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "X-Pangolin-Tenant: my-tenant" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "azure-client-creds",
    "storage_config": {
      "account_name": "mystorageaccount",
      "container": "iceberg-tables"
    },
    "vending_strategy": {
      "type": "None"
    }
  }'
```

**Note**: Managed Identity support via Azure AD is not yet implemented in the VendingStrategy enum. Use AzureSas for credential vending or None for client-provided credentials.

## Client Configuration

### Scenario 1: Catalog WITH Warehouse (Recommended)

Pangolin vends credentials automatically.

#### PyIceberg

```python
from pyiceberg.catalog import load_catalog

# Pangolin vends Azure credentials automatically
catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080/api/v1/catalogs/analytics",
        "warehouse": "abfss://iceberg-tables@mystorageaccount.dfs.core.windows.net/analytics/"
    }
)

# Create table - Pangolin handles Azure access
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
    .appName("Pangolin Iceberg Azure") \
    .config("spark.sql.catalog.pangolin", "org.apache.iceberg.spark.SparkCatalog") \
    .config("spark.sql.catalog.pangolin.catalog-impl", "org.apache.iceberg.rest.RESTCatalog") \
    .config("spark.sql.catalog.pangolin.uri", "http://localhost:8080/api/v1/catalogs/analytics") \
    .config("spark.sql.catalog.pangolin.warehouse", "abfss://iceberg-tables@mystorageaccount.dfs.core.windows.net/analytics/") \
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

Clients configure Azure access themselves.

#### PyIceberg

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080/api/v1/catalogs/analytics",
        "warehouse": "abfss://iceberg-tables@mystorageaccount.dfs.core.windows.net/analytics/",
        # Client-side Azure configuration
        "adls.account-name": "mystorageaccount",
        "adls.account-key": "your-account-key-here==",
        "adls.connection-string": "DefaultEndpointsProtocol=https;AccountName=mystorageaccount;AccountKey=your-key;EndpointSuffix=core.windows.net"
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
    .appName("Pangolin Iceberg Azure") \
    .config("spark.sql.catalog.pangolin", "org.apache.iceberg.spark.SparkCatalog") \
    .config("spark.sql.catalog.pangolin.catalog-impl", "org.apache.iceberg.rest.RESTCatalog") \
    .config("spark.sql.catalog.pangolin.uri", "http://localhost:8080/api/v1/catalogs/analytics") \
    .config("spark.sql.catalog.pangolin.warehouse", "abfss://iceberg-tables@mystorageaccount.dfs.core.windows.net/analytics/") \
    .config("spark.hadoop.fs.azure.account.key.mystorageaccount.dfs.core.windows.net", "your-account-key-here==") \
    .getOrCreate()

# Use Spark normally
spark.table("pangolin.db.users").show()
```

## Azure Best Practices

### Performance
1. **Use Premium Block Blobs** for high-performance workloads
2. **Enable Hierarchical Namespace** (ADLS Gen2) for better performance
3. **Use Azure CDN** for global distribution
4. **Collocate compute and storage** in same region

### Security
1. **Use Managed Identities** instead of storage keys
2. **Enable Storage Encryption** (enabled by default)
3. **Use Private Endpoints** for network isolation
4. **Enable Storage Analytics Logging**
5. **Use Azure RBAC** for fine-grained access control

### Cost Optimization
1. **Use Lifecycle Management** to tier data to Cool/Archive
2. **Enable Soft Delete** for data protection
3. **Use Azure Storage Reserved Capacity** for discounts
4. **Monitor with Azure Cost Management**

## Troubleshooting

### Access Denied

```
Error: This request is not authorized to perform this operation
```

**Solutions**:
1. Check storage account key is correct
2. Verify container exists
3. Check managed identity permissions
4. Ensure storage account allows public access (if needed)

### Connection Issues

**Solutions**:
1. Verify storage account name
2. Check endpoint URL format
3. Ensure firewall rules allow access
4. Check network connectivity

## Additional Resources

- [Azure Blob Storage Documentation](https://docs.microsoft.com/en-us/azure/storage/blobs/)
- [ADLS Gen2 Documentation](https://docs.microsoft.com/en-us/azure/storage/blobs/data-lake-storage-introduction)
- [Iceberg Azure Configuration](https://iceberg.apache.org/docs/latest/azure/)

## Next Steps

- [S3 Warehouse](s3.md)
- [GCS Warehouse](gcs.md)
- [Warehouse Concept](README.md)
