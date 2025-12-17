**Note**: This scenario uses SQLite for a simple file-based persistent setup.

**Features:**
- **Authentication**: Enabled. Default root credentials: `admin` / `password`.
- **SQLite Store**: Persistent metadata storage (in Docker volume).
- **MinIO**: Local S3-compatible storage.

## Usage

1. Start the stack:
   ```bash
   docker compose up -d
   ```

2. Access the UI: http://localhost:3000
   - Login with `admin` / `password`.
   - Create a Tenant.
   - Create Users/Warehouses/Catalogs within that Tenant.

3. Access the API: http://localhost:8080

4. Access MinIO Console: http://localhost:9001 (user: `minioadmin`, pass: `minioadmin`)

## PyIceberg Configuration

You need to obtain a token first via UI or API login.

```python
from pyiceberg.catalog import load_catalog

# 1. Get Token (or copy from UI after login)
# POST http://localhost:8080/auth/token with username/password/tenant_id

catalog = load_catalog(
    "local",
    **{
        "type": "rest",
        "uri": "http://localhost:8080/v1/catalogs/my_catalog/iceberg",
        "token": "<YOUR_ACCESS_TOKEN>",
        "header.X-Pangolin-Tenant": "<YOUR_TENANT_ID>",
        "s3.endpoint": "http://localhost:9000",
        "s3.access-key-id": "minioadmin",
        "s3.secret-access-key": "minioadmin",
        "s3.region": "us-east-1"
    }
)
```
