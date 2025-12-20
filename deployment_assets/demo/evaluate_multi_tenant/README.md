# Evaluate Multi-Tenant

This scenario demonstrates Pangolin's multi-tenant capabilities with authentication and persistent storage.

**Features:**
- **Authentication**: Enabled. Default root credentials: `admin` / `password`.
- **SQLite Store**: Persistent metadata storage (in Docker volume).
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

3. Initial Setup via UI:
   - Login with `admin` / `password`
   - Create a Tenant
   - Create a Tenant Admin user
   - Create Warehouses and Catalogs within that Tenant

## Getting a Token

You need a JWT token to authenticate with Pangolin. You can get one via:

### Option 1: Via UI
1. Login to http://localhost:3000
2. Navigate to your profile
3. Generate a new token
4. Copy the token value

### Option 2: Via API
```bash
curl -X POST http://localhost:8080/api/v1/users/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "password"
  }'
```

## Testing with Jupyter Notebook

The included Jupyter notebook server has PyIceberg pre-installed and can access Pangolin and MinIO via Docker DNS.

1. Open Jupyter at http://localhost:8888

2. Get your token (see above)

3. Create a new Python notebook and test the connection:

```python
from pyiceberg.catalog import load_catalog

# Replace with your actual token
TOKEN = "YOUR_JWT_TOKEN_HERE"

# Connect to Pangolin with authentication
catalog = load_catalog(
    "pangolin",
    **{
        "type": "rest",
        "uri": "http://pangolin-api:8080/v1/my_catalog",
        "token": TOKEN,
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
        "token": "YOUR_JWT_TOKEN_HERE",
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

- Metadata persists in SQLite database stored in Docker volume
- MinIO data persists in a Docker volume
- Use the UI to manage tenants, users, and permissions
- Tokens can be revoked and rotated via the UI
