# No Auth Mode Guide

This guide covers how to use `pypangolin` when the Pangolin API is running in No Auth Mode. This mode is ideal for local development and testing.

## Prerequisites

- Pangolin API running with `PANGOLIN_NO_AUTH=true`.

## 1. Connecting without Credentials

In No Auth mode, requests without an authentication token are automatically treated as requests from the default **Tenant Admin**.

```python
from pypangolin import PangolinClient

# Connect without credentials
client = PangolinClient(uri="http://localhost:8080")

# You are now acting as the default Tenant Admin
# Tenant ID: 00000000-0000-0000-0000-000000000000
```

## 2. Default Admin Actions

You can immediately create resources without any setup.

```python
# Create a Warehouse
# Note: You still need to provide credentials for the STORAGE backend (e.g. MinIO/S3), 
# but they are stored in the catalog, not used for API access.
wh = client.warehouses.create_s3(
    name="local_dev_wh",
    bucket="dev-bucket",
    access_key="minio",
    secret_key="minio123"
)

# Create a Catalog
catalog = client.catalogs.create("sandbox", warehouse="local_dev_wh")

# Create a Namespace
ns = client.catalogs.namespaces("sandbox").create(["experiments"])
print(f"Created: {ns.name}")
```

## 3. Creating Users

Even in No Auth mode, you can create users to test permissions or simulate a multi-user environment.

```python
user = client.users.create(
    username="analyst_bob",
    email="bob@example.com",
    role="tenant-user",
    tenant_id="00000000-0000-0000-0000-000000000000",
    password="any_password"
)
```

## 4. Masquerading (Simulating Users)

To test as a specific user (e.g., to verify they CANNOT delete a catalog), you must authenticate. 

*Note: In the current version, the `/login` endpoint still validates passwords if called. To bypass this for testing without passwords, ensure you create users with known passwords or use the default admin context.*

```python
# Authenticate as the new user
bob_client = PangolinClient(
    uri="http://localhost:8080",
    username="analyst_bob", 
    password="any_password", # Must match what was created
    tenant_id="00000000-0000-0000-0000-000000000000"
)

# Bob is restricted by TenantUser role
try:
    bob_client.catalogs.create("cannot_do_this", warehouse="local_dev_wh")
except Exception as e:
    print("Permission Denied as expected")
```
