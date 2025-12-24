# Apache Paimon Assets

Manage Apache Paimon tables in Pangolin.

## Dependencies

Install `pypaimon`:
```bash
pip install "pypangolin[paimon]"
```

## Usage

You can register existing Paimon tables. Writing support is experimental via `pypaimon`.

### Registration

```python
from pypangolin.assets import PaimonAsset

PaimonAsset.register(
    client=client,
    catalog="my_catalog",
    namespace="my_ns",
    name="my_paimon_table",
    location="s3://bucket/paimon/table",
    properties={
        "snapshot.id": "1",
        "schema.id": "0"
    }
)
```

## Recommended Properties

| Property | Description | Example |
|---|---|---|
| `snapshot.id` | Current snapshot ID | `123` |
| `schema.id` | Current schema ID | `5` |
| `primary_keys` | Primary Keys | `id, user_id` |
| `partition_keys` | Partition Keys | `dt, hr` |
| `file_format` | Underlying file format | `orc`, `parquet` |

