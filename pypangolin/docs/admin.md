# Admin & System

Administrative features for monitoring, auditing, and system management.

## System Dashboard

Get system-wide statistics and catalog summaries.

```python
from pypangolin import PangolinClient

client = PangolinClient(uri="http://localhost:8080")
client.login("admin", "password")

# Get system stats (Root only)
stats = client.system.get_stats()
print(f"Catalogs: {stats.catalogs_count}, Tenants: {stats.tenants_count}")

# Get catalog summary
summary = client.system.get_catalog_summary("my_catalog")
print(f"Tables: {summary.table_count}, Namespaces: {summary.namespace_count}")
```

## Audit Logging

Track all operations for compliance and debugging.

```python
# List recent audit events
events = client.audit.list_events(limit=10)
for event in events:
    print(f"{event.action} on {event.resource_type} at {event.timestamp}")

# Count events
total = client.audit.count()
print(f"Total audit events: {total}")

# Get specific event
event = client.audit.get(event_id)
print(f"Result: {event.result}, User: {event.user_id}")
```

## Asset Search

Search across all assets with tag filtering.

```python
# Search by query
results = client.search.query("sales")

# Search with tags
results = client.search.query("customer", tags=["pii", "verified"])

for asset in results:
    print(f"{asset.name} ({asset.kind}) - {asset.catalog}.{asset.namespace}")
    print(f"  Has Access: {asset.has_access}, Discoverable: {asset.discoverable}")
```

## Token Management

Generate and manage API tokens for automation.

```python
# Generate token
token_response = client.tokens.generate("ci-pipeline", expires_in_days=90)
token = token_response["token"]

# List my tokens
tokens = client.tokens.list_my_tokens()
for t in tokens:
    print(f"{t['name']}: expires {t.get('expires_at', 'never')}")

# Revoke token
client.tokens.revoke(token_id)
```

## System Configuration

Manage system-wide settings (Root only).

```python
# Get current settings
settings = client.system.get_settings()
print(f"Public signup: {settings['allow_public_signup']}")

# Update settings
new_settings = {
    "allow_public_signup": False,
    "default_retention_days": 90
}
client.system.update_settings(new_settings)
```
