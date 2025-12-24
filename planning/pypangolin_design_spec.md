# PyPangolin Design Specification

**Package Name**: `pypangolin`
**Version**: 0.1.0 (Target)
**Purpose**: A unified Python client for the Pangolin Data Catalog, providing ergonomic wrappers for the REST API, seamless PyIceberg integration, and high-level abstractions for generic asset management.

## 1. Project Structure

```text
pypangolin/
├── client.py          # Core REST API wrapper (Tenants, Users, Warehouses)
├── catalog.py         # PyIceberg compatibility layer
├── assets/            # Generic Asset abstractions
│   ├── __init__.py
│   ├── base.py        # Base Asset Interface
│   ├── delta.py       # Delta Lake integration (deltalake-python)
│   ├── lancedb.py     # LanceDB integration
│   └── files.py       # Polars/Arrow based file handlers (Parquet/CSV/JSON)
├── auth.py            # Authentication & Token management
├── exceptions.py      # Custom exceptions
└── models.py          # Pydantic models for API responses
```

## 2. Dependencies

**Core:**
- `requests`: HTTP client
- `pydantic`: Data validation
- `pyiceberg`: Core Iceberg integration

**Optional (Extras):**
- `deltalake`: For `pypangolin[delta]`
- `pylance`: For `pypangolin[lance]`
- `polars` / `pandas` / `pyarrow`: For generic file IO

## 3. Module Specifications

### 3.1 Core Client (`PangolinClient`)

The entry point for interacting with the Pangolin administrative and metadata APIs.

```python
from pypangolin import PangolinClient

# Initialize with credentials
client = PangolinClient(
    uri="http://localhost:8080",
    username="admin", 
    password="password"
)

# Tenant Management
tenant = client.tenants.create("my_org")
client.tenants.switch("my_org")

# Warehouse Management
warehouse = client.warehouses.create_s3(
    name="prod_warehouse",
    bucket="my-bucket",
    region="us-east-1",
    vending_strategy="AwsStatic"
)

# Catalog Management
catalog = client.catalogs.create(
    name="analytics",
    warehouse="prod_warehouse"
)
```

### 3.2 PyIceberg Abstraction (`get_catalog`)

Simplifies the verbose PyIceberg configuration, automatically handling auth tokens and warehouse properties.

```python
from pypangolin import get_iceberg_catalog

# Returns a configured pyiceberg.catalog.Catalog instance
# Automatically fetches token and configures S3/Azure/GCP credentials
catalog = get_iceberg_catalog(
    name="analytics",
    uri="http://localhost:8080",
    username="my_user",
    password="my_password",
    tenant="my_org"
)

# Standard PyIceberg API follows
table = catalog.load_table("sales.transactions")
df = table.scan().to_pandas()
```

### 3.3 Generic Asset Abstractions

High-level wrappers that combine **File I/O** (using native libraries) with **Metadata Registration** (calling Pangolin API).

#### A. Delta Lake Integration
Requires `deltalake` library.

```python
from pypangolin.assets import DeltaAsset

# 1. Write Data (using deltalake logic under the hood) & Register in Pangolin
# If the table exists, it appends/overwrites. 
# If new, it creates the Delta table AND registers it in Pangolin.
asset = DeltaAsset.write(
    client=client,
    catalog="analytics",
    namespace="bronze",
    name="raw_events",
    data=pandas_df,        # or polars_df or pyarrow_table
    mode="append",
    storage_options={"AWS_ACCESS_KEY_ID": "..."} 
)

# 2. Register Existing Table
asset = DeltaAsset.register(
    client=client,
    catalog="analytics",
    namespace="bronze",
    name="legacy_events",
    location="s3://legacy/delta/events_v1"
)
```

#### B. Generic File Integration (Parquet/CSV/JSON)
Requires `pyarrow` or `polars`.

```python
from pypangolin.assets import ParquetAsset

# Writes parquet file to S3 and registers as generic asset
ParquetAsset.write(
    client=client,
    catalog="raw",
    namespace="landing",
    name="daily_dump_2023_11",
    data=my_dataframe,
    location="s3://bucket/path/to/file.parquet"
)
```

## 4. Workflows

### 4.1 "Pandas to Lake" Flow

```python
import pandas as pd
from pypangolin import PangolinClient
from pypangolin.assets import DeltaAsset

# 1. Setup
client = PangolinClient(uri="...", token="...")
df = pd.DataFrame({"id": [1, 2], "val": ["a", "b"]})

# 2. Write & Catalog in one step
# - User doesn't need to manually call 'create_asset' API
# - User doesn't need to manually manage `deltalake` configurations
DeltaAsset.write(
    client,
    catalog="analytics",
    namespace="staging",
    name="temp_data",
    data=df,
    location="s3://datalake/staging/temp_data"
)

# 3. Validation
# Verify asset exists in Pangolin
assets = client.catalogs.get("analytics").namespaces.get("staging").list_assets()
assert "temp_data" in [a.name for a in assets]
```

## 5. CLI Integration

The library will expose a CLI tool `pypangolin` (distinct from the Rust CLIs `pangolin-admin/user`?) or simply be a library.
*Decision*: Keep as a pure library for now to be used in Notebooks/Scripts.

## 6. Implementation Phases

1.  **Phase 1**: Core Client & PyIceberg Helper (`client.py`, `catalog.py`). Accessing existing APIs.
2.  **Phase 2**: Generic Asset Writer/Register logic (`assets/`).
3.  **Phase 3**: Advanced integrations (Vortex, Lance, Nimble) as libraries mature.
