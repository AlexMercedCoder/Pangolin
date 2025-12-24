# Snowflake Connections

> [!WARNING]
> **Untested Implementation**
> 
> This connection type has been implemented based on Snowflake's Python connector documentation but has not been tested against a live Snowflake instance. The implementation should work as designed, but users should verify functionality in their environment.

Securely store and connect to Snowflake data warehouses with encrypted credentials.

## Installation

```bash
pip install "pypangolin[snowflake]"
```

## Registering a Connection

```python
from pypangolin import PangolinClient
from pypangolin.assets.connections import SnowflakeAsset
from cryptography.fernet import Fernet

client = PangolinClient(uri="http://localhost:8080")
client.login("username", "password")

# User-managed encryption (recommended)
encryption_key = Fernet.generate_key().decode('utf-8')

SnowflakeAsset.register(
    client,
    catalog="data_sources",
    namespace="warehouses",
    name="prod_snowflake",
    connection_string="snowflake://account.region.snowflakecomputing.com",
    credentials={
        "account": "myaccount",
        "username": "snowflake_user",
        "password": "securepassword123",
        "warehouse": "COMPUTE_WH",      # Optional
        "database": "ANALYTICS",        # Optional
        "schema": "PUBLIC",             # Optional
        "role": "ANALYST"               # Optional
    },
    encryption_key=encryption_key,
    store_key=False,
    description="Production Snowflake warehouse"
)
```

## Connecting to Snowflake

```python
import os

encryption_key = os.getenv("SNOWFLAKE_ENCRYPTION_KEY")

conn = SnowflakeAsset.connect(
    client,
    catalog="data_sources",
    namespace="warehouses",
    name="prod_snowflake",
    encryption_key=encryption_key
)

# Use the connection
cursor = conn.cursor()
cursor.execute("SELECT CURRENT_VERSION()")
print(cursor.fetchone())
cursor.close()
conn.close()
```

## Required Credentials

- **account** (required) - Snowflake account identifier
- **username** (required) - Snowflake username
- **password** (required) - Snowflake password

## Optional Parameters

- **warehouse** - Virtual warehouse to use
- **database** - Default database
- **schema** - Default schema
- **role** - Role to use for the session

## Connection String Format

```
snowflake://account.region.snowflakecomputing.com
```

The connection string is primarily for reference; actual connection parameters are extracted from credentials.

## Security Best Practices

- Use user-managed encryption keys for production
- Store encryption keys in environment variables or secrets managers
- Rotate Snowflake credentials regularly
- Use Snowflake's role-based access control in conjunction with Pangolin RBAC

## Troubleshooting

### Connection Issues
- Verify account identifier is correct (format: `account.region.snowflakecomputing.com`)
- Check network connectivity to Snowflake
- Ensure user has appropriate permissions

### Authentication Errors
- Verify username and password are correct
- Check if MFA is required (may need alternative authentication method)
- Ensure role has necessary privileges

## Example Usage

```python
# Query Snowflake
conn = SnowflakeAsset.connect(client, "data_sources", "warehouses", "prod_snowflake", encryption_key=key)
cursor = conn.cursor()

cursor.execute("""
    SELECT 
        date_trunc('day', order_date) as day,
        count(*) as order_count,
        sum(amount) as total_amount
    FROM orders
    WHERE order_date >= dateadd(day, -30, current_date())
    GROUP BY 1
    ORDER BY 1 DESC
""")

for row in cursor:
    print(f"Date: {row[0]}, Orders: {row[1]}, Amount: ${row[2]:,.2f}")

cursor.close()
conn.close()
```

## Notes

- This implementation uses `snowflake-connector-python`
- Connection pooling is not currently implemented
- For production use, consider implementing connection retry logic
- Test thoroughly in your environment before production deployment
