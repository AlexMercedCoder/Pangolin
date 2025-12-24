# Vortex Assets

Support for Vortex file format assets.

## Dependencies

Install `vortex-data` (or similar) manually. Currently `pypangolin` supports registration only.

## Usage

```python
from pypangolin.assets import VortexAsset

VortexAsset.register(
    client=client,
    catalog="my_catalog",
    namespace="my_ns",
    name="my_vortex_file",
    location="s3://bucket/data.vortex",
    properties={
        "vortex.version": "1.0",
        "row_count": "1000"
    }
)
```

## Recommended Properties

| Property | Description | Example |
|---|---|---|
| `vortex.version` | Format version | `1.0` |
| `row_count` | Number of rows | `1000` |
| `compression` | Compression used | `zstd` |
| `schema_description` | Text description of schema | `User events log` |
