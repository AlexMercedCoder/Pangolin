# PyIceberg: No Auth with Client Credentials

In this scenario, the Pangolin server is running in `NO_AUTH` mode, and the client provides storage credentials directly.

## Configuration

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "pangolin",
    **{
        "type": "rest",
        "uri": "http://localhost:8080/v1/my_catalog",
        
        # Client-Provided Storage Credentials
        "s3.access-key-id": "minioadmin",
        "s3.secret-access-key": "minioadmin",
        "s3.region": "us-east-1",
        "s3.endpoint": "http://localhost:9000", # Required for local MinIO
        "s3.path-style-access": "true",        # Required for local MinIO
    }
)
```

## When to use
- Initial development and local testing against a MinIO container.
- Simple integration tests where complex auth/vending flows are out of scope.
