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
