# Local Filesystem Warehouse

Pangolin supports using the local filesystem for warehouse storage. This is ideal for development, testing, and single-node deployments where the Pangolin server and the compute engine (e.g., Spark, PyIceberg) run on the same machine or share a mounted volume.

## Configuration

To use a local filesystem warehouse, set the `type` to `local` (or omit specific config keys usually required for S3/Azure/GCS) and provide a path.

### API Example

```bash
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "local-warehouse",
    "storage_config": {
      "type": "local",
      "path": "/tmp/pangolin-data" 
    },
    "vending_strategy": {
      "type": "None"
    }
  }'
```

> **Note**: The `path` must be accessible/writable by the Pangolin server process.

## Usage with PyIceberg

When using a local warehouse, PyIceberg needs to be able to access the files directly.

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "local",
    **{
        "type": "rest",
        "uri": "http://localhost:8080/api/v1/catalogs/my_catalog/iceberg",
        "token": "...",
        # No specific storage credentials needed for local files
    }
)
```

## Credential Vending

The Local Filesystem warehouse type supports the `None` vending strategy. It does **not** vend temporary credentials because file access is managed by the operating system's file permissions.

## Limitations

- **Scalability**: Limited by the local disk size and IO.
- **Concurrency**: Local filesystems may not handle high concurrent write loads as well as object stores.
- **Sharing**: Data is only accessible to processes that can read the specific directory. Not suitable for distributed compute clusters unless a shared network file system (NFS/EFS) is used.
