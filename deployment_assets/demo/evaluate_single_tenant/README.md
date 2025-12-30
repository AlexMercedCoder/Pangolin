# Evaluate Single Tenant

This scenario is designed for quick evaluation without authentication complexity.

**Features:**
- **No Authentication**: `PANGOLIN_NO_AUTH=true`
- **Memory Store**: Metadata is lost on restart.
- **MinIO**: Local S3-compatible storage.
- **Jupyter Notebook**: Pre-configured environment for testing with PyIceberg.

## Usage

### 1. Start the Stack
```bash
docker compose up -d
```
[View Docker Compose File](./docker-compose.yml)

### 2. Configure Resources

Since the memory store starts empty, you must create a Warehouse and Catalog.

#### 🖥️ UI
1.  Open **Pangolin UI**: http://localhost:3000
2.  **Warehouses** → **Create Warehouse**:
    *   Name: `demo_warehouse`
    *   Provider: `AWS S3`, Bucket: `warehouse`
    *   Endpoint: `http://minio:9000`, Region: `us-east-1`
    *   Access Key: `minioadmin`, Secret Key: `minioadmin`
    *   Vending Strategy: `AWS Static`
    *   Path Style: `true`
    *   Click **Create**.
3.  **Catalogs** → **Create Catalog**:
    *   Name: `demo`
    *   Warehouse: `demo_warehouse`
    *   Location: `s3://warehouse/demo_catalog`
    *   Click **Create**.

#### ⌨️ CLI
```bash
# In No-Auth, use any mock username/password/tenant if prompted, or rely on defaults
export PANGOLIN_NO_AUTH=true 

# Create Warehouse
pangolin-admin create-warehouse \
  --name demo_warehouse \
  --bucket warehouse \
  --region us-east-1 \
  --endpoint http://minio:9000 \
  --access-key minioadmin \
  --secret-key minioadmin \
  --path-style true

# Create Catalog
pangolin-admin create-catalog \
  --name demo \
  --warehouse demo_warehouse \
  --location s3://warehouse/demo_catalog
```

#### 🐍 Python SDK
```python
from pypangolin import PangolinClient

# No-Auth Login (creates dummy session)
client = PangolinClient("http://localhost:8080")
client.login("root", "root") 

# Create Warehouse
client.warehouses.create_s3(
    name="demo_warehouse",
    bucket="warehouse",
    region="us-east-1",
    endpoint="http://minio:9000",
    access_key="minioadmin",
    secret_key="minioadmin",
    path_style_access=True
)

# Create Catalog
client.catalogs.create(
    name="demo",
    warehouse="demo_warehouse",
    storage_location="s3://warehouse/demo_catalog"
)
```

#### 🌐 API (cURL)
```bash
# Create Warehouse
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "Content-Type: application/json" \
  -d '{
    "name": "demo_warehouse",
    "vending_strategy": {"AwsStatic": {"access_key_id": "minioadmin", "secret_access_key": "minioadmin"}},
    "storage_config": {
      "type": "s3", "bucket": "warehouse", "region": "us-east-1", "endpoint": "http://minio:9000",
      "access_key_id": "minioadmin", "secret_access_key": "minioadmin", "s3.path-style-access": "true"
    }
  }'

# Create Catalog
curl -X POST http://localhost:8080/api/v1/catalogs \
  -H "Content-Type: application/json" \
  -d '{"name": "demo", "warehouse_name": "demo_warehouse", "storage_location": "s3://warehouse/demo_catalog"}'
```

---

### 3. Run PyIceberg Test (Jupyter)

The Jupyter notebook is pre-configured to talk to the `demo` catalog.

1.  Open **Jupyter Notebook**: http://localhost:8888
2.  Run the following:

```python
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, IntegerType
import pyarrow as pa

# Connect to Pangolin
catalog = load_catalog(
    "pangolin",
    **{
        "type": "rest",
        "uri": "http://pangolin-api:8080/v1/demo",
        "header.X-Iceberg-Access-Delegation": "vended-credentials",
        # Default Tenant ID for No-Auth
        "header.X-Pangolin-Tenant": "00000000-0000-0000-0000-000000000000",
    }
)

# Create Namespace & Table
try: catalog.create_namespace("demo_ns")
except: pass

schema = Schema(NestedField(1, "id", IntegerType(), True), NestedField(2, "name", StringType(), False))
try: table = catalog.create_table("demo_ns.users", schema=schema)
except: table = catalog.load_table("demo_ns.users")

# Write Data (Persists to MinIO)
df = pa.Table.from_pylist([{"id": 1, "name": "Alice"}], schema=schema.as_arrow())
table.append(df)
print(table.scan().to_arrow().to_pydict())
```
