# PyIceberg Testing Guide

This guide provides step-by-step instructions for testing Pangolin with PyIceberg.

## Prerequisites

1. **Install PyIceberg and dependencies:**
   ```bash
   pip install "pyiceberg[s3fs,pyarrow]" pyjwt
   ```

2. **Start Pangolin server in NO_AUTH mode (for testing):**
   ```bash
   PANGOLIN_NO_AUTH=1 cargo run --release --bin pangolin_api
   ```

3. **Or start with authentication (production):**
   ```bash
   cargo run --release --bin pangolin_api
   ```

## Quick Start (NO_AUTH Mode)

The easiest way to test PyIceberg is using NO_AUTH mode:

```python
from pyiceberg.catalog import load_catalog

# Connect without authentication
catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080",
        "prefix": "analytics",  # Your catalog name
    }
)

# Test connection
print(f"Connected: {catalog}")

# List namespaces
namespaces = catalog.list_namespaces()
print(f"Namespaces: {namespaces}")
```

## Complete Test Scenarios

### Scenario 1: Basic Connection

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080",
        "prefix": "analytics",
    }
)

print(f"✓ Connected to catalog: {catalog}")
```

### Scenario 2: Namespace Operations

```python
# Create namespace
catalog.create_namespace("test_namespace", properties={"owner": "test"})
print("✓ Created namespace")

# List namespaces
namespaces = catalog.list_namespaces()
print(f"✓ Namespaces: {namespaces}")

# Get namespace properties
props = catalog.load_namespace_properties("test_namespace")
print(f"✓ Properties: {props}")
```

### Scenario 3: Table Creation

```python
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, IntegerType, StringType

# Define schema
schema = Schema(
    NestedField(1, "id", IntegerType(), required=True),
    NestedField(2, "name", StringType(), required=False),
    NestedField(3, "email", StringType(), required=False),
)

# Create table
table = catalog.create_table(
    identifier="test_namespace.users",
    schema=schema,
    location="memory://test-bucket/analytics/test_namespace/users"
)

print(f"✓ Created table: {table}")
```

### Scenario 4: Data Write

```python
import pyarrow as pa

# Create data
data = pa.Table.from_pydict({
    "id": [1, 2, 3],
    "name": ["Alice", "Bob", "Charlie"],
    "email": ["alice@example.com", "bob@example.com", "charlie@example.com"]
})

# Write data
table.append(data)
print(f"✓ Wrote {len(data)} rows")
```

### Scenario 5: Data Read

```python
# Scan table
scan = table.scan()
result = scan.to_arrow()

print(f"✓ Read {len(result)} rows")
print("\nData:")
print(result.to_pandas())
```

### Scenario 6: List Tables

```python
# List tables in namespace
tables = catalog.list_tables("test_namespace")
print(f"✓ Tables: {tables}")
```


## Authentication

Pangolin supports both **OAuth 2.0** and **Basic Authentication**.

> **Note**: For detailed configuration of OAuth providers (Google, GitHub, etc.), please see [Authentication](../authentication.md).

### Option 1: Using an OAuth Token (Recommended)

After logging in via the UI/OAuth flow, you can use your JWT token directly.

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080/iceberg/default", # 'default' is the catalog name
        "type": "rest",
        "token": "YOUR_JWT_TOKEN",            # Token from UI or OAuth callback
        # No S3 keys needed if Credential Vending is active
    }
)
```

### Option 2: Basic Authentication (Root User Only)

**Only the Root User** can use the `credential` parameter directly. Standard Tenant Users must use a Token (Option 1).

```python
catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080/iceberg/default",
        "type": "rest",
        "credential": "root_user:root_password",
    }
)
```

## Obtaining a Token (for Scripts/Service Accounts)

For automated scripts acting as a Tenant User ("Service Account"), you must programmically exchange your credentials for a token before initializing the catalog.

```python
import requests
from pyiceberg.catalog import load_catalog

# 1. Login to get Token
response = requests.post("http://localhost:8080/api/v1/users/login", json={
    "username": "my-service-account",
    "password": "my-secure-password",
    "tenant_id": "00000000-0000-0000-0000-000000000001" # Optional if user unique
})
token = response.json()["token"]

# 2. Use Token
catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080/iceberg/default",
        "type": "rest",
        "token": token
    }
)
```

## Complete Test Script

Save as `test_pyiceberg.py`:

```python
#!/usr/bin/env python3
"""
PyIceberg Integration Test for Pangolin
"""

import sys
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, IntegerType, StringType
import pyarrow as pa

def test_connection():
    """Test 1: Basic connection"""
    print("\n=== Test 1: Connection ===")
    catalog = load_catalog(
        "pangolin",
        **{
            "uri": "http://localhost:8080",
            "prefix": "analytics",
        }
    )
    print(f"✓ Connected to catalog: {catalog}")
    return catalog

def test_namespaces(catalog):
    """Test 2: Namespace operations"""
    print("\n=== Test 2: Namespaces ===")
    
    # List existing
    namespaces = catalog.list_namespaces()
    print(f"Existing namespaces: {namespaces}")
    
    # Create namespace
    catalog.create_namespace("test_namespace")
    print("✓ Created namespace: test_namespace")
    
    # Verify
    namespaces = catalog.list_namespaces()
    print(f"✓ Updated namespaces: {namespaces}")

def test_table_creation(catalog):
    """Test 3: Table creation"""
    print("\n=== Test 3: Table Creation ===")
    
    schema = Schema(
        NestedField(1, "id", IntegerType(), required=True),
        NestedField(2, "name", StringType(), required=False),
        NestedField(3, "email", StringType(), required=False),
    )
    
    table = catalog.create_table(
        identifier="test_namespace.users",
        schema=schema,
        location="memory://test-bucket/analytics/test_namespace/users"
    )
    
    print(f"✓ Created table: {table}")
    return table

def test_data_write(table):
    """Test 4: Write data"""
    print("\n=== Test 4: Data Write ===")
    
    data = pa.Table.from_pydict({
        "id": [1, 2, 3],
        "name": ["Alice", "Bob", "Charlie"],
        "email": ["alice@example.com", "bob@example.com", "charlie@example.com"]
    })
    
    table.append(data)
    print(f"✓ Wrote {len(data)} rows")

def test_data_read(table):
    """Test 5: Read data"""
    print("\n=== Test 5: Data Read ===")
    
    scan = table.scan()
    result = scan.to_arrow()
    
    print(f"✓ Read {len(result)} rows")
    print("\nData:")
    print(result.to_pandas())

def test_list_tables(catalog):
    """Test 6: List tables"""
    print("\n=== Test 6: List Tables ===")
    
    tables = catalog.list_tables("test_namespace")
    print(f"✓ Tables in test_namespace: {tables}")

def main():
    print("=" * 60)
    print("Pangolin PyIceberg Integration Test")
    print("=" * 60)
    
    try:
        catalog = test_connection()
        test_namespaces(catalog)
        table = test_table_creation(catalog)
        test_data_write(table)
        test_data_read(table)
        test_list_tables(catalog)
        
        print("\n" + "=" * 60)
        print("✓ ALL TESTS PASSED!")
        print("=" * 60)
        return 0
        
    except Exception as e:
        print("\n" + "=" * 60)
        print(f"✗ TESTS FAILED: {e}")
        print("=" * 60)
        import traceback
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    sys.exit(main())
```

Run with:
```bash
python3 test_pyiceberg.py
```

## Troubleshooting

### 401 Unauthorized

**In NO_AUTH mode:**
- Ensure server is started with `PANGOLIN_NO_AUTH=1`
- Restart server if you changed the environment variable

**With authentication:**
- Check token is valid and not expired
- Verify token contains correct tenant_id
- Decode token to inspect: `jwt.decode(token, options={"verify_signature": False})`

### 404 Not Found

- Verify catalog name in `prefix` parameter matches existing catalog
- Check catalog was created: `curl http://localhost:8080/api/v1/catalogs`

### Connection Refused

- Ensure Pangolin server is running
- Check server is listening on port 8080
- Verify no firewall blocking connection

### Table Creation Fails

- Check namespace exists first
- Verify storage location is accessible
- Review server logs for detailed error

## Verification Checklist

- [ ] Server starts successfully
- [ ] Connection to catalog works
- [ ] Can list namespaces
- [ ] Can create namespace
- [ ] Can create table
- [ ] Can write data
- [ ] Can read data back
- [ ] Can list tables

## Next Steps

After successful testing:

1. **Production Setup:**
   - Remove `PANGOLIN_NO_AUTH` environment variable
   - Set up proper JWT secret: `PANGOLIN_JWT_SECRET`
   - Configure token generation endpoint
   - Implement token refresh logic

2. **Multi-Tenant Testing:**
   - Create multiple tenants
   - Generate tokens for each tenant
   - Verify tenant isolation

3. **Performance Testing:**
   - Large dataset writes
   - Concurrent operations
   - Query performance

## Additional Resources

- [Client Configuration](./client_configuration.md) - Detailed client setup
- [Getting Started](./getting_started.md) - Quick start guide
- [Warehouse Management](./warehouse_management.md) - Warehouse and catalog setup
