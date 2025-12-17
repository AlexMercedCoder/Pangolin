# PyIceberg: No Auth with Credential Vending

In this scenario, Pangolin `NO_AUTH` mode is on, but it securely vends scoped credentials to PyIceberg. The client needs NO storage credentials.

## Configuration

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "local",
    **{
        "type": "rest",
        "uri": "http://localhost:8080/v1/my_catalog",
        
        # Enable Credential Vending
        "header.X-Iceberg-Access-Delegation": "vended-credentials",
        
        # No S3 keys required!
        # Note: If running locally with Docker, you might still need s3.endpoint
        # so your host machine knows how to resolve the container hostname.
    }
)
```

## When to use
- Internal scenarios where you want to hide long-lived storage keys from clients.
