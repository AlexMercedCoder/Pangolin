# PyIceberg: Authenticated with Credential Vending

This is the **Gold Standard** for secure Lakehouse implementations. Clients authenticate via Token, and the Catalog vends short-lived, scoped credentials for specific table operations.

## Configuration

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "pangolin",
    **{
        "type": "rest",
        "uri": "http://localhost:8080/v1/my_catalog",
        "token": "YOUR_JWT_TOKEN", 
        
        # Enable Credential Vending
        "header.X-Iceberg-Access-Delegation": "vended-credentials",
    }
)
```

## How it works
1. **Authentication**: PyIceberg sends the `token` in the Authorization header.
2. **Access Delegation**: The `X-Iceberg-Access-Delegation` header tells Pangolin to return temporary storage credentials (STS, SAS, or Downscoped tokens) in the `load-table` response.
3. **Data Access**: PyIceberg uses these temporary credentials to read/write data directly to S3, ADLS, or GCS without needing long-lived keys on the client machine.

## When to use
- Production Multi-Tenant SaaS.
- Zero-Trust environments.
- Scenarios requiring strict audit logging of data access.
