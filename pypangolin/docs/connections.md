# Database Connections

Securely store and manage database connection credentials with encrypted storage.

## Overview

PyPangolin provides secure credential management for database connections using Fernet encryption. Credentials are encrypted before storage and decrypted only when creating connections.

## Supported Databases

### SQL Databases
- **[PostgreSQL](connections/postgresql.md)** - Open-source relational database ✅ *Tested*
- **[MySQL](connections/mysql.md)** - Popular relational database ✅ *Tested*
- **[Amazon Redshift](connections/redshift.md)** - Cloud data warehouse (Postgres-compatible) ⚠️ *Untested*

### NoSQL Databases
- **[MongoDB](connections/mongodb.md)** - Document database ✅ *Tested*

### Cloud Data Warehouses
- **[Snowflake](connections/snowflake.md)** - Cloud data platform ⚠️ *Untested*
- **[Azure Synapse](connections/synapse.md)** - Microsoft analytics service ⚠️ *Untested*
- **[Google BigQuery](connections/bigquery.md)** - Serverless data warehouse ⚠️ *Untested*

### Analytics Platforms
- **[Dremio](connections/dremio.md)** - Data lakehouse platform with Arrow Flight ✅ *Tested*

## Quick Start

### 1. Install Dependencies

```bash
# Install specific database support
pip install "pypangolin[postgres]"
pip install "pypangolin[mysql]"
pip install "pypangolin[mongodb]"
pip install "pypangolin[dremio]"

# Or install all connection support
pip install "pypangolin[all-connections]"
```

### 2. Register a Connection

```python
from pypangolin import PangolinClient
from pypangolin.assets.connections import PostgreSQLAsset

client = PangolinClient(uri="http://localhost:8080")
client.login("username", "password")

PostgreSQLAsset.register(
    client,
    catalog="data_sources",
    namespace="databases",
    name="my_database",
    connection_string="postgresql://localhost:5432/mydb",
    credentials={
        "username": "dbuser",
        "password": "secret123"
    },
    store_key=True  # Auto-managed encryption
)
```

### 3. Connect to Database

```python
conn = PostgreSQLAsset.connect(
    client,
    catalog="data_sources",
    namespace="databases",
    name="my_database"
)

# Use the connection
cursor = conn.cursor()
cursor.execute("SELECT * FROM users")
results = cursor.fetchall()
cursor.close()
conn.close()
```

## Encryption Modes

### Auto-managed Keys (Development)
```python
Asset.register(..., store_key=True)
```
- Encryption key generated automatically
- Key stored in asset properties
- Convenient for development
- ⚠️ Anyone with asset access can decrypt credentials

### User-managed Keys (Production)
```python
from cryptography.fernet import Fernet

key = Fernet.generate_key().decode('utf-8')
Asset.register(..., encryption_key=key, store_key=False)
```
- You control the encryption key
- Key NOT stored in Pangolin
- More secure for production
- Must provide key when connecting

## Security Best Practices

> [!IMPORTANT]
> **Production Security**
> 
> 1. **Use user-managed keys** for production databases
> 2. **Store keys securely** in environment variables or secrets managers
> 3. **Rotate credentials** regularly
> 4. **Use Pangolin RBAC** to control asset access
> 5. **Monitor access** via audit logs
> 6. **Never commit keys** to version control

## Key Management

### Environment Variables
```python
import os

encryption_key = os.getenv("DB_ENCRYPTION_KEY")
conn = Asset.connect(..., encryption_key=encryption_key)
```

### AWS Secrets Manager
```python
import boto3
import json

secrets = boto3.client('secretsmanager')
secret = secrets.get_secret_value(SecretId='db-encryption-key')
encryption_key = json.loads(secret['SecretString'])['key']
```

### HashiCorp Vault
```python
import hvac

client = hvac.Client(url='https://vault.example.com')
secret = client.secrets.kv.v2.read_secret_version(path='db-keys')
encryption_key = secret['data']['data']['encryption_key']
```

## Connection Lifecycle

1. **Register**: Store encrypted credentials in Pangolin
2. **Connect**: Retrieve and decrypt credentials, create connection
3. **Use**: Execute queries/operations
4. **Close**: Clean up connection resources
5. **Rotate**: Update credentials periodically

## Troubleshooting

### "No encryption key found"
- For auto-managed keys: Ensure `store_key=True` was used during registration
- For user-managed keys: Provide `encryption_key` parameter to `connect()`

### "Fernet key must be 32 url-safe base64-encoded bytes"
- Use `Fernet.generate_key()` to create proper keys
- Don't use arbitrary strings as encryption keys

### Connection Errors
- Verify database is accessible from your network
- Check connection string format
- Ensure credentials are correct
- Review database-specific documentation

## See Also

- [PostgreSQL Guide](connections/postgresql.md)
- [MySQL Guide](connections/mysql.md)
- [MongoDB Guide](connections/mongodb.md)
- [Dremio Guide](connections/dremio.md)
