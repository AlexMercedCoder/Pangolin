# Federated Catalogs

## Overview

**Federated Catalogs** enable Pangolin to act as a transparent proxy for external Iceberg REST catalogs. This allows users to:

- **Authenticate once** to Pangolin and access multiple catalogs (local and external)
- **Centralize access control** using Pangolin's RBAC for all catalogs
- **Simplify credential management** by storing external catalog credentials in Pangolin
- **Enable cross-tenant federation** where one tenant can access another tenant's catalog

## Value Proposition

### For End Users
- **Single Sign-On**: Authenticate to Pangolin once, access all catalogs
- **Unified Interface**: Use the same PyIceberg/Spark configuration for all catalogs
- **No Credential Juggling**: Don't manage separate credentials for each catalog

### For Administrators
- **Centralized Access Control**: Manage permissions in one place
- **Audit Trail**: All catalog access logged through Pangolin
- **Flexible Integration**: Connect to any Iceberg REST-compliant catalog

### For Organizations
- **Cross-Team Collaboration**: Teams can share catalogs across tenants
- **Partner Integration**: Connect to external partner catalogs securely
- **Multi-Cloud**: Access catalogs across different cloud providers

## How It Works

```
User (PyIceberg/Spark)
    ↓ [Authenticate to Pangolin with JWT]
Pangolin API
    ↓ [Check RBAC permissions]
    ↓ [Detect catalog is federated]
Federated Proxy
    ↓ [Forward request with federated credentials]
External Iceberg REST Catalog
    ↓ [Response]
Pangolin (passthrough)
    ↓
User
```

## Creating a Federated Catalog

### Prerequisites
- TenantAdmin or Root role
- External catalog URL and credentials

### API Request

```bash
POST /api/v1/federated-catalogs
Authorization: Bearer <admin-token>
Content-Type: application/json

{
  "name": "partner_catalog",
  "config": {
    "base_url": "https://partner.example.com",
    "auth_type": "ApiKey",
    "credentials": {
      "api_key": "pgl_partner_service_key_xyz..."
    },
    "timeout_seconds": 30
  }
}
```

### Authentication Types

#### 1. None
No authentication required (for public catalogs).

```json
{
  "auth_type": "None",
  "credentials": null
}
```

#### 2. Basic Authentication
HTTP Basic Auth with username and password.

```json
{
  "auth_type": "BasicAuth",
  "credentials": {
    "username": "catalog_user",
    "password": "secret_password"
  }
}
```

#### 3. Bearer Token
JWT or other bearer token authentication.

```json
{
  "auth_type": "BearerToken",
  "credentials": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  }
}
```

#### 4. API Key
X-API-Key header authentication (recommended for Pangolin-to-Pangolin).

```json
{
  "auth_type": "ApiKey",
  "credentials": {
    "api_key": "pgl_service_key_xyz..."
  }
}
```

## Using a Federated Catalog

### PyIceberg

```python
from pyiceberg.catalog import load_catalog

# Connect to Pangolin
catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080",
        "warehouse": "partner_catalog",  # Federated catalog name
        "token": "your-pangolin-jwt-token",
    }
)

# Use normally - requests are transparently forwarded
namespaces = catalog.list_namespaces()
table = catalog.load_table("partner_ns.partner_table")
df = table.scan().to_pandas()
```

### Spark

```scala
spark.conf.set("spark.sql.catalog.partner_catalog", "org.apache.iceberg.spark.SparkCatalog")
spark.conf.set("spark.sql.catalog.partner_catalog.catalog-impl", "org.apache.iceberg.rest.RESTCatalog")
spark.conf.set("spark.sql.catalog.partner_catalog.uri", "http://localhost:8080")
spark.conf.set("spark.sql.catalog.partner_catalog.warehouse", "partner_catalog")
spark.conf.set("spark.sql.catalog.partner_catalog.token", "your-pangolin-jwt-token")

// Query federated catalog
spark.sql("SELECT * FROM partner_catalog.partner_ns.partner_table").show()
```

## Cross-Tenant Federation

### Scenario
- **Tenant A**: Data science team
- **Tenant B**: Data engineering team
- **Goal**: Tenant A needs read access to Tenant B's production catalog

### Setup

**Step 1: Tenant B creates a service user for Tenant A**

```bash
# As Tenant B admin
POST /api/v1/service-users
Authorization: Bearer <tenant-b-admin-token>

{
  "name": "tenant_a_access",
  "description": "Service user for Tenant A to access our catalog",
  "role": "TenantUser",
  "expires_at": "2025-12-31T23:59:59Z"
}

# Response includes API key (save this!)
{
  "id": "...",
  "name": "tenant_a_access",
  "api_key": "pgl_tenant_b_key_xyz123..."
}
```

**Step 2: Tenant A creates federated catalog**

```bash
# As Tenant A admin
POST /api/v1/federated-catalogs
Authorization: Bearer <tenant-a-admin-token>

{
  "name": "tenant_b_production",
  "config": {
    "base_url": "http://tenant-b-pangolin:8080",
    "auth_type": "ApiKey",
    "credentials": {
      "api_key": "pgl_tenant_b_key_xyz123..."
    },
    "timeout_seconds": 30
  }
}
```

**Step 3: Tenant A users access Tenant B's catalog**

```python
# Tenant A user
catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://tenant-a-pangolin:8080",
        "warehouse": "tenant_b_production",  # Federated catalog
        "token": "tenant-a-user-jwt",
    }
)

# Access Tenant B's tables through Pangolin
table = catalog.load_table("production.sales.transactions")
```

## Management Operations

### List Federated Catalogs

```bash
GET /api/v1/federated-catalogs
Authorization: Bearer <token>
```

**Response:**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "partner_catalog",
    "base_url": "https://partner.example.com",
    "auth_type": "ApiKey",
    "timeout_seconds": 30
  }
]
```

### Get Federated Catalog

```bash
GET /api/v1/federated-catalogs/partner_catalog
Authorization: Bearer <token>
```

### Test Connection

```bash
POST /api/v1/federated-catalogs/partner_catalog/test
Authorization: Bearer <token>
```

**Response (Success):**
```json
{
  "status": "connected",
  "catalog": "partner_catalog",
  "base_url": "https://partner.example.com"
}
```

**Response (Failure):**
```json
{
  "status": "failed",
  "catalog": "partner_catalog",
  "error": "Connection timeout"
}
```

### Delete Federated Catalog

```bash
DELETE /api/v1/federated-catalogs/partner_catalog
Authorization: Bearer <admin-token>
```

## Security Best Practices

### 1. **Use Service Users for Federation**
Create dedicated service users with minimal permissions for federated access.

```bash
# Good: Dedicated service user with TenantUser role
POST /api/v1/service-users
{
  "name": "partner_federation",
  "role": "TenantUser",
  "expires_at": "2025-12-31T23:59:59Z"
}
```

### 2. **Set Expiration Dates**
Always set expiration dates for federated credentials.

```json
{
  "expires_at": "2025-12-31T23:59:59Z"
}
```

### 3. **Use HTTPS**
Always use HTTPS for external catalog URLs in production.

```json
{
  "base_url": "https://partner.example.com"  // ✅ HTTPS
}
```

### 4. **Rotate Credentials Regularly**
Rotate service user API keys periodically.

```bash
POST /api/v1/service-users/{id}/rotate
```

### 5. **Monitor Access**
Review audit logs for federated catalog access.

```bash
GET /api/v1/audit
```

### 6. **Least Privilege**
Grant minimum necessary permissions on both sides.

## Limitations

### Allowed Operations
Federated catalogs support **Iceberg REST Spec operations only**:

✅ **Supported:**
- List/create/drop namespaces
- List/load/create/drop/update tables
- List/load/create/drop views
- Report metrics
- Rename table

❌ **Not Supported (Pangolin-specific):**
- Branch operations
- Tag operations
- Merge operations
- Business metadata
- Access requests
- Audit logs

### Technical Constraints
- **No local caching**: All requests forwarded in real-time
- **External catalog must be available**: Returns error if unreachable
- **Read-through proxy**: Pangolin doesn't modify federated data
- **Timeout limit**: Configurable, default 30 seconds

## Troubleshooting

### "Catalog not found"
**Cause**: Federated catalog doesn't exist or wrong name.

**Solution**: List catalogs to verify name.
```bash
GET /api/v1/federated-catalogs
```

### "Connection timeout"
**Cause**: External catalog unreachable or slow.

**Solution**: 
1. Test connection: `POST /api/v1/federated-catalogs/{name}/test`
2. Increase timeout in config
3. Check network connectivity

### "Authentication failed"
**Cause**: Invalid credentials for external catalog.

**Solution**:
1. Verify credentials are correct
2. Check if service user is active
3. Verify service user hasn't expired

### "Forbidden"
**Cause**: User lacks permissions on federated catalog in Pangolin.

**Solution**: Grant user permissions on the federated catalog.

### "Operation not supported"
**Cause**: Attempting Pangolin-specific operation on federated catalog.

**Solution**: Only use Iceberg REST spec operations on federated catalogs.

## Architecture

### Components

1. **FederatedCatalogConfig**: Stores connection details and credentials
2. **FederatedCatalogProxy**: HTTP client for forwarding requests
3. **Route Interceptor**: Detects federated catalogs and routes requests
4. **Management API**: CRUD operations for federated catalogs

### Request Flow

```
1. User request arrives at Pangolin
2. Auth middleware validates JWT
3. Handler checks catalog type
4. If federated:
   a. Get federated config
   b. Forward request with federated credentials
   c. Return response to user
5. If local:
   a. Process normally with local store
```

### Data Model

```rust
pub struct Catalog {
    pub id: Uuid,
    pub name: String,
    pub catalog_type: CatalogType,  // Local or Federated
    pub federated_config: Option<FederatedCatalogConfig>,
    // ...
}

pub struct FederatedCatalogConfig {
    pub base_url: String,
    pub auth_type: FederatedAuthType,
    pub credentials: Option<FederatedCredentials>,
    pub timeout_seconds: u64,
}
```

## Related Documentation

- [Service Users](./service_users.md) - For creating service users for federation
- [Permissions System](../permissions.md) - For managing permissions on federated catalogs
- [Audit Logs](./features/audit_logs.md) - For monitoring federated access
- [API Reference](./api/api_overview.md) - Complete API documentation
