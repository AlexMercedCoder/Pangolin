# Evaluate Single Tenant

This scenario is designed for quick evaluation without authentication complexity.

**Features:**
- **No Authentication**: `PANGOLIN_NO_AUTH=true`
- **Memory Store**: Metadata is lost on restart.
- **MinIO**: Local S3-compatible storage.
- **Jupyter Notebook**: Pre-configured environment for testing with PyIceberg.

## Usage

1. Start the stack:
   ```bash
   docker compose up -d
   ```

2. Access the services:
   - **Pangolin UI**: http://localhost:3000
   - **Pangolin API**: http://localhost:8080
   - **Jupyter Notebook**: http://localhost:8888
   - **MinIO Console**: http://localhost:9001 (user: `minioadmin`, pass: `minioadmin`)

## Testing with Jupyter Notebook

The included Jupyter notebook server has PyIceberg pre-installed and can access Pangolin and MinIO via Docker DNS.

1. Open Jupyter at http://localhost:8888

2. Create a new Python notebook and test the connection:

```python
from pyiceberg.catalog import load_catalog

# Connect to Pangolin (no auth required in this mode)
catalog = load_catalog(
    "pangolin",
    **{
        "type": "rest",
        "uri": "http://pangolin-api:8080/v1/my_catalog",
        "header.X-Iceberg-Access-Delegation": "vended-credentials",
    }
)

# List namespaces
print(catalog.list_namespaces())

# Create a namespace
catalog.create_namespace("demo")

# Create a table
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, IntegerType

schema = Schema(
    NestedField(1, "id", IntegerType(), required=True),
    NestedField(2, "name", StringType(), required=False),
)

table = catalog.create_table(
    "demo.users",
    schema=schema,
)

print(f"Created table: {table}")
```

## PyIceberg Configuration (External)

If you want to connect from your host machine instead of the Jupyter notebook:

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "pangolin",
    **{
        "type": "rest",
        "uri": "http://localhost:8080/v1/my_catalog",
        "header.X-Iceberg-Access-Delegation": "vended-credentials",
        # For external access, you may need to provide S3 credentials
        # since vended credentials might not work from outside Docker network
        "s3.endpoint": "http://localhost:9000",
        "s3.access-key-id": "minioadmin",
        "s3.secret-access-key": "minioadmin",
        "s3.region": "us-east-1",
        "s3.path-style-access": "true",
    }
)
```

## Notes

- The default tenant ID in NO_AUTH mode is `00000000-0000-0000-0000-000000000000`
- Metadata is stored in memory and will be lost when the container restarts
- MinIO data persists in a Docker volume
