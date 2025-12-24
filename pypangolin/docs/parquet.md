# Parquet Assets

`pypangolin` simplifies the management of Parquet files as managed assets in your catalog.

## Dependencies

Requires `pyarrow` (included in `pypangolin[all]` or base install).

## Usage

Use `ParquetAsset` to write a DataFrame (Pandas/Polars/PyArrow) to a Parquet file and register it.

```python
import pandas as pd
from pypangolin.assets import ParquetAsset

df = pd.DataFrame({"a": [1, 2], "b": [3, 4]})

ParquetAsset.write(
    client=client,
    catalog="my_catalog",
    namespace="my_ns",
    name="my_parquet_data",
    data=df,
    location="s3://bucket/data.parquet",
    storage_options={"key": "...", "secret": "..."},
    properties={"compression": "snappy", "rows": "100"}
)
```

## Recommended Properties

| Property | Description | Example |
|---|---|---|
| `file_size_bytes` | Size of the file | `1024` |
| `row_count` | Number of rows | `500` |
| `compression` | Compression codec used | `snappy`, `gzip`, `zstd` |
| `columns` | JSON list of column names | `["col1", "col2"]` |
| `schema_json` | Full schema definition | `{"fields": [...]}` |

