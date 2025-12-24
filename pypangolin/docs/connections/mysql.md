# MySQL Connections

Securely store and connect to MySQL databases with encrypted credentials.

## Installation

```bash
pip install "pypangolin[mysql]"
```

## Registering a Connection

```python
from pypangolin import PangolinClient
from pypangolin.assets.connections import MySQLAsset
from cryptography.fernet import Fernet

client = PangolinClient(uri="http://localhost:8080")
client.login("username", "password")

# User-managed encryption (recommended)
encryption_key = Fernet.generate_key().decode('utf-8')

MySQLAsset.register(
    client,
    catalog="data_sources",
    namespace="databases",
    name="prod_mysql",
    connection_string="mysql://db.example.com:3306/production",
    credentials={
        "username": "dbuser",
        "password": "securepassword123"
    },
    encryption_key=encryption_key,
    store_key=False,
    description="Production MySQL database"
)
```

## Connecting to Database

```python
import os

encryption_key = os.getenv("DB_ENCRYPTION_KEY")

conn = MySQLAsset.connect(
    client,
    catalog="data_sources",
    namespace="databases",
    name="prod_mysql",
    encryption_key=encryption_key
)

cursor = conn.cursor()
cursor.execute("SELECT DATABASE()")
print(cursor.fetchone())
cursor.close()
conn.close()
```

## Connection String Formats

```
mysql://host:port/database
mysql://host/database  # Default port 3306
```

## Security Best Practices

Same as PostgreSQL - use user-managed keys for production, store keys in secrets managers, and leverage Pangolin RBAC.
