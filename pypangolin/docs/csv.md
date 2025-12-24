# CSV Assets

Manage CSV files as first-class citizens in your Pangolin Catalog.

## Usage

Use `CsvAsset` to write and register CSV files.

```python
import pandas as pd
from pypangolin.assets import CsvAsset

df = pd.DataFrame({"col1": ["val1"], "col2": ["val2"]})

CsvAsset.write(
    client=client,
    catalog="my_catalog",
    namespace="my_ns",
    name="my_csv_data",
    data=df,
    location="s3://bucket/data.csv",
    index=False, # Passed to to_csv
    storage_options={"key": "...", "secret": "..."},
    properties={"delimiter": ",", "encoding": "utf-8"}
)
```

## Recommended Properties

| Property | Description | Example |
|---|---|---|
| `file_size_bytes` | Size of the file | `2048` |
| `row_count` | Number of rows | `100` |
| `delimiter` | Field separator character | `,`, `\t`, `|` |
| `encoding` | Character encoding | `utf-8`, `latin1` |
| `has_header` | Whether the first row is a header | `true`, `false` |

