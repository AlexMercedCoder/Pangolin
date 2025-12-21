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

### 2. Configure Resources

Since the memory store starts empty, you must create a Warehouse and Catalog before you can write data. You can do this via the **UI** or **API**.

#### Option A: Via UI
1. Open **Pangolin UI**: http://localhost:3000
2. Go to **Warehouses** → **Create Warehouse**
   - **Name**: `demo_warehouse`
   - **Provider**: `AWS S3`
   - **Bucket**: `warehouse`
   - **Region**: `us-east-1`
   - **Endpoint**: `http://minio:9000`
   - **Access Key**: `minioadmin`
   - **Secret Key**: `minioadmin`
   - **Vending Strategy**: `AWS Static`
   - Click **Create**.
3. Go to **Catalogs** → **Create Catalog**
   - **Name**: `demo`
   - **Warehouse**: `demo_warehouse`
   - **Storage Location**: `s3://warehouse/demo_catalog`
   - Click **Create**.

#### Option B: Via API (curl)
Run the following commands in your terminal:

```bash
# 1. Create Warehouse
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "Content-Type: application/json" \
  -d '{
    "name": "demo_warehouse",
    "vending_strategy": {
      "AwsStatic": {
        "access_key_id": "minioadmin",
        "secret_access_key": "minioadmin"
      }
    },
    "storage_config": {
      "type": "s3",
      "bucket": "warehouse",
      "region": "us-east-1",
      "endpoint": "http://minio:9000",
      "access_key_id": "minioadmin",
      "secret_access_key": "minioadmin"
    }
  }'

# 2. Create Catalog
curl -X POST http://localhost:8080/api/v1/catalogs \
  -H "Content-Type: application/json" \
  -d '{
    "name": "demo",
    "warehouse_name": "demo_warehouse",
    "storage_location": "s3://warehouse/demo_catalog"
  }'
```

### 3. Run PyIceberg Test (Jupyter)
The included Jupyter notebook server has PyIceberg pre-installed.

1. Open **Jupyter Notebook**: http://localhost:8888
2. Create a new **Python 3** notebook.
3. Run the following code to verify connectivity:

```python
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, IntegerType
import pyarrow as pa

# 1. Connect to Pangolin
# We use the 'demo' catalog we created earlier.
# credentials are 'vended-credentials' to use the API's S3 vending.
catalog = load_catalog(
    "pangolin",
    **{
        "type": "rest",
        "uri": "http://pangolin-api:8080/v1/demo",
        "header.X-Iceberg-Access-Delegation": "vended-credentials",
    }
)

# 2. Create Namespace
print("Creating namespace 'test_ns'...")
try:
    catalog.create_namespace("test_ns")
    print("Namespace created.")
except Exception as e:
    print(f"Namespace likely exists: {e}")

# 3. Define Schema and Create/Load Table
schema = Schema(
    NestedField(1, "id", IntegerType(), required=True),
    NestedField(2, "data", StringType(), required=False),
)

print(f"Successfully loaded table: {table}")
```

## Service URLs
- **Pangolin UI**: http://localhost:3000
- **Pangolin API**: http://localhost:8080
- **Jupyter Notebook**: http://localhost:8888
- **MinIO Console**: http://localhost:9001 (user: `minioadmin`, pass: `minioadmin`)

## Notes
- The default tenant ID in NO_AUTH mode is `00000000-0000-0000-0000-000000000000`.
- Metadata is stored in memory and **will be lost** when the container restarts.
- MinIO data persists in a Docker volume.
