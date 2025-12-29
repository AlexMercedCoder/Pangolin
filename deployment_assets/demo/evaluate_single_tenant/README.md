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
   - **Path Style Access**: `true` (Add as property or check box if available)
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
      "secret_access_key": "minioadmin",
      "s3.path-style-access": "true"
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

> [!IMPORTANT]
> To verify file persistence in MinIO, you must run the Python logic **inside the Jupyter Notebook container**. Running scripts from your host machine will fail to write data unless you configure your `/etc/hosts` to resolve `minio` to `127.0.0.1`.

1. Open **Jupyter Notebook**: http://localhost:8888
2. Create a new **Python 3** notebook.
3. Run the content of `demo_client.py`:

```python
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, IntegerType
import pyarrow as pa

# 1. Connect to Pangolin (using internal Docker names)
catalog = load_catalog(
    "pangolin",
    **{
        "type": "rest",
        "uri": "http://pangolin-api:8080/v1/demo",
        "header.X-Iceberg-Access-Delegation": "vended-credentials",
        "header.X-Pangolin-Tenant": "00000000-0000-0000-0000-000000000000",
    }
)

# 2. Create Namespace and Table
try:
    catalog.create_namespace("demo_ns")
except:
    pass

schema = Schema(
    NestedField(1, "id", IntegerType(), required=True),
    NestedField(2, "name", StringType(), required=False),
)

try:
    table = catalog.create_table("demo_ns.users", schema=schema)
except:
    table = catalog.load_table("demo_ns.users")

# 3. Write Data (Required for MinIO Persistence)
# Pangolin's MemoryStore keeps metadata in memory, but data files are written to MinIO.
# You must write data to see files in the bucket.
df = pa.Table.from_pylist(
    [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}],
    schema=schema.as_arrow()
)
table.append(df)
print("Data written to MinIO!")

# 4. Read Data
print("Reading data back...")
read_df = table.scan().to_arrow()
print(read_df.to_pydict())
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
