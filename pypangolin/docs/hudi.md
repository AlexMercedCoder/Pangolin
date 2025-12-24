# Hudi Assets

Apache Hudi tables can be registered in Pangolin.

**Note:** Currently, `pypangolin` only supports **Registration** of existing Hudi tables. Native Python writing is limited (mostly requires Spark), so you should write your Hudi tables using Spark/Flink/Deltastreamer and then register them here.

## Usage

```python
from pypangolin.assets import HudiAsset

HudiAsset.register(
    client=client,
    catalog="my_catalog",
    namespace="my_ns",
    name="my_hudi_table",
    location="s3://bucket/hudi/base/path",
    properties={
        "hoodie.table.type": "COPY_ON_WRITE",
        "hoodie.table.version": "5"
    }
)
```

## Recommended Properties

| Property | Description | Example |
|---|---|---|
| `hoodie.table.type` | Table Type | `COPY_ON_WRITE` or `MERGE_ON_READ` |
| `hoodie.table.version` | Protocol version | `5` |
| `last_commit_time` | Timestamp of last commit | `20240101120000` |
| `record_count` | Approx record count | `5000` |

