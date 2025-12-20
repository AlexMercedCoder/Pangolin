# PyIceberg: No Auth with Credential Vending

In this scenario, Pangolin's `NO_AUTH` mode is enabled (for evaluation), but it still securely vends scoped credentials to PyIceberg. 

## Configuration

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "pangolin",
    **{
        "type": "rest",
        "uri": "http://localhost:8080/v1/my_catalog",
        
        # Enable Credential Vending
        "header.X-Iceberg-Access-Delegation": "vended-credentials",
        
        # No storage keys required!
    }
)
```

## When to use
- Internal scenarios where you want to evaluate the vending workflow without setting up full JWT authentication.
- Demonstration and local development where you want to mimic production credentials behavior.

> [!IMPORTANT]
> Even in `NO_AUTH` mode, Pangolin requires a tenant context. If no token is provided, it defaults to the system tenant (`00000000-0000-0000-0000-000000000000`).
