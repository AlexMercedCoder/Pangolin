# PyIceberg Integration Guide

`pypangolin` provides seamless integration with [PyIceberg](https://py.iceberg.apache.org/), simplifying authentication and configuration.

## Installation

To use PyIceberg with Pangolin, install `pypangolin` with the appropriate storage extras to ensure you have the necessary file system drivers.

```bash
# For AWS S3 / MinIO
pip install "pypangolin[s3]"

# For Azure Blob Storage
pip install "pypangolin[azure]"

# For Google Cloud Storage
pip install "pypangolin[gcs]"

# For all backends
pip install "pypangolin[all]"
```

## Getting an Iceberg Catalog

Instead of manually configuring the PyIceberg `RestCatalog`, use the `get_iceberg_catalog` helper. This function:
1.  Constructs the correct REST URI.
2.  Injects your authentication token.
3.  Sets the `X-Pangolin-Tenant` header (critical for multi-tenancy).
4.  Enables **Credential Vending** (`X-Iceberg-Access-Delegation`) by default.

```python
from pypangolin import PangolinClient, get_iceberg_catalog

# 1. Login to Pangolin
client = PangolinClient(uri="http://localhost:8080", username="admin", password="password")

# 2. Get the Iceberg Catalog Client
# This catalog is fully authenticated and scoped to your user/tenant.
catalog = get_iceberg_catalog("my_catalog", uri=client.uri, token=client.token)

# 3. Use standard PyIceberg API
namespaces = catalog.list_namespaces()
table = catalog.load_table("sales.transactions")
```

## Using with MinIO (or On-Prem S3)

Pangolin supports on-premise S3-compatible storage like MinIO.

### 1. Warehouse Configuration
When creating an S3 Warehouse for MinIO, you **must** provide the `endpoint` URL so the Pangolin server knows where to write metadata files.

```python
client.warehouses.create_s3(
    name="minio_wh",
    bucket="my-data",
    endpoint="http://minio:9000", # URL reachable by the Pangolin SERVER
    access_key="minio",
    secret_key="minio123",
    vending_strategy="AwsStatic"
)
```

### 2. Client Configuration
When initializing the Iceberg catalog, you may also need to provide the endpoint for the **Client** (PyIceberg) to read/write data, as it might differ from the server's internal URL (e.g., `localhost` vs `minio` container name).

```python
catalog = get_iceberg_catalog(
    "minio_catalog",
    uri=client.uri,
    token=client.token,
    # Pass extra properties to PyIceberg
    **{"s3.endpoint": "http://localhost:9000"} 
)
```

### 3. Client-Side Credentials (No Vending)

If you are connecting to a catalog that does not have a warehouse configured, or if you prefer to manage credentials manually:

1.  Disable credential vending: `credential_vending=False`.
2.  Pass your storage credentials directly in the `**properties` argument.

**Note:** For write operations (creating tables), the Pangolin Server must also have write access to the metadata location (e.g., via IAM roles or environment variables).

```python
catalog = get_iceberg_catalog(
    "manual_catalog",
    uri=client.uri,
    token=client.token,
    credential_vending=False,
    **{
        "s3.endpoint": "http://localhost:9000",
        "s3.access-key-id": "YOUR_ACCESS_KEY",
        "s3.secret-access-key": "YOUR_SECRET_KEY",
        "s3.region": "us-east-1"
    }
)
```
