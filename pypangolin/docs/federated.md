# Federated Catalogs & Views

Connect to remote Iceberg catalogs and create virtual views across your data.

## Federated Catalogs

Federated catalogs allow you to connect to remote Iceberg REST catalogs and query them as if they were local.

### Creating a Federated Catalog

```python
from pypangolin import PangolinClient

client = PangolinClient(uri="http://localhost:8080")
client.login("admin", "password")

# Create federated catalog
fed_cat = client.federated_catalogs.create(
    name="prod_analytics",
    uri="https://prod-catalog.company.com",
    warehouse="prod_warehouse",
    credential="bearer_token_here",
    properties={
        "region": "us-east-1",
        "team": "analytics"
    }
)
```

### Managing Federated Catalogs

```python
# List all federated catalogs
catalogs = client.federated_catalogs.list()
for cat in catalogs:
    print(f"{cat.name}: {cat.properties.get('uri')}")

# Get specific catalog
catalog = client.federated_catalogs.get("prod_analytics")

# Test connection
result = client.federated_catalogs.test_connection("prod_analytics")
if result['status'] == 'connected':
    print(f"✅ Connected to {result['base_url']}")

# Trigger sync
client.federated_catalogs.sync("prod_analytics")

# Get sync stats
stats = client.federated_catalogs.get_stats("prod_analytics")
print(f"Last sync: {stats.last_sync}, Tables: {stats.tables_synced}")

# Delete federated catalog
client.federated_catalogs.delete("prod_analytics")
```

## Views

Create SQL views that reference tables across catalogs.

### Creating Views

```python
# Get view client for a catalog
view_client = client.catalogs.views("my_catalog")

# Create a view
view = view_client.create(
    namespace="analytics",
    name="high_value_customers",
    sql="""
        SELECT customer_id, SUM(amount) as total_spent
        FROM sales.transactions
        WHERE amount > 1000
        GROUP BY customer_id
    """,
    properties={
        "description": "Customers with high transaction values",
        "owner": "analytics-team"
    }
)
```

### Querying Views

```python
# Get view definition
view = view_client.get("analytics", "high_value_customers")
print(f"SQL: {view.sql}")
print(f"Properties: {view.properties}")
```

## CRUD Operations

Full create, read, update, delete support for core resources.

### Tenant Management

```python
# Create tenant
tenant = client.tenants.create("new_tenant")

# Update tenant
updated = client.tenants.update(tenant.id, properties={
    "department": "engineering",
    "cost_center": "CC-1234"
})

# Delete tenant
client.tenants.delete(tenant.id)
```

### Warehouse Management

```python
# Create warehouse
wh = client.warehouses.create_s3(
    "data_warehouse",
    "s3://my-bucket",
    access_key="...",
    secret_key="..."
)

# Update warehouse
updated_wh = client.warehouses.update(
    "data_warehouse",
    storage_config={"s3.bucket": "s3://new-bucket"}
)

# Delete warehouse
client.warehouses.delete("data_warehouse")
```

### Catalog Management

```python
# Create catalog
cat = client.catalogs.create("analytics", "data_warehouse")

# Update catalog
updated_cat = client.catalogs.update("analytics", properties={
    "owner": "data-team",
    "sla": "gold"
})

# Delete catalog
client.catalogs.delete("analytics")
```

## Best Practices

### Federated Catalogs
- **Test connections** before relying on federated catalogs in production
- **Monitor sync stats** to ensure data freshness
- **Use credentials securely** - store tokens in environment variables
- **Set appropriate properties** for tracking ownership and metadata

### Views
- **Document SQL logic** in view properties
- **Version control** view definitions outside Pangolin
- **Test views** against sample data before deployment
- **Use qualified names** to avoid ambiguity across catalogs
