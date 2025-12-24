# PyPangolin

Python Client for the [Pangolin Data Catalog](https://github.com/AlexMercedCoder/Pangolin).

## Documentation

- [Authenticated Mode Guide](docs/auth_mode.md) - Standard operational mode.
- [No Auth Mode Guide](docs/no_auth_mode.md) - For local development and testing.
- [PyIceberg Integration Guide](docs/iceberg.md) - Setup and MinIO configuration.
- [Publishing Guide](PUBLISHING.md) - Build and release instructions.
- **Generic Asset Guides:**
  - [Delta Lake](docs/delta.md) | [Parquet](docs/parquet.md) | [CSV](docs/csv.md) | [JSON](docs/json.md)
  - [Lance](docs/lance.md) | [Hudi](docs/hudi.md) | [Paimon](docs/paimon.md) | [Vortex](docs/vortex.md)
  - [Other Assets](docs/other.md) (ML Models, Video, Images, etc.)
- [Advanced Git Operations](docs/git_operations.md)
- [Governance & Security](docs/governance.md)
- [Admin & System](docs/admin.md)
- [Federated Catalogs & Views](docs/federated.md)

## Installation

```bash
pip install pypangolin
```

For Delta Lake support:
```bash
pip install "pypangolin[delta]"
```

## Usage

### Core Client

```python
from pypangolin import PangolinClient

client = PangolinClient(uri="http://localhost:8080", token="...")
client.tenants.list()
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
