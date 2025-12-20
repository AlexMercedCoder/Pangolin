# Client Configuration

This guide shows how to configure Apache Iceberg clients (PyIceberg, PySpark, Trino, etc.) to connect to Pangolin.

## Authentication Modes

Pangolin supports two authentication modes:

### 1. NO_AUTH Mode (Testing/Evaluation)
- Set `PANGOLIN_NO_AUTH=1` environment variable on the server
- No authentication required from clients
- Uses default tenant automatically
- **Not recommended for production**

### 2. Bearer Token Authentication (Production)
- JWT tokens with tenant information
- Iceberg REST specification compliant
- Required for multi-tenant deployments

## PyIceberg

### Installation

```bash
pip install "pyiceberg[s3fs,pyarrow]" pyjwt
```

### NO_AUTH Mode (Testing)

When the Pangolin server is running with `PANGOLIN_NO_AUTH=1`:

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080",
        "prefix": "analytics",  # Catalog name
        "s3.endpoint": "http://localhost:9000",
        "s3.access-key-id": "minioadmin",
        "s3.secret-access-key": "minioadmin",
        "s3.region": "us-east-1",
    }
)

# List namespaces
namespaces = catalog.list_namespaces()
print(namespaces)
```

### Bearer Token Authentication (Production)

#### Step 1: Generate Token

```bash
curl -X POST http://localhost:8080/api/v1/tokens \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_id": "00000000-0000-0000-0000-000000000001",
    "username": "user@example.com",
    "expires_in_hours": 24
  }'
```

Response:
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "expires_at": "2025-12-14T02:37:49+00:00",
  "tenant_id": "00000000-0000-0000-0000-000000000001"
}
```

#### Step 2: Use Token with PyIceberg

```python
from pyiceberg.catalog import load_catalog

# Token from /api/v1/tokens endpoint
token = "eyJ0eXAiOiJKV1QiLCJhbGc..."

catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080",
        "prefix": "analytics",  # Catalog name
        "token": token,         # Bearer token authentication
        # S3 configuration is Optional here! 
        # If the warehouse has Vending enabled, Pangolin will provide credentials.
    }
)
```

> [!TIP]
> **Credential Vending**: If your Pangolin warehouse is configured with a `vending_strategy` (like `AwsStatic` or `AwsSts`), you **do not** need to provide `s3.access-key-id` or `s3.secret-access-key` in your client configuration. Pangolin will automatically vend temporary credentials to the client.

#### Step 3: Generate Token Programmatically

```python
import jwt
import datetime
from pyiceberg.catalog import load_catalog

def generate_token(tenant_id: str, secret: str = "secret") -> str:
    """Generate JWT token with tenant_id"""
    payload = {
        "sub": "api-user",
        "tenant_id": tenant_id,
        "roles": ["User"],
        "exp": datetime.datetime.utcnow() + datetime.timedelta(hours=24)
    }
    return jwt.encode(payload, secret, algorithm="HS256")

# Generate token
token = generate_token("00000000-0000-0000-0000-000000000001")

# Use with PyIceberg
catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080",
        "prefix": "analytics",
        "token": token,
    }
)
```

### Configuration File (`~/.pyiceberg.yaml`)

#### NO_AUTH Mode
```yaml
catalog:
  pangolin:
    uri: http://localhost:8080
    prefix: analytics
    s3.endpoint: http://localhost:9000
    s3.access-key-id: minioadmin
    s3.secret-access-key: minioadmin
    s3.region: us-east-1
```

#### With Bearer Token
```yaml
catalog:
  pangolin:
    uri: http://localhost:8080
    prefix: analytics
    token: eyJ0eXAiOiJKV1QiLCJhbGc...  # From /api/v1/tokens
    s3.endpoint: http://localhost:9000
    s3.access-key-id: minioadmin
    s3.secret-access-key: minioadmin
    s3.region: us-east-1
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
    .config("spark.sql.catalog.pangolin.uri", "http://localhost:8080/v1/analytics") \\
    .config("spark.sql.catalog.pangolin.io-impl", "org.apache.iceberg.aws.s3.S3FileIO") \\
    .config("spark.sql.catalog.pangolin.s3.endpoint", "http://localhost:9000") \\
    .config("spark.sql.catalog.pangolin.s3.access-key-id", "minioadmin") \\
    .config("spark.sql.catalog.pangolin.s3.secret-access-key", "minioadmin") \\
    .config("spark.sql.catalog.pangolin.s3.path-style-access", "true") \\
    .config("spark.sql.catalog.pangolin.header.Authorization", f"Bearer {token}") \\  # Bearer token
    .getOrCreate()
```

## Trino

### Configuration (`catalog/pangolin.properties`)

```properties
connector.name=iceberg
iceberg.catalog.type=rest
iceberg.rest.uri=http://localhost:8080/v1/analytics
iceberg.rest.http-headers=Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGc...
fs.s3a.endpoint=http://localhost:9000
fs.s3a.access-key=minioadmin
fs.s3a.secret-key=minioadmin
fs.s3a.path.style.access=true
```

## Environment Variables

For production deployments, use environment variables:

### PyIceberg

```python
import os
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "pangolin",
    **{
        "uri": os.getenv("PANGOLIN_URI", "http://localhost:8080"),
        "prefix": os.getenv("PANGOLIN_CATALOG", "analytics"),
        "token": os.getenv("PANGOLIN_TOKEN"),  # JWT token
        "s3.endpoint": os.getenv("S3_ENDPOINT"),
        "s3.access-key-id": os.getenv("AWS_ACCESS_KEY_ID"),
        "s3.secret-access-key": os.getenv("AWS_SECRET_ACCESS_KEY"),
        "s3.region": os.getenv("AWS_REGION", "us-east-1"),
    }
)
```

## Troubleshooting

### Connection Issues

1. **401 Unauthorized**: 
   - In NO_AUTH mode: Ensure server has `PANGOLIN_NO_AUTH=1` set
   - In production: Check that Bearer token is valid and not expired
   
2. **404 Not Found**: Ensure the catalog name in the prefix/URI matches an existing catalog

3. **S3 Access Denied**: Verify S3 credentials and endpoint configuration

### Debugging

Enable debug logging:

**PyIceberg:**
```python
import logging
logging.basicConfig(level=logging.DEBUG)
```

**Check Token:**
```python
import jwt
decoded = jwt.decode(token, options={"verify_signature": False})
print(f"Tenant ID: {decoded.get('tenant_id')}")
print(f"Expires: {decoded.get('exp')}")
```

## Migration from Custom Headers

If you were using `header.X-Pangolin-Tenant`:

**Old (Not compatible with PyIceberg):**
```python
"header.X-Pangolin-Tenant": "00000000-0000-0000-0000-000000000001"
```

**New (Iceberg REST spec compliant):**
```python
"token": generate_token("00000000-0000-0000-0000-000000000001")
```

The custom header approach still works for direct API calls but is not supported by PyIceberg due to its authentication architecture.
