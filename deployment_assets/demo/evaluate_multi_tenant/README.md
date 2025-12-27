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
*Note: If you encounter "table warehouses has no column named vending_strategy", run `docker compose down -v` to clear stale volumes.*

### 2. Configure Resources (UI)
You must create a Tenant and a specific Tenant Admin user to manage resources.

#### Part A: Create Tenant & Admin
1. Open **Pangolin UI**: http://localhost:3000
2. Login as **Root**:
   - Username: `admin`
   - Password: `password`
3. Go to **Tenants** → **Create Tenant**
   - **Name**: `Acme`
   - Click **Create**
4. Go to **Users** (Click "Users" in sidebar or go to http://localhost:3000/users)
   - Click **Create User**
   - **Username**: `acme_admin`
   - **Email**: `admin@acme.com`
   - **Password**: `Password123`
   - **Role**: `TenantAdmin` (or `tenant-admin`)
   - **Tenant**: Select `Acme` (Important!)
   - Click **Create**
5. **Logout** (User Icon -> Logout)

#### Part B: Create Warehouse & Catalog
1. Login as **Tenant Admin**:
   - Username: `acme_admin`
   - Password: `Password123`
2. Go to **Warehouses** → **Create Warehouse**
   - **Name**: `acme_wh`
   - **Provider**: `AWS S3`
   - **Bucket**: `warehouse`
   - **Region**: `us-east-1`
   - **Endpoint**: `http://minio:9000`
   - **Access Key**: `minioadmin`
   - **Secret Key**: `minioadmin`
   - **Vending Strategy**: `AWS Static` (Ensure "Use STS" is **UNCHECKED**)
   - **Path Style Access**: `true`
   - Click **Create**
3. Go to **Catalogs** → **Create Catalog**
   - **Name**: `acme_cat`
   - **Warehouse**: `acme_wh`
   - **Storage Location**: `s3://warehouse/acme`
   - Click **Create**

#### Part C: Get Access Token
1. Click **User Icon** (top right) → **Profile**
2. Click **Generate Token**
3. Copy the token string.

### 3. Run PyIceberg Test (Jupyter)

1. Open **Jupyter Notebook**: http://localhost:8888
2. Create a new **Python 3** notebook.
3. Run the following code (replace `<YOUR_TOKEN>`):

```python
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, IntegerType

# REPLACE THIS WITH YOUR COPIED TOKEN
TOKEN = "<YOUR_TOKEN>"

# Connect to Pangolin
catalog = load_catalog(
    "pangolin",
    **{
        "type": "rest",
        "uri": "http://pangolin-api:8080/v1/acme_cat",
        "token": TOKEN,
        "header.X-Iceberg-Access-Delegation": "vended-credentials",
    }
)

# List namespaces
print(f"Namespaces: {catalog.list_namespaces()}")

# Create namespace
print("Creating namespace 'acme_ns'...")
catalog.create_namespace("acme_ns")

# Define Schema
schema = Schema(
    NestedField(1, "id", IntegerType(), required=True),
    NestedField(2, "data", StringType(), required=False),
)

# Create table
print("Creating table 'acme_ns.test_table'...")
table = catalog.create_table("acme_ns.test_table", schema=schema)
print(f"Created table: {table}")
```

## Service URLs
- **Pangolin UI**: http://localhost:3000
- **Pangolin API**: http://localhost:8080
- **Jupyter Notebook**: http://localhost:8888
- **MinIO Console**: http://localhost:9001 (user: `minioadmin`, pass: `minioadmin`)

## Database Info
- Metadata persists in a SQLite database at `/data/pangolin.db` inside the container.
- This maps to the `sqlite_data` Docker volume.
