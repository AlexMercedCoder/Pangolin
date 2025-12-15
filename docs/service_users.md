# Service Users

## Overview

Service Users are dedicated programmatic identities designed for machine-to-machine API access to Pangolin. Unlike regular user accounts that authenticate with username/password and JWT tokens, Service Users use long-lived API keys for authentication, making them ideal for:

- **CI/CD Pipelines**: Automated data ingestion and catalog management
- **Data Integration Tools**: ETL processes, data orchestration platforms
- **Monitoring & Analytics**: Automated metrics collection and reporting
- **Microservices**: Backend services that need catalog access

## Key Features

- **API Key Authentication**: Secure, bcrypt-hashed API keys via `X-API-Key` header
- **Role-Based Access**: Service users inherit the same RBAC system as regular users
- **Tenant Isolation**: Each service user belongs to a specific tenant
- **Expiration Support**: Optional expiration dates for temporary access
- **Activity Tracking**: Automatic `last_used` timestamp updates
- **Key Rotation**: Secure API key rotation without service interruption
- **Deactivation**: Temporarily disable service users without deletion

## Creating a Service User

### API Request

```bash
POST /api/v1/service-users
Authorization: Bearer <admin-jwt-token>
Content-Type: application/json

{
  "name": "ci-pipeline-user",
  "description": "Service user for CI/CD data ingestion",
  "role": "TenantUser",
  "expires_in_days": 90
}
```

### Response

```json
{
  "service_user_id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "ci-pipeline-user",
  "api_key": "pgl_AbCdEfGhIjKlMnOpQrStUvWxYz1234567890AbCdEfGhIjKlMnOpQrStUvWxYz12",
  "expires_at": "2025-03-13T19:00:00Z"
}
```

> **⚠️ IMPORTANT**: The `api_key` is only shown once during creation. Store it securely - it cannot be retrieved later.

## Using Service Users

### Authentication

Service users authenticate using the `X-API-Key` header:

```bash
curl -H "X-API-Key: pgl_AbCdEfGhIjKlMnOpQrStUvWxYz..." \
     https://pangolin.example.com/api/v1/catalogs
```

### Example: PyIceberg Integration

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "pangolin",
    **{
        "uri": "https://pangolin.example.com",
        "warehouse": "my-warehouse",
        "header.X-API-Key": "pgl_AbCdEfGhIjKlMnOpQrStUvWxYz...",
    }
)

# Use catalog normally
tables = catalog.list_tables("my_namespace")
```

### Example: CI/CD Pipeline

```yaml
# .github/workflows/data-ingestion.yml
name: Data Ingestion
on:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM

jobs:
  ingest:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Ingest Data
        env:
          PANGOLIN_API_KEY: ${{ secrets.PANGOLIN_SERVICE_USER_KEY }}
        run: |
          python scripts/ingest_data.py \
            --api-key "$PANGOLIN_API_KEY" \
            --catalog production \
            --namespace analytics
```

## Managing Service Users

### List Service Users

```bash
GET /api/v1/service-users
Authorization: Bearer <admin-jwt-token>
```

### Get Service User Details

```bash
GET /api/v1/service-users/{id}
Authorization: Bearer <admin-jwt-token>
```

### Update Service User

```bash
PUT /api/v1/service-users/{id}
Authorization: Bearer <admin-jwt-token>
Content-Type: application/json

{
  "name": "updated-name",
  "description": "Updated description",
  "active": true
}
```

### Deactivate Service User

```bash
PUT /api/v1/service-users/{id}
Authorization: Bearer <admin-jwt-token>
Content-Type: application/json

{
  "active": false
}
```

### Delete Service User

```bash
DELETE /api/v1/service-users/{id}
Authorization: Bearer <admin-jwt-token>
```

## API Key Rotation

Rotate an API key to maintain security without service interruption:

```bash
POST /api/v1/service-users/{id}/rotate
Authorization: Bearer <admin-jwt-token>
```

**Response:**
```json
{
  "service_user_id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "ci-pipeline-user",
  "api_key": "pgl_NewKeyAbCdEfGhIjKlMnOpQrStUvWxYz1234567890AbCdEfGhIjKlMnOpQrStUv",
  "expires_at": "2025-03-13T19:00:00Z"
}
```

**Best Practice**: Implement a rotation strategy:
1. Create new API key via rotation endpoint
2. Update your services with the new key
3. Verify services are working with new key
4. Old key is immediately invalidated

## Security Best Practices

### 1. **Principle of Least Privilege**
Assign the minimum role necessary:
- `TenantUser`: Read/write access to tenant resources
- `TenantAdmin`: Full tenant management (use sparingly)

### 2. **Key Storage**
- **DO**: Store in secrets managers (AWS Secrets Manager, HashiCorp Vault, GitHub Secrets)
- **DON'T**: Commit to version control, log in plaintext, or share via email

### 3. **Expiration**
Set expiration dates for temporary access:
```json
{
  "expires_in_days": 90  // Expires in 3 months
}
```

### 4. **Regular Rotation**
Rotate keys periodically (e.g., every 90 days) to limit exposure window.

### 5. **Monitoring**
Monitor `last_used` timestamps to detect:
- Unused service users (candidates for deletion)
- Unexpected usage patterns
- Compromised keys

### 6. **Deactivation Over Deletion**
Temporarily deactivate instead of deleting to preserve audit trails.

## Permissions

Only **Tenant Admins** and **Root Users** can:
- Create service users
- List service users
- View service user details
- Update service users
- Delete service users
- Rotate API keys

Service users themselves have permissions based on their assigned role.

## Implementation Details

### API Key Format
- Prefix: `pgl_`
- Length: 64 random characters (alphanumeric)
- Storage: bcrypt-hashed (cost factor 12)

### Authentication Flow
1. Client sends request with `X-API-Key` header
2. Middleware iterates through service users
3. bcrypt verifies key against stored hash
4. Validates service user is active and not expired
5. Creates session with service user's role and tenant
6. Updates `last_used` timestamp (async)

### Data Model
```rust
pub struct ServiceUser {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub tenant_id: Uuid,
    pub api_key_hash: String,  // bcrypt hash
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub last_used: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub active: bool,
}
```

## Troubleshooting

### "Invalid or expired API key"
- Verify the API key is correct (check for typos)
- Ensure service user is active (`active: true`)
- Check expiration date hasn't passed
- Confirm service user hasn't been deleted

### "Only admins can create service users"
- Ensure you're authenticated as a Tenant Admin or Root user
- Check your JWT token is valid and not expired

### Service user not appearing in list
- Verify you're querying the correct tenant
- Ensure you have admin permissions for that tenant

## Related Documentation

- [Authentication & Authorization](./authentication.md)
- [Role-Based Access Control](./features/rbac.md)
- [API Reference](./api/api_overview.md)
