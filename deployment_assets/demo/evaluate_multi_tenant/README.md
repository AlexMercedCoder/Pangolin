# Evaluate Multi-Tenant

This scenario demonstrates Pangolin's multi-tenant capabilities with authentication and persistent storage.

**Features:**
- **Authentication**: Enabled. Default root credentials: `admin` / `password`.
- **SQLite Store**: Persistent metadata storage (in Docker volume).
- **MinIO**: Local S3-compatible storage.
- **Jupyter Notebook**: Pre-configured environment for testing with PyIceberg.

## Usage

### 1. Start the Stack
```bash
docker compose up -d
```
[View Docker Compose File](./docker-compose.yml)

### 2. Create Tenant & Admin

You need a tenant and a tenant administrator to manage resources.

#### 🖥️ UI
1.  **Login** as `admin` / `password` (Root).
2.  Go to **Tenants** → **Create Tenant**.
3.  Name: `Acme`.
4.  **Check** "Create initial admin user".
5.  Admin Username: `acme_admin`, Password: `Password123`.
6.  Click **Create**.
7.  **Copy the Tenant ID** from the list.
8.  **Logout**.

#### ⌨️ CLI
```bash
# Login as Root
export PANGOLIN_TOKEN=$(pangolin-admin login --username admin --password password --format json | jq -r .token)

# Create Tenant
TENANT_ID=$(pangolin-admin create-tenant --name Acme --format json | jq -r .id)

# Create Tenant Admin
pangolin-admin create-user \
  --username acme_admin \
  --password Password123 \
  --role tenant-admin \
  --tenant-id $TENANT_ID
```

#### 🐍 Python SDK
```python
from pypangolin import PangolinClient

# Login as Root
client = PangolinClient("http://localhost:8080")
client.login("admin", "password")

# Create Tenant
tenant = client.tenants.create(name="Acme")

# Create Admin
client.users.create(
    username="acme_admin",
    password="Password123",
    role="tenant-admin",
    tenant_id=tenant.id
)
print(f"Tenant ID: {tenant.id}")
```

#### 🌐 API (cURL)
```bash
# Login
TOKEN=$(curl -s -X POST http://localhost:8080/api/v1/users/login \
  -d '{"username":"admin", "password":"password"}' \
  -H "Content-Type: application/json" | jq -r .token)

# Create Tenant
TENANT_ID=$(curl -s -X POST http://localhost:8080/api/v1/tenants \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name": "Acme"}' | jq -r .id)

# Create Admin
curl -X POST http://localhost:8080/api/v1/users \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"username\": \"acme_admin\",
    \"password\": \"Password123\",
    \"role\": \"tenant-admin\",
    \"tenant_id\": \"$TENANT_ID\"
  }"
```

---

### 3. Create Warehouse & Catalog

Now login as the new Tenant Admin to configure storage.

#### 🖥️ UI
1.  **Login** as Tenant Admin (`acme_admin` / `Password123`).
    *   **Tenant ID**: (Paste ID from Step 2)
2.  **Warehouses** → **Create Warehouse**:
    *   Name: `acme_wh`
    *   Provider: `AWS S3`, Bucket: `warehouse`
    *   Endpoint: `http://minio:9000`, Region: `us-east-1`
    *   Access/Secret Key: `minioadmin`
    *   Vending Strategy: `AWS Static`
    *   Path Style: `true`
3.  **Catalogs** → **Create Catalog**:
    *   Name: `acme_cat`
    *   Warehouse: `acme_wh`
    *   Location: `s3://warehouse/acme`

#### ⌨️ CLI
```bash
# Login as Tenant Admin
export PANGOLIN_TOKEN=$(pangolin-admin login \
  --username acme_admin \
  --password Password123 \
  --tenant-id $TENANT_ID \
  --format json | jq -r .token)

# Create Warehouse
pangolin-admin create-warehouse \
  --name acme_wh \
  --bucket warehouse \
  --region us-east-1 \
  --endpoint http://minio:9000 \
  --access-key minioadmin \
  --secret-key minioadmin \
  --path-style true

# Create Catalog
pangolin-admin create-catalog \
  --name acme_cat \
  --warehouse acme_wh \
  --location s3://warehouse/acme
```

#### 🐍 Python SDK
```python
# Login as Tenant Admin
client.login("acme_admin", "Password123", tenant_id="<TENANT_ID>")

# Create Warehouse
client.warehouses.create_s3(
    name="acme_wh",
    bucket="warehouse",
    region="us-east-1",
    endpoint="http://minio:9000",
    access_key="minioadmin",
    secret_key="minioadmin",
    path_style_access=True
)

# Create Catalog
client.catalogs.create(
    name="acme_cat",
    warehouse="acme_wh",
    storage_location="s3://warehouse/acme"
)
```

#### 🌐 API (cURL)
```bash
# Login
TOKEN=$(curl -s -X POST http://localhost:8080/api/v1/users/login \
  -d "{\"username\":\"acme_admin\", \"password\":\"Password123\", \"tenant_id\":\"$TENANT_ID\"}" \
  -H "Content-Type: application/json" | jq -r .token)

# Create Warehouse
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "acme_wh",
    "vending_strategy": {"AwsStatic": {"access_key_id": "minioadmin", "secret_access_key": "minioadmin"}},
    "storage_config": {
      "type": "s3", "bucket": "warehouse", "region": "us-east-1", "endpoint": "http://minio:9000",
      "access_key_id": "minioadmin", "secret_access_key": "minioadmin", "s3.path-style-access": "true"
    }
  }'

# Create Catalog
curl -X POST http://localhost:8080/api/v1/catalogs \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name": "acme_cat", "warehouse_name": "acme_wh", "storage_location": "s3://warehouse/acme"}'
```

---

### 4. Run PyIceberg Test (Jupyter)

1.  Open **Jupyter Notebook**: http://localhost:8888
2.  Use the token from Step 3 (or generate one in UI Profile).
3.  Run the test script:

```python
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, IntegerType
import pyarrow as pa

TOKEN = "<YOUR_TOKEN>"

# Connect via REST
catalog = load_catalog(
    "pangolin",
    **{
        "type": "rest",
        "uri": "http://pangolin-api:8080/v1/acme_cat",
        "token": TOKEN,
        "header.X-Iceberg-Access-Delegation": "vended-credentials",
    }
)

# Operations
catalog.create_namespace("acme_ns")
schema = Schema(NestedField(1, "id", IntegerType(), True), NestedField(2, "data", StringType(), False))
table = catalog.create_table("acme_ns.test_table", schema=schema)

# Write Data
df = pa.Table.from_pylist([{"id": 1, "data": "Hello World"}], schema=schema.as_arrow())
table.append(df)
print(table.scan().to_arrow().to_pydict())
```
