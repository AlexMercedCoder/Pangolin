# Authenticated Mode Guide

This guide covers how to use `pypangolin` when the Pangolin API is running in Authenticated Mode (default).

## Prerequisites

- Pangolin API running with `PANGOLIN_AUTH_MODE=auth` (default).
- A valid user account (or Root credentials).

## 1. Connecting as Root

The system has a built-in Root superuser defined by environment variables (`PANGOLIN_ROOT_USER`, `PANGOLIN_ROOT_PASSWORD`) or database seeding.

```python
from pypangolin import PangolinClient

# Connect and Login as Root
client = PangolinClient(
    uri="http://localhost:8080",
    username="admin", 
    password="password"
)

print(f"Logged in. Token: {client.token}")
```

## 2. Onboarding a Tenant

The Root user's primary job is to create Tenants and onboard Tenant Admins.

```python
# 1. Create a Tenant
tenant = client.tenants.create(name="acme_corp")
print(f"Created Tenant: {tenant.id}")

# 2. Create a Tenant Admin
# Note: Root users cannot login to tenants directly with their root credentials in some configurations.
# Best practice is to create a specific admin for the tenant.
admin_user = client.users.create(
    username="acme_admin",
    email="admin@acme.com",
    role="tenant-admin",
    tenant_id=tenant.id,
    password="secure_password"
)
```

## 3. Connecting as a Tenant User

Regular users and Tenant Admins must be scoped to their tenant.

```python
# Connect as Tenant Admin
client = PangolinClient(
    uri="http://localhost:8080", 
    username="acme_admin", 
    password="secure_password",
    tenant_id=tenant.id # Optional if username is unique, but recommended
)

# The client automatically handles the Tenant Context for subsequent requests
warehouses = client.warehouses.list()
```

## 4. Managing Resources

Once authenticated as a Tenant Admin, you can manage resources for your organization.

### Warehouses
Warehouses define where your data is stored (S3, GCS, Azure, Local).

```python
warehouse = client.warehouses.create_s3(
    name="analytics_wh",
    bucket="my-company-data",
    region="us-east-1",
    access_key="AWS_ACCESS_KEY", # Optional: for credential vending
    secret_key="AWS_SECRET_KEY",
    vending_strategy="AwsStatic"
)
```

### Catalogs
Catalogs organize your data tables (Iceberg, Delta, etc.).

```python
catalog = client.catalogs.create(
    name="gold_data", 
    warehouse="analytics_wh", 
    type="Local" # Native Pangolin Catalog
)
```

### Namespaces & Tables
Manage the hierarchy within your catalog.

```python
# Working with Namespaces
ns_client = client.catalogs.namespaces("gold_data")
ns_client.create(["sales", "2024"])

# Listing Namespaces
namespaces = ns_client.list()
```
