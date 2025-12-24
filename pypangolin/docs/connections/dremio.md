# Dremio Connections

Connect to Dremio using Arrow Flight with dremioframe.

## Installation

```bash
pip install "pypangolin[dremio]"
```

## Registering a Connection

```python
from pypangolin import PangolinClient
from pypangolin.assets.connections import DremioAsset

client = PangolinClient(uri="http://localhost:8080")
client.login("username", "password")

# For Dremio Cloud
DremioAsset.register(
    client,
    catalog="data_sources",
    namespace="analytics",
    name="dremio_cloud",
    connection_string="grpc+tls://data.dremio.cloud:443",
    credentials={
        "token": "your-personal-access-token",
        "project_id": "your-project-id",  # Optional: specify non-default project
        "tls": "true",
        "disable_certificate_verification": "false"
    },
    store_key=False,  # Recommended for tokens
    description="Dremio Cloud connection"
)

# For Dremio Software
DremioAsset.register(
    client,
    catalog="data_sources",
    namespace="analytics",
    name="dremio_onprem",
    connection_string="grpc://dremio.company.com:32010",
    credentials={
        "username": "dremio_user",
        "password": "password123"
    },
    store_key=True,
    description="On-premise Dremio"
)
```

## Connecting to Dremio

```python
import os

# Connect to Dremio Cloud
encryption_key = os.getenv("DREMIO_ENCRYPTION_KEY")

dremio_conn = DremioAsset.connect(
    client,
    catalog="data_sources",
    namespace="analytics",
    name="dremio_cloud",
    encryption_key=encryption_key
)

# Query using dremioframe
df = dremio_conn.query("SELECT * FROM Samples.\"samples.dremio.com\".\"NYC-taxi-trips\" LIMIT 10")
print(df.head())
```

## Connection String Formats

```
grpc://host:port              # Dremio Software (unencrypted)
grpc+tls://host:port          # Dremio with TLS
grpc+tls://data.dremio.cloud:443  # Dremio Cloud
```

## Credential Options

- **Token-based** (Dremio Cloud): Use `token` credential
- **Username/Password** (Dremio Software): Use `username` and `password`
- **Project ID** (Dremio Cloud): Use `project_id` to specify a non-default project
- **TLS**: Set `tls` to `"true"` for encrypted connections

## Notes

- Dremio Cloud requires TLS and personal access tokens
- For on-premise Dremio, verify the Arrow Flight port (default: 32010)
- `dremioframe` provides pandas-like DataFrame operations
