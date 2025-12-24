# Delta Lake Assets

`pypangolin` allows you to create, write to, and register Delta Lake tables within the Pangolin Catalog.

## Dependencies

To use Delta Asset support, install `pypangolin` with the `delta` extra or install `deltalake` manually:

```bash
pip install "pypangolin[delta]"
# or
pip install deltalake
```

## Usage

You can use the `DeltaAsset` class to write a Pandas DataFrame as a Delta table and automatically register it.

```python
import pandas as pd
from pypangolin import PangolinClient
from pypangolin.assets import DeltaAsset

# Initialize Client
client = PangolinClient("http://localhost:8080", "user", "password", tenant_id="...")

# Data
df = pd.DataFrame({"id": [1, 2, 3], "data": ["a", "b", "c"]})

# Write and Register
# This writes the dataframe to the specified S3 location as a Delta Table
# and registers it in the 'generic_demo' catalog under 'generic_ns' namespace.
DeltaAsset.write(
    client=client,
    catalog="generic_demo",
    namespace="generic_ns",
    name="my_delta_table",
    data=df,
    location="s3://my-bucket/path/to/delta_table",
    mode="overwrite", # or "append"
    storage_options={
        "AWS_ACCESS_KEY_ID": "...",
        "AWS_SECRET_ACCESS_KEY": "...",
        "AWS_REGION": "us-east-1"
    }
)
```

## Manual Registration

If you have an existing Delta table, you can register it without writing data:

```python
DeltaAsset.register(
    client=client,
    catalog="generic_demo",
    namespace="generic_ns",
    name="existing_delta_table",
    location="s3://existing-bucket/delta-table",
    properties={
        "delta.minReaderVersion": "1",
        "delta.minWriterVersion": "2",
        "description": "Sales data for Q3"
    }
)
```

## Recommended Properties

When registering Delta tables, consider tracking these properties for better discoverability:

| Property | Description | Example |
|---|---|---|
| `delta.minReaderVersion` | Minimum protocol version required to read | `1` |
| `delta.minWriterVersion` | Minimum protocol version required to write | `2` |
| `record_count` | Approximate number of rows | `1000000` |
| `size_in_bytes` | Total size of the table | `500MB` |
| `last_modified` | Timestamp of the last commit | `2025-12-24T12:00:00Z` |
| `partition_columns` | Comma-separated list of partition keys | `date,region` |

