# PyPangolin

Python Client for the [Pangolin Data Catalog](https://github.com/AlexMercedCoder/Pangolin).

## Installation

```bash
pip install pypangolin
```

For Delta Lake support:
```bash
pip install "pypangolin[delta]"
```

## Quick Start

### Core Client

```python
from pypangolin import PangolinClient

# Connect to Pangolin
client = PangolinClient(uri="http://localhost:8080")
client.login("username", "password")

# Work with catalogs
catalogs = client.catalogs.list()

### Warehouse Management

```python
# Create an S3 warehouse with custom configuration (e.g. MinIO)
warehouse = client.warehouses.create_s3(
    name="minio_warehouse",
    bucket="my-bucket",
    endpoint="http://minio:9000",
    access_key="minio",
    secret_key="minio123",
    # Pass extra S3 properties via kwargs
    **{"s3.path-style-access": "true"}
)
```
```

### PyIceberg Integration

```python
from pypangolin import get_iceberg_catalog

catalog = get_iceberg_catalog("analytics", uri="http://localhost:8080", token="...")
table = catalog.load_table("sales.transactions")
```

### Generic Assets (Delta Lake)

```python
from pypangolin.assets import DeltaAsset

# Write data and register in Pangolin automatically
DeltaAsset.write(
    client, 
    "analytics", "staging", "my_delta_table", 
    dataframe, 
    location="s3://bucket/path"
)
```

## Documentation

### Getting Started
- [Authenticated Mode Guide](docs/auth_mode.md) - Standard operational mode with user authentication
- [No Auth Mode Guide](docs/no_auth_mode.md) - For local development and testing
- [Publishing Guide](PUBLISHING.md) - Build and release instructions

### Core Features
- [PyIceberg Integration](docs/iceberg.md) - Apache Iceberg tables with PyIceberg
- [Advanced Git Operations](docs/git_operations.md) - Branching, merging, tagging, and conflict resolution

### Table Formats & Assets
- **Apache Iceberg**: [PyIceberg Integration](docs/iceberg.md)
- **Delta Lake**: [Delta Guide](docs/delta.md)
- **Apache Hudi**: [Hudi Guide](docs/hudi.md)
- **Apache Paimon**: [Paimon Guide](docs/paimon.md)
- **File Formats**: [Parquet](docs/parquet.md) | [CSV](docs/csv.md) | [JSON](docs/json.md)
- **Specialized Formats**: [Lance](docs/lance.md) | [Vortex](docs/vortex.md)
- **Other Assets**: [ML Models, Video, Images, etc.](docs/other.md)

### Advanced Features
- [Governance & Security](docs/governance.md) - RBAC, permissions, service users, metadata
- [Admin & System](docs/admin.md) - Audit logging, search, tokens, system config
- [Federated Catalogs & Views](docs/federated.md) - Remote catalogs and SQL views
- [Database Connections](docs/connections.md) - Secure credential management for databases
  - [PostgreSQL](docs/connections/postgresql.md) ✅ | [MySQL](docs/connections/mysql.md) ✅ | [MongoDB](docs/connections/mongodb.md) ✅
  - [Snowflake](docs/connections/snowflake.md) ⚠️ | [Redshift](docs/connections/redshift.md) ⚠️ | [BigQuery](docs/connections/bigquery.md) ⚠️ | [Synapse](docs/connections/synapse.md) ⚠️
  - [Dremio](docs/connections/dremio.md) ✅

## Features

✅ **Full API Coverage** - Complete support for all Pangolin REST API endpoints  
✅ **PyIceberg Integration** - Seamless Apache Iceberg table operations  
✅ **Multi-Format Support** - Delta, Hudi, Paimon, Parquet, CSV, JSON, and more  
✅ **Git-like Operations** - Branching, merging, tagging with conflict resolution  
✅ **Governance** - Role-based access control and business metadata  
✅ **Federated Catalogs** - Connect to remote Iceberg catalogs  
✅ **Type-Safe** - Pydantic models for all API responses  
✅ **Audit Logging** - Comprehensive audit logs with user attribution

## Verification

To run the end-to-end verification suite against a local Pangolin + MinIO stack:

```bash
# Ensure MinIO is running and buckets exist
python3 scripts/ensure_buckets.py

# Run verification script
python3 scripts/verify_pypangolin_live.py
```

## License

MIT

