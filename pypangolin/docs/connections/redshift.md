# Amazon Redshift Connections

> [!WARNING]
> **Untested Implementation**
> 
> This connection type has been implemented based on Redshift's PostgreSQL compatibility but has not been tested against a live Redshift cluster. The implementation should work as designed, but users should verify functionality in their environment.

Securely store and connect to Amazon Redshift data warehouses with encrypted credentials.

## Installation

```bash
pip install "pypangolin[redshift]"
```

Redshift uses the PostgreSQL driver (`psycopg2`) since Redshift is PostgreSQL-compatible.

## Registering a Connection

```python
from pypangolin import PangolinClient
from pypangolin.assets.connections import RedshiftAsset
from cryptography.fernet import Fernet

client = PangolinClient(uri="http://localhost:8080")
client.login("username", "password")

# User-managed encryption (recommended)
encryption_key = Fernet.generate_key().decode('utf-8')

RedshiftAsset.register(
    client,
    catalog="data_sources",
    namespace="warehouses",
    name="prod_redshift",
    connection_string="redshift://cluster-name.region.redshift.amazonaws.com:5439/analytics",
    credentials={
        "username": "redshift_user",
        "password": "securepassword123"
    },
    encryption_key=encryption_key,
    store_key=False,
    description="Production Redshift cluster"
)
```

## Connecting to Redshift

```python
import os

encryption_key = os.getenv("REDSHIFT_ENCRYPTION_KEY")

conn = RedshiftAsset.connect(
    client,
    catalog="data_sources",
    namespace="warehouses",
    name="prod_redshift",
    encryption_key=encryption_key
)

# Use the connection (same as PostgreSQL)
cursor = conn.cursor()
cursor.execute("SELECT version()")
print(cursor.fetchone())
cursor.close()
conn.close()
```

## Connection String Format

```
redshift://cluster-name.region.redshift.amazonaws.com:5439/database
redshift://cluster-endpoint:5439/database
```

## Security Best Practices

- Use IAM authentication when possible (requires additional setup)
- Enable SSL/TLS for connections
- Use VPC security groups to restrict access
- Rotate credentials regularly
- Use user-managed encryption keys

## Redshift-Specific Features

### Using SSL

```python
RedshiftAsset.register(
    client,
    catalog="data_sources",
    namespace="warehouses",
    name="secure_redshift",
    connection_string="redshift://cluster.region.redshift.amazonaws.com:5439/analytics",
    credentials={
        "username": "redshift_user",
        "password": "securepassword123",
        "sslmode": "require"  # Require SSL connection
    },
    encryption_key=encryption_key,
    store_key=False
)
```

## Example Usage

```python
# Query Redshift
conn = RedshiftAsset.connect(client, "data_sources", "warehouses", "prod_redshift", encryption_key=key)
cursor = conn.cursor()

# Redshift-optimized query
cursor.execute("""
    SELECT 
        date_trunc('day', event_time) as day,
        event_type,
        count(*) as event_count
    FROM events
    WHERE event_time >= current_date - interval '7 days'
    GROUP BY 1, 2
    ORDER BY 1 DESC, 3 DESC
""")

for row in cursor:
    print(f"Day: {row[0]}, Event: {row[1]}, Count: {row[2]}")

cursor.close()
conn.close()
```

## Troubleshooting

### Connection Refused
- Verify cluster is available and not paused
- Check VPC security group rules
- Ensure cluster endpoint is correct
- Verify port 5439 is accessible

### Authentication Failed
- Verify username and password
- Check if user exists in the cluster
- Ensure user has CONNECT privilege on database

## Notes

- Redshift is PostgreSQL 8.0.2 compatible
- Uses `psycopg2` driver (same as PostgreSQL)
- Default port is 5439 (not 5432 like PostgreSQL)
- Supports most PostgreSQL syntax with some differences
- Test thoroughly in your environment before production deployment
