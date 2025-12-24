# JSON Assets

Manage JSON data files with Pangolin.

## Usage

Use `JsonAsset` to write and register JSON files.

```python
import pandas as pd
from pypangolin.assets import JsonAsset

df = pd.DataFrame({"key": ["value"]})

JsonAsset.write(
    client=client,
    catalog="my_catalog",
    namespace="my_ns",
    name="my_json_data",
    data=df,
    location="s3://bucket/data.json",
    orient="records", # Passed to to_json
    lines=True,
    storage_options={"key": "...", "secret": "..."},
    properties={"orient": "records", "compression": "gzip"}
)
```

## Recommended Properties

| Property | Description | Example |
|---|---|---|
| `file_size_bytes` | Size of the file | `4096` |
| `row_count` | Number of records | `250` |
| `json_type` | Structure of the JSON | `records` (lines), `table`, `split` |
| `compression` | Compression codec if applicable | `gzip`, `none` |
| `schema_json` | Inferred schema structure | `{"fields": [...]}` |

