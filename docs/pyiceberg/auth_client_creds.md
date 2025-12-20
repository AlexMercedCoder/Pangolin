# PyIceberg: Authenticated with Client Credentials

In this scenario, you provide a valid Bearer Token for detailed RBAC and Tenant tracking, but you still manage storage credentials manually on the client side.

## Configuration

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "pangolin",
    **{
        "type": "rest",
        "uri": "http://localhost:8080/v1/my_catalog",
        "token": "YOUR_JWT_TOKEN", # Your Bearer Token

        # Client-Provided Storage Credentials
        "s3.access-key-id": "YOUR_ACCESS_KEY",
        "s3.secret-access-key": "YOUR_SECRET_KEY",
        "s3.region": "us-east-1",
    }
)
```

## When to use
- Multi-tenant environments where clients already have their own direct S3/Azure/GCP access established.
- Situations where your storage provider does not yet support the vending strategies implemented in Pangolin.

## Comparison to Vending
| Aspect | Client Credentials | Credential Vending |
| :--- | :--- | :--- |
| **Security** | Long-lived keys on client. | Short-lived keys on client. |
| **Simplicity** | Client must manage secrets. | Client only needs a JWT token. |
| **Auditability** | Access logged at Storage level. | Access logged at both levels. |
