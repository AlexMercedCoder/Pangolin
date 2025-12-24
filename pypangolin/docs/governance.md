# Governance & Security

Pangolin provides robust governance features including Role-Based Access Control (RBAC), Service Users for automation, and Business Metadata for asset enrichment.

## Role-Based Access Control (RBAC)

Manage who can do what with Roles and Permissions.

### Roles

```python
from pypangolin import PangolinClient

client = PangolinClient(...)

# Create a custom role
data_steward = client.roles.create("DataSteward", description="Responsible for data quality")

# List roles
for role in client.roles.list():
    print(f"{role.name}: {role.id}")
```

### Permissions

Grant granular permissions to users or roles.

```python
# Grant 'MANAGE_METADATA' on a specific catalog to the DataSteward role
client.permissions.grant(
    role_id=data_steward.id,
    action="MANAGE_METADATA",
    scope_type="catalog",
    scope_id="catalog_uuid"
)

# Assign role to a user
client.permissions.assign_role(user_id="user_uuid", role_id=data_steward.id)
```

## Service Users

Create restricted users for automated pipelines (ETL/CI).

```python
# Create a service user (defaults to 'tenant-user' role)
bot = client.service_users.create("etl-bot")
print(f"API Key: {bot.api_key}") # Only shown once!

# Rotate key if compromised
new_bot = client.service_users.rotate_key(bot.id)
print(f"New Key: {new_bot.api_key}")
```

## Business Metadata

Enrich assets with searchable metadata, tags, and properties.

```python
# Upsert metadata
client.metadata.upsert(
    asset_id="asset_uuid_or_name", 
    tags=["verified", "pii"],
    properties={
        "owner": "Data Team",
        "sla": "gold"
    },
    description="Primary sales transaction table",
    discoverable=True
)

# Search/Get
meta = client.metadata.get("asset_uuid")
print(meta.tags)
```

## Access Requests

Users can request access to discoverable assets they don't have permission to view.

```python
# Request access
request = client.metadata.request_access("asset_uuid", motivation="Q4 Analysis")

# Admins see requests
requests = client.metadata.list_requests()
```
