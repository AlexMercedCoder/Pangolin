# Client Configuration

This guide shows how to configure Apache Iceberg clients (PyIceberg, PySpark, Trino, etc.) to connect to Pangolin.

## PyIceberg

### Installation

```bash
pip install "pyiceberg[s3fs,pyarrow]"
```

### Configuration

#### Python Code

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080/v1/main_warehouse",
        "s3.endpoint": "http://localhost:9000",
        "s3.access-key-id": "minioadmin",
        "s3.secret-access-key": "minioadmin",
        "s3.region": "us-east-1",
        "py-io-impl": "pyiceberg.io.pyarrow.PyArrowFileIO",
        "header.X-Pangolin-Tenant": "00000000-0000-0000-0000-000000000001",
    }
)

# List namespaces
namespaces = catalog.list_namespaces()
print(namespaces)

# Create a namespace
catalog.create_namespace("my_namespace")

# Create a table
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, IntegerType, StringType

schema = Schema(
    NestedField(1, "id", IntegerType(), required=True),
    NestedField(2, "data", StringType(), required=False),
)

table = catalog.create_table(
    identifier="my_namespace.my_table",
    schema=schema,
    location="s3://pangolin/data/my_namespace/my_table"
)
```

#### Configuration File (`~/.pyiceberg.yaml`)

```yaml
catalog:
  pangolin:
    uri: http://localhost:8080/v1/main_warehouse
    s3.endpoint: http://localhost:9000
    s3.access-key-id: minioadmin
    s3.secret-access-key: minioadmin
    s3.region: us-east-1
    py-io-impl: pyiceberg.io.pyarrow.PyArrowFileIO
    header.X-Pangolin-Tenant: 00000000-0000-0000-0000-000000000001
```

Then in Python:

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog("pangolin")
```

## PySpark

### Installation

```bash
pip install pyspark
```

### Configuration

```python
from pyspark.sql import SparkSession

spark = SparkSession.builder \\
    .appName("Pangolin Example") \\
    .config("spark.jars.packages", "org.apache.iceberg:iceberg-spark-runtime-3.5_2.12:1.5.0,org.apache.iceberg:iceberg-aws-bundle:1.5.0") \\
    .config("spark.sql.extensions", "org.apache.iceberg.spark.extensions.IcebergSparkSessionExtensions") \\
    .config("spark.sql.catalog.pangolin", "org.apache.iceberg.spark.SparkCatalog") \\
    .config("spark.sql.catalog.pangolin.catalog-impl", "org.apache.iceberg.rest.RESTCatalog") \\
    .config("spark.sql.catalog.pangolin.uri", "http://localhost:8080/v1/main_warehouse") \\
    .config("spark.sql.catalog.pangolin.io-impl", "org.apache.iceberg.aws.s3.S3FileIO") \\
    .config("spark.sql.catalog.pangolin.s3.endpoint", "http://localhost:9000") \\
    .config("spark.sql.catalog.pangolin.s3.access-key-id", "minioadmin") \\
    .config("spark.sql.catalog.pangolin.s3.secret-access-key", "minioadmin") \\
    .config("spark.sql.catalog.pangolin.s3.path-style-access", "true") \\
    .config("spark.sql.catalog.pangolin.header.X-Pangolin-Tenant", "00000000-0000-0000-0000-000000000001") \\
    .getOrCreate()

# Create a namespace
spark.sql("CREATE NAMESPACE IF NOT EXISTS pangolin.my_namespace")

# Create a table
spark.sql("""
    CREATE TABLE IF NOT EXISTS pangolin.my_namespace.my_table (
        id INT,
        data STRING
    )
    USING iceberg
    LOCATION 's3://pangolin/data/my_namespace/my_table'
""")

# Insert data
spark.sql("""
    INSERT INTO pangolin.my_namespace.my_table VALUES (1, 'hello'), (2, 'world')
""")

# Query data
df = spark.sql("SELECT * FROM pangolin.my_namespace.my_table")
df.show()
```

## Trino

### Configuration (`catalog/pangolin.properties`)

```properties
connector.name=iceberg
iceberg.catalog.type=rest
iceberg.rest.uri=http://localhost:8080/v1/main_warehouse
iceberg.rest.http-headers=X-Pangolin-Tenant:00000000-0000-0000-0000-000000000001
fs.s3a.endpoint=http://localhost:9000
fs.s3a.access-key=minioadmin
fs.s3a.secret-key=minioadmin
fs.s3a.path.style.access=true
```

## Dremio

### Configuration

1. Navigate to **Settings** → **Data Sources** → **Add Source**
2. Select **Iceberg Catalog (REST)**
3. Configure:
   - **Name**: `pangolin`
   - **REST Catalog URI**: `http://localhost:8080/v1/main_warehouse`
   - **HTTP Headers**: `X-Pangolin-Tenant: 00000000-0000-0000-0000-000000000001`
   - **S3 Endpoint**: `http://localhost:9000`
   - **S3 Access Key**: `minioadmin`
   - **S3 Secret Key**: `minioadmin`
   - **Path Style Access**: `true`

## Environment Variables

For production deployments, use environment variables instead of hardcoding credentials:

### PyIceberg

```python
import os

catalog = load_catalog(
    "pangolin",
    **{
        "uri": os.getenv("PANGOLIN_URI", "http://localhost:8080/v1/main_warehouse"),
        "s3.endpoint": os.getenv("S3_ENDPOINT", "http://localhost:9000"),
        "s3.access-key-id": os.getenv("AWS_ACCESS_KEY_ID"),
        "s3.secret-access-key": os.getenv("AWS_SECRET_ACCESS_KEY"),
        "s3.region": os.getenv("AWS_REGION", "us-east-1"),
        "header.X-Pangolin-Tenant": os.getenv("PANGOLIN_TENANT_ID"),
    }
)
```

### PySpark

```bash
export PANGOLIN_URI="http://localhost:8080/v1/main_warehouse"
export PANGOLIN_TENANT_ID="00000000-0000-0000-0000-000000000001"
export AWS_ACCESS_KEY_ID="minioadmin"
export AWS_SECRET_ACCESS_KEY="minioadmin"
export S3_ENDPOINT="http://localhost:9000"
```

Then reference in Spark config:

```python
.config("spark.sql.catalog.pangolin.uri", os.getenv("PANGOLIN_URI")) \\
.config("spark.sql.catalog.pangolin.header.X-Pangolin-Tenant", os.getenv("PANGOLIN_TENANT_ID")) \\
```

## Authentication

For production environments with authentication enabled:

### PyIceberg with JWT

```python
catalog = load_catalog(
    "pangolin",
    **{
        "uri": "https://pangolin.example.com/v1/main_warehouse",
        "header.Authorization": f"Bearer {jwt_token}",
        "header.X-Pangolin-Tenant": tenant_id,
        # ... other S3 config
    }
)
```

### PySpark with JWT

```python
.config("spark.sql.catalog.pangolin.header.Authorization", f"Bearer {jwt_token}") \\
.config("spark.sql.catalog.pangolin.header.X-Pangolin-Tenant", tenant_id) \\
```

## Troubleshooting

### Connection Issues

1. **404 Not Found**: Ensure the warehouse name in the URI matches an existing warehouse in Pangolin.
2. **403 Forbidden**: Check that the `X-Pangolin-Tenant` header is set correctly.
3. **S3 Access Denied**: Verify S3 credentials and endpoint configuration.

### Debugging

Enable debug logging:

**PyIceberg:**
```python
import logging
logging.basicConfig(level=logging.DEBUG)
```

**PySpark:**
```python
.config("spark.sql.debug.maxToStringFields", "100") \\
```
