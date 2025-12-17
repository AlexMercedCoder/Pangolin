# PyIceberg: No Auth with Client Credentials

In this scenario, the Pangolin server is running in `NO_AUTH` mode, and the client provides S3 credentials directly.

## Configuration

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "local",
    **{
        "type": "rest",
        "uri": "http://localhost:8080/v1/my_catalog",
        
        # Client-Provided Credentials
        "s3.access-key-id": "YOUR_ACCESS_KEY",
        "s3.secret-access-key": "YOUR_SECRET_KEY",
        "s3.region": "us-east-1",
        "s3.endpoint": "http://minio:9000", # Optional: Only if using custom S3/MinIO
        "s3.path-style-access": "true",     # Optional: For MinIO compatibility
    }
)
```

## When to use
- Development / Testing.
- Trusted internal networks where authentication is handled at the network layer.
