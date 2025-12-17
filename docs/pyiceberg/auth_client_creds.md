# PyIceberg: Authenticated with Client Credentials

In this scenario, you must provide a valid Bearer Token for detailed RBAC/Tenant tracking, but you still manage storage credentials manually.

## Configuration

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "local",
    **{
        "type": "rest",
        "uri": "http://localhost:8080/v1/my_catalog",
        
        # Authentication
        "token": "eyJhbGciOi...", # Your Bearer Token
        # Optional: Explicitly setting Tenant ID if utilizing root credentials for a tenant
        # "header.X-Pangolin-Tenant": "tenant-uuid",

        # Client-Provided Credentials
        "s3.access-key-id": "YOUR_ACCESS_KEY",
        "s3.secret-access-key": "YOUR_SECRET_KEY",
        "s3.region": "us-east-1",
    }
)
```

## When to use
- Multi-tenant environments where clients have their own direct S3 access.
