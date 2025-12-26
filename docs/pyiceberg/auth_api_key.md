# PyIceberg: Authenticated with Service User API Keys

Service Users provide an alternative to JWT tokens for machine-to-machine communication. Instead of short-lived tokens, you can use a persistent API key.

## Configuration

Unlike standard OIDC/OAuth2 flows, API keys are passed via the `X-API-Key` header. In PyIceberg, you configure this using the `header.` prefix.

### With Credential Vending (Recommended)

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "pangolin",
    **{
        "type": "rest",
        "uri": "http://localhost:8080/v1/my_catalog",
        
        # API Key Authentication
        "header.X-API-Key": "pgl_key_your_api_key_here",
        
        # Tenant Routing (Recommended for Service Users)
        "header.X-Pangolin-Tenant": "550e8400-e29b-41d4-a716-446655440000",
        
        # Enable Credential Vending
        "header.X-Iceberg-Access-Delegation": "vended-credentials",
    }
)
```

### With Client-Provided Credentials

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "pangolin",
    **{
        "type": "rest",
        "uri": "http://localhost:8080/v1/my_catalog",
        "header.X-API-Key": "pgl_key_your_api_key_here",
        "header.X-Pangolin-Tenant": "550e8400-e29b-41d4-a716-446655440000",

        # Local MinIO or specific S3 credentials
        "s3.access-key-id": "minioadmin",
        "s3.secret-access-key": "minioadmin",
        "s3.region": "us-east-1",
        "s3.endpoint": "http://localhost:9000",
        "s3.path-style-access": "true",
    }
)
```

## Security Considerations

1. **Persistent Access**: API keys do not expire unless configured with an expiration date or revoked.
2. **Tenant Scoping**: API keys are locked to a specific tenant.
3. **Secret Management**: Handle API keys as secrets. Do not hardcode them in version control.

## Key Differences from JWT

| Feature | JWT (OAuth2) | API Key (Service User) |
| :--- | :--- | :--- |
| **Duration** | Short-lived (Minutes/Hours) | Long-lived (Months/Years) |
| **Identity** | Individual User | Machine/Account |
| **Header** | `Authorization: Bearer <token>` | `X-API-Key: <key>` |
| **Tenant** | Extracted from Token | Requires `X-Pangolin-Tenant` (Recommended) |
