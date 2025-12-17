# PyIceberg: Authenticated with Credential Vending

This is the **Gold Standard** for secure Lakehouse implementations. Clients authenticate via Token, and the Catalog vends short-lived, scoped credentials for specific table operations.

## Configuration

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "local",
    **{
        "type": "rest",
        "uri": "http://localhost:8080/v1/my_catalog",
        
        # Authentication
        "token": "eyJhbGciOi...", 

        # Enable Credential Vending
        "header.X-Iceberg-Access-Delegation": "vended-credentials",
        
        # No S3 keys required!
    }
)
```

## When to use
- Production Multi-Tenant SaaS.
- Zero-Trust environments.
- Scenarios requiring strict audit logging of data access.
