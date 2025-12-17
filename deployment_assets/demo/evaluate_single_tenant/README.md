# Evaluate Single Tenant

This scenario is designed for quick evaluation without authentication complexity.

**Features:**
- **No Authentication**: `PANGOLIN_NO_AUTH=true`
- **Memory Store**: Metadata is lost on restart.
- **MinIO**: Local S3-compatible storage.

## Usage

1. Start the stack:
   ```bash
   docker compose up -d
   ```

2. Access the UI: http://localhost:3000

3. Access the API: http://localhost:8080

4. Access MinIO Console: http://localhost:9001 (user: `minioadmin`, pass: `minioadmin`)

## PyIceberg Configuration

Since auth is disabled, you can use any token or an empty string, but you should specify the default tenant ID `00000000-0000-0000-0000-000000000000`.

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "local",
    **{
        "type": "rest",
        "uri": "http://localhost:8080/v1/catalogs/my_catalog/iceberg",
        "header.X-Pangolin-Tenant": "00000000-0000-0000-0000-000000000000",
        "s3.endpoint": "http://localhost:9000",
        "s3.access-key-id": "minioadmin",
        "s3.secret-access-key": "minioadmin",
        "s3.region": "us-east-1"
    }
)
```
