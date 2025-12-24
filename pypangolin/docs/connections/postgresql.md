# PostgreSQL Connections

Securely store and connect to PostgreSQL databases with encrypted credentials.

## Installation

```bash
pip install "pypangolin[postgres]"
```

## Registering a Connection

### Auto-managed Encryption (Recommended for Development)

```python
from pypangolin import PangolinClient
from pypangolin.assets.connections import PostgreSQLAsset

client = PangolinClient(uri="http://localhost:8080")
client.login("username", "password")

# Register with auto-generated encryption key (stored in properties)
PostgreSQLAsset.register(
    client,
    catalog="data_sources",
    namespace="databases",
    name="prod_postgres",
    connection_string="postgresql://db.example.com:5432/production",
    credentials={
        "username": "dbuser",
        "password": "securepassword123"
    },
    store_key=True,  # Key stored in asset properties
    description="Production PostgreSQL database"
)
```

### User-managed Encryption (Recommended for Production)

```python
from cryptography.fernet import Fernet

# Generate and securely store your encryption key
encryption_key = Fernet.generate_key().decode('utf-8')
# Store this key in environment variables or secrets manager!

PostgreSQLAsset.register(
    client,
    catalog="data_sources",
    namespace="databases",
    name="secure_postgres",
    connection_string="postgresql://db.example.com:5432/production",
    credentials={
        "username": "dbuser",
        "password": "securepassword123"
    },
    encryption_key=encryption_key,
    store_key=False,  # Key NOT stored - you must provide it when connecting
    description="Secure PostgreSQL with user-managed key"
)
```

## Connecting to Database

### Auto-managed Key

```python
# Connect (key retrieved from properties)
conn = PostgreSQLAsset.connect(
    client,
    catalog="data_sources",
    namespace="databases",
    name="prod_postgres"
)

# Use the connection
cursor = conn.cursor()
cursor.execute("SELECT * FROM users LIMIT 10")
results = cursor.fetchall()
cursor.close()
conn.close()
```

### User-managed Key

```python
import os

# Retrieve key from environment variable
encryption_key = os.getenv("DB_ENCRYPTION_KEY")

# Connect (must provide key)
conn = PostgreSQLAsset.connect(
    client,
    catalog="data_sources",
    namespace="databases",
    name="secure_postgres",
    encryption_key=encryption_key
)

# Use the connection
cursor = conn.cursor()
cursor.execute("SELECT version()")
print(cursor.fetchone())
cursor.close()
conn.close()
```

## Connection String Formats

```
postgresql://host:port/database
postgresql://host/database  # Default port 5432
```

## Security Best Practices

> [!IMPORTANT]
> **Production Recommendations**
> 
> 1. **Use user-managed keys** for production databases
> 2. **Store encryption keys** in environment variables or secrets managers (AWS Secrets Manager, HashiCorp Vault)
> 3. **Rotate credentials** periodically and re-register assets
> 4. **Use Pangolin RBAC** to restrict who can access connection assets
> 5. **Monitor access** via audit logs

> [!WARNING]
> **Auto-managed Keys**
> 
> When `store_key=True`, anyone with Read access to the asset can decrypt credentials. Only use for development/testing.

## Advanced Usage

### Connection with SSL

```python
PostgreSQLAsset.register(
    client,
    catalog="data_sources",
    namespace="databases",
    name="ssl_postgres",
    connection_string="postgresql://db.example.com:5432/production",
    credentials={
        "username": "dbuser",
        "password": "securepassword123",
        "sslmode": "require"  # Additional connection parameters
    },
    store_key=False
)
```

## Troubleshooting

### Connection Refused
- Verify PostgreSQL is running and accessible
- Check firewall rules
- Ensure correct host and port

### Authentication Failed
- Verify username and password
- Check PostgreSQL `pg_hba.conf` for authentication method
- Ensure user has necessary permissions

### Encryption Key Errors
- For user-managed keys, ensure you provide the same key used during registration
- Fernet keys must be 32 bytes base64-encoded
