# Azure Synapse Analytics Connections

> [!WARNING]
> **Untested Implementation**
> 
> This connection type has been implemented based on Azure Synapse's ODBC driver documentation but has not been tested against a live Synapse workspace. The implementation should work as designed, but users should verify functionality in their environment.

Securely store and connect to Azure Synapse Analytics with encrypted credentials.

## Installation

```bash
pip install "pypangolin[synapse]"
```

**Additional Requirement**: You must also install the Microsoft ODBC Driver for SQL Server on your system.

### Installing ODBC Driver

**Ubuntu/Debian:**
```bash
curl https://packages.microsoft.com/keys/microsoft.asc | sudo apt-key add -
curl https://packages.microsoft.com/config/ubuntu/$(lsb_release -rs)/prod.list | sudo tee /etc/apt/sources.list.d/mssql-release.list
sudo apt-get update
sudo ACCEPT_EULA=Y apt-get install -y msodbcsql17
```

**macOS:**
```bash
brew tap microsoft/mssql-release https://github.com/Microsoft/homebrew-mssql-release
brew update
brew install msodbcsql17
```

**Windows:**
Download and install from [Microsoft Download Center](https://docs.microsoft.com/en-us/sql/connect/odbc/download-odbc-driver-for-sql-server)

## Registering a Connection

```python
from pypangolin import PangolinClient
from pypangolin.assets.connections import SynapseAsset
from cryptography.fernet import Fernet

client = PangolinClient(uri="http://localhost:8080")
client.login("username", "password")

# User-managed encryption (recommended)
encryption_key = Fernet.generate_key().decode('utf-8')

SynapseAsset.register(
    client,
    catalog="data_sources",
    namespace="warehouses",
    name="prod_synapse",
    connection_string="synapse://myworkspace.sql.azuresynapse.net",
    credentials={
        "server": "tcp:myworkspace.sql.azuresynapse.net,1433",
        "database": "mySynapseDW",
        "username": "synapse_user",
        "password": "securepassword123",
        "driver": "ODBC Driver 17 for SQL Server"  # Optional, defaults to this
    },
    encryption_key=encryption_key,
    store_key=False,
    description="Production Synapse Analytics"
)
```

## Connecting to Synapse

```python
import os

encryption_key = os.getenv("SYNAPSE_ENCRYPTION_KEY")

conn = SynapseAsset.connect(
    client,
    catalog="data_sources",
    namespace="warehouses",
    name="prod_synapse",
    encryption_key=encryption_key
)

# Use the connection
cursor = conn.cursor()
cursor.execute("SELECT @@VERSION")
print(cursor.fetchone())
cursor.close()
conn.close()
```

## Required Credentials

- **server** (required) - Synapse server endpoint (format: `tcp:server.sql.azuresynapse.net,1433`)
- **database** (required) - Database name
- **username** (required) - SQL authentication username
- **password** (required) - SQL authentication password

## Optional Credentials

- **driver** - ODBC driver name (default: "ODBC Driver 17 for SQL Server")

## Connection String Format

```
synapse://workspace.sql.azuresynapse.net
```

## Authentication Methods

### SQL Authentication (Implemented)
```python
credentials={
    "server": "tcp:myworkspace.sql.azuresynapse.net,1433",
    "database": "mydb",
    "username": "sql_user",
    "password": "password123"
}
```

### Azure AD Authentication (Future Enhancement)
Currently not implemented. For Azure AD auth, you would need to extend the implementation to support authentication tokens.

## Example Usage

### Query Data

```python
conn = SynapseAsset.connect(client, "data_sources", "warehouses", "prod_synapse", encryption_key=key)
cursor = conn.cursor()

# Query Synapse
cursor.execute("""
    SELECT 
        CAST(order_date AS DATE) as date,
        COUNT(*) as order_count,
        SUM(total_amount) as revenue
    FROM dbo.orders
    WHERE order_date >= DATEADD(day, -30, GETDATE())
    GROUP BY CAST(order_date AS DATE)
    ORDER BY date DESC
""")

for row in cursor:
    print(f"Date: {row[0]}, Orders: {row[1]}, Revenue: ${row[2]:,.2f}")

cursor.close()
conn.close()
```

### Using with Pandas

```python
import pandas as pd

conn = SynapseAsset.connect(client, "data_sources", "warehouses", "prod_synapse", encryption_key=key)

query = """
    SELECT TOP 1000 
        customer_id,
        product_name,
        quantity,
        price
    FROM dbo.sales
    WHERE sale_date >= DATEADD(month, -1, GETDATE())
"""

df = pd.read_sql(query, conn)
print(df.head())

conn.close()
```

## Security Best Practices

- Use Azure AD authentication when possible (requires custom implementation)
- Enable firewall rules to restrict access
- Use Azure Private Link for secure connectivity
- Rotate SQL credentials regularly
- Use user-managed encryption keys
- Enable Azure Synapse audit logging

## Synapse-Specific Considerations

### Performance Optimization
- Use CTAS (CREATE TABLE AS SELECT) for large data loads
- Leverage distribution and partitioning strategies
- Use PolyBase for external data access
- Consider result set caching

### Resource Management
- Be aware of DWU (Data Warehouse Units) allocation
- Pause compute when not in use to save costs
- Use workload management for query prioritization

## Troubleshooting

### ODBC Driver Not Found
```
Error: [IM002] [Microsoft][ODBC Driver Manager] Data source name not found
```
**Solution**: Install Microsoft ODBC Driver 17 for SQL Server

### Connection Timeout
- Verify firewall rules allow your IP address
- Check if Synapse workspace is paused
- Ensure correct server endpoint format

### Authentication Failed
- Verify username and password are correct
- Check if user has access to the specified database
- Ensure SQL authentication is enabled (not just Azure AD)

## Notes

- Synapse uses T-SQL (Transact-SQL) syntax
- Supports both dedicated SQL pools and serverless SQL pools
- This implementation targets dedicated SQL pools
- Connection string format differs from standard SQL Server
- Test thoroughly in your environment before production deployment
