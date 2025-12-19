# Pangolin API - Curl Examples

This document provides practical curl examples for all Pangolin API endpoints.

> **Note**: For Iceberg REST API endpoints (`/v1/*`), refer to the [Apache Iceberg REST Catalog specification](https://github.com/apache/iceberg/blob/main/open-api/rest-catalog-open-api.yaml).

## Table of Contents
- [Authentication](#authentication)
- [Tenant Management](#tenant-management)
- [Warehouse Management](#warehouse-management)
- [Catalog Management](#catalog-management)
- [User Management](#user-management)
- [Credential Vending](#credential-vending)
- [Federated Catalogs](#federated-catalogs)
- [Service Users](#service-users-api-keys)
- [Roles & Permissions](#roles--permissions)
- [OAuth](#oauth)
- [Branch Operations](#branch-operations)
- [Tag Operations](#tag-operations)
- [Merge Operations & Conflict Resolution](#merge-operations--conflict-resolution)
- [Business Metadata & Access Requests](#business-metadata--access-requests)
- [Audit Logs](#audit-logs)
- [Token Management (Admin)](#token-management-admin)
- [System Configuration (Admin)](#system-configuration-admin)
- [Federated Catalog Operations](#federated-catalog-operations)
- [Data Explorer](#data-explorer)

---

## Authentication

### No Auth Mode (Development)
```bash
# Set environment variable
export PANGOLIN_NO_AUTH=true

# No authentication headers needed
curl http://localhost:8080/api/v1/tenants
```

### JWT Authentication
```bash
# Login to get JWT token
curl -X POST http://localhost:8080/api/v1/users/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "password"
  }'

# Use token in subsequent requests
export TOKEN="eyJhbGciOiJIUzI1NiIs..."
curl http://localhost:8080/api/v1/tenants \
  -H "Authorization: Bearer $TOKEN"
```

### Generate Long-Lived Token
```bash
# Generate token for automation/scripts
curl -X POST http://localhost:8080/api/v1/tokens \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "automation-user",
    "expires_in_hours": 720
  }'
```

**Response**:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_at": "2026-01-17T10:00:00Z"
}
```

### Revoke Token
```bash
# Revoke your current token
curl -X POST http://localhost:8080/api/v1/auth/revoke \
  -H "Authorization: Bearer $TOKEN"
```

### API Key Authentication (Service Users)
```bash
# Use API key header
export API_KEY="pgl_key_abc123..."
curl http://localhost:8080/api/v1/catalogs \
  -H "X-API-Key: $API_KEY"
```

---

## Tenant Management

### List All Tenants
```bash
curl http://localhost:8080/api/v1/tenants \
  -H "Authorization: Bearer $TOKEN"
```

**Response**:
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "acme-corp",
    "properties": {},
    "created_at": "2025-12-14T10:00:00Z"
  }
]
```

### Create Tenant
```bash
curl -X POST http://localhost:8080/api/v1/tenants \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "new-tenant",
    "properties": {
      "department": "analytics",
      "owner": "data-team"
    }
  }'
```

### Get Tenant
```bash
curl http://localhost:8080/api/v1/tenants/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer $TOKEN"
```

### Update Tenant
```bash
curl -X PUT http://localhost:8080/api/v1/tenants/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "updated-tenant-name",
    "properties": {
      "department": "data-engineering"
    }
  }'
```

### Delete Tenant
```bash
curl -X DELETE http://localhost:8080/api/v1/tenants/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer $TOKEN"
```

---

## Warehouse Management

### List Warehouses
```bash
curl http://localhost:8080/api/v1/warehouses \
  -H "Authorization: Bearer $TOKEN"
```

### Create S3 Warehouse (Static Credentials)
```bash
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "s3-warehouse",
    "use_sts": false,
    "storage_config": {
      "type": "s3",
      "bucket": "my-data-bucket",
      "region": "us-east-1",
      "access_key_id": "AKIAIOSFODNN7EXAMPLE",
      "secret_access_key": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
      "endpoint": "http://localhost:9000"
    }
  }'
```

### Create S3 Warehouse (AWS STS)
```bash
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "s3-sts-warehouse",
    "use_sts": true,
    "storage_config": {
      "type": "s3",
      "bucket": "my-data-bucket",
      "region": "us-east-1",
      "role_arn": "arn:aws:iam::123456789:role/DataAccess",
      "external_id": "optional-external-id"
    }
  }'
```

### Create Azure Warehouse (OAuth2)
```bash
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "azure-warehouse",
    "use_sts": true,
    "storage_config": {
      "type": "azure",
      "tenant_id": "your-azure-tenant-id",
      "client_id": "your-azure-client-id",
      "client_secret": "your-azure-client-secret",
      "account_name": "mystorageaccount",
      "container": "data"
    }
  }'
```

### Create GCP Warehouse (Service Account)
```bash
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "gcp-warehouse",
    "use_sts": true,
    "storage_config": {
      "type": "gcs",
      "service_account_key": "{\"type\":\"service_account\",\"project_id\":\"my-project\",...}",
      "project_id": "my-project",
      "bucket": "my-bucket"
    }
  }'
```

### Get Warehouse
```bash
curl http://localhost:8080/api/v1/warehouses/s3-warehouse \
  -H "Authorization: Bearer $TOKEN"
```

### Update Warehouse
```bash
curl -X PUT http://localhost:8080/api/v1/warehouses/s3-warehouse \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "use_sts": true,
    "storage_config": {
      "role_arn": "arn:aws:iam::123456789:role/NewRole"
    }
  }'
```

### Delete Warehouse
```bash
curl -X DELETE http://localhost:8080/api/v1/warehouses/s3-warehouse \
  -H "Authorization: Bearer $TOKEN"
```

---

## Catalog Management

### List Catalogs
```bash
curl http://localhost:8080/api/v1/catalogs \
  -H "Authorization: Bearer $TOKEN"
```

### Create Catalog
```bash
curl -X POST http://localhost:8080/api/v1/catalogs \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "production",
    "warehouse_name": "s3-warehouse",
    "storage_location": "s3://my-bucket/warehouse",
    "properties": {
      "owner": "data-team"
    }
  }'
```

### Get Catalog
```bash
curl http://localhost:8080/api/v1/catalogs/production \
  -H "Authorization: Bearer $TOKEN"
```

### Update Catalog
```bash
curl -X PUT http://localhost:8080/api/v1/catalogs/production \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "warehouse_name": "new-warehouse",
    "properties": {
      "environment": "production"
    }
  }'
```

### Delete Catalog
```bash
curl -X DELETE http://localhost:8080/api/v1/catalogs/production \
  -H "Authorization: Bearer $TOKEN"
```

---

## User Management

### List Users
```bash
curl http://localhost:8080/api/v1/users \
  -H "Authorization: Bearer $TOKEN"
```

### Create User
```bash
curl -X POST http://localhost:8080/api/v1/users \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "john.doe",
    "email": "john@example.com",
    "password": "secure-password",
    "tenant_id": "550e8400-e29b-41d4-a716-446655440000"
  }'
```

### Get User
```bash
curl http://localhost:8080/api/v1/users/USER_ID \
  -H "Authorization: Bearer $TOKEN"
```

### Update User
```bash
curl -X PUT http://localhost:8080/api/v1/users/USER_ID \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "newemail@example.com"
  }'
```

### Delete User
```bash
curl -X DELETE http://localhost:8080/api/v1/users/USER_ID \
  -H "Authorization: Bearer $TOKEN"
```

---

## Credential Vending

### Get Table Credentials (AWS STS)
```bash
curl http://localhost:8080/v1/production/namespaces/db/tables/users/credentials \
  -H "Authorization: Bearer $TOKEN"
```

**Response**:
```json
{
  "storage-credentials": [
    {
      "prefix": "s3://my-bucket/warehouse/",
      "config": {
        "access-key": "ASIA...",
        "secret-key": "...",
        "session-token": "...",
        "expiration": "2025-12-14T17:00:00Z",
        "s3.region": "us-east-1"
      }
    }
  ]
}
```

### Get Table Credentials (Azure OAuth2)
```bash
curl http://localhost:8080/v1/production/namespaces/db/tables/users/credentials \
  -H "Authorization: Bearer $TOKEN"
```

**Response**:
```json
{
  "storage-credentials": [
    {
      "prefix": "abfss://data@mystorageaccount.dfs.core.windows.net/",
      "config": {
        "adls.auth.type": "OAuth2",
        "adls.oauth2.token": "eyJ0eXAiOiJKV1QiLCJhbGci..."
      }
    }
  ]
}
```

---

## Federated Catalogs

### List Federated Catalogs
```bash
curl http://localhost:8080/api/v1/federated-catalogs \
  -H "Authorization: Bearer $TOKEN"
```

### Create Federated Catalog
```bash
curl -X POST http://localhost:8080/api/v1/federated-catalogs \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "partner-catalog",
    "config": {
      "base_url": "https://partner.example.com",
      "auth_type": "BearerToken",
      "credentials": {
        "token": "partner-api-token"
      }
    }
  }'
```

### Get Federated Catalog
```bash
curl http://localhost:8080/api/v1/federated-catalogs/partner-catalog \
  -H "Authorization: Bearer $TOKEN"
```

### Test Federated Catalog Connection
```bash
curl -X POST http://localhost:8080/api/v1/federated-catalogs/partner-catalog/test \
  -H "Authorization: Bearer $TOKEN"
```

### Delete Federated Catalog
```bash
curl -X DELETE http://localhost:8080/api/v1/federated-catalogs/partner-catalog \
  -H "Authorization: Bearer $TOKEN"
```

---

## Service Users (API Keys)

### Create Service User
```bash
curl -X POST http://localhost:8080/api/v1/service-users \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "ci-cd-pipeline",
    "description": "Service user for CI/CD automation",
    "expires_at": "2026-12-31T23:59:59Z"
  }'
```

**Response**:
```json
{
  "id": "...",
  "name": "ci-cd-pipeline",
  "api_key": "pgl_key_abc123...",
  "created_at": "2025-12-14T10:00:00Z",
  "expires_at": "2026-12-31T23:59:59Z"
}
```

> **Important**: Save the `api_key` - it's only shown once!

### List Service Users
```bash
curl http://localhost:8080/api/v1/service-users \
  -H "Authorization: Bearer $TOKEN"
```

### Delete Service User
```bash
curl -X DELETE http://localhost:8080/api/v1/service-users/SERVICE_USER_ID \
  -H "Authorization: Bearer $TOKEN"
```

---

## Roles & Permissions

### List Roles
```bash
curl http://localhost:8080/api/v1/roles \
  -H "Authorization: Bearer $TOKEN"
```

### Create Role
```bash
curl -X POST http://localhost:8080/api/v1/roles \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "DataEngineer",
    "description": "Can manage tables and schemas",
    "permissions": ["read", "write", "create"]
  }'
```

### List Permissions
```bash
curl http://localhost:8080/api/v1/permissions \
  -H "Authorization: Bearer $TOKEN"
```

### Grant Permission
```bash
curl -X POST http://localhost:8080/api/v1/permissions \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "USER_ID",
    "scope": "catalog:production",
    "actions": ["read", "write"]
  }'
```

### Revoke Permission
```bash
curl -X DELETE http://localhost:8080/api/v1/permissions/PERMISSION_ID \
  -H "Authorization: Bearer $TOKEN"
```

---

## OAuth

### Initiate OAuth Flow (Google)
```bash
# Open in browser or redirect
curl http://localhost:8080/oauth/authorize/google
```

### Initiate OAuth Flow (GitHub)
```bash
# Open in browser or redirect
curl http://localhost:8080/oauth/authorize/github
```

> **Note**: OAuth flows require browser interaction. The callback endpoint `/oauth/callback/{provider}` is handled automatically.

---

---

## Complete Workflow Example

### 1. Create Complete Environment
```bash
# Set base URL and token
export BASE_URL="http://localhost:8080"
export TOKEN="your-jwt-token"

# Create tenant
TENANT_RESPONSE=$(curl -s -X POST $BASE_URL/api/v1/tenants \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"acme-corp"}')

TENANT_ID=$(echo $TENANT_RESPONSE | jq -r '.id')

# Create warehouse
curl -X POST $BASE_URL/api/v1/warehouses \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name":"s3-warehouse",
    "use_sts":true,
    "storage_config":{
      "type":"s3",
      "bucket":"my-bucket",
      "region":"us-east-1",
      "role_arn":"arn:aws:iam::123:role/DataAccess"
    }
  }'

# Create catalog
curl -X POST $BASE_URL/api/v1/catalogs \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name":"production",
    "warehouse_name":"s3-warehouse"
  }'

# Create namespace (Iceberg REST API)
curl -X POST $BASE_URL/v1/production/namespaces \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "namespace":["analytics"]
  }'

echo "✅ Environment created successfully!"
```

---

## Error Handling

### Common Error Responses

**401 Unauthorized**:
```json
{
  "error": "Unauthorized",
  "message": "Invalid or missing authentication token"
}
```

**404 Not Found**:
```json
{
  "error": "Not Found",
  "message": "Warehouse 'nonexistent' not found"
}
```

**500 Internal Server Error**:
```json
{
  "error": "Internal Server Error",
  "message": "Failed to connect to database"
}
```

---

## Tips & Best Practices

1. **Use jq for JSON parsing**:
   ```bash
   curl http://localhost:8080/api/v1/catalogs | jq '.[] | .name'
   ```

2. **Save tokens in environment variables**:
   ```bash
   export TOKEN=$(curl -s -X POST http://localhost:8080/api/v1/auth/login \
     -H "Content-Type: application/json" \
     -d '{"username":"admin","password":"password"}' | jq -r '.token')
   ```

3. **Use `-v` for debugging**:
   ```bash
   curl -v http://localhost:8080/api/v1/tenants
   ```

4. **Pretty print JSON responses**:
   ```bash
   curl http://localhost:8080/api/v1/catalogs | jq '.'
   ```

---

## Related Documentation

- [OpenAPI Specification](./openapi.yaml) - Complete API schema
- [Apache Iceberg REST Spec](https://github.com/apache/iceberg/blob/main/open-api/rest-catalog-open-api.yaml) - Iceberg endpoints
- [Authentication Guide](../authentication.md) - Authentication setup
- [Getting Started](../getting-started/getting_started.md) - Quick start guide

---

## Branch Operations

### List Branches
```bash
curl http://localhost:8080/api/v1/branches \
  -H "Authorization: Bearer $TOKEN"
```

### Create Branch
```bash
curl -X POST http://localhost:8080/api/v1/branches \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "dev",
    "catalog_name": "production",
    "source_branch": "main"
  }'
```

### Get Branch
```bash
curl http://localhost:8080/api/v1/branches/dev \
  -H "Authorization: Bearer $TOKEN"
```

### List Branch Commits
```bash
curl http://localhost:8080/api/v1/branches/dev/commits \
  -H "Authorization: Bearer $TOKEN"
```

### Merge Branch
```bash
curl -X POST http://localhost:8080/api/v1/branches/merge \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "source_branch": "dev",
    "target_branch": "main",
    "catalog_name": "production"
  }'
```

---

## Tag Operations

### List Tags
```bash
curl http://localhost:8080/api/v1/tags \
  -H "Authorization: Bearer $TOKEN"
```

### Create Tag
```bash
curl -X POST http://localhost:8080/api/v1/tags \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "v1.0.0",
    "catalog_name": "production",
    "commit_id": "abc123..."
  }'
```

### Delete Tag
```bash
curl -X DELETE http://localhost:8080/api/v1/tags/v1.0.0 \
  -H "Authorization: Bearer $TOKEN"
```

---

## Merge Operations & Conflict Resolution

### List Merge Operations
```bash
curl http://localhost:8080/api/v1/catalogs/production/merge-operations \
  -H "Authorization: Bearer $TOKEN"
```

### Get Merge Operation
```bash
curl http://localhost:8080/api/v1/merge-operations/OPERATION_ID \
  -H "Authorization: Bearer $TOKEN"
```

### List Merge Conflicts
```bash
curl http://localhost:8080/api/v1/merge-operations/OPERATION_ID/conflicts \
  -H "Authorization: Bearer $TOKEN"
```

### Resolve Conflict
```bash
curl -X POST http://localhost:8080/api/v1/conflicts/CONFLICT_ID/resolve \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "strategy": "use_source"
  }'
```

### Complete Merge
```bash
curl -X POST http://localhost:8080/api/v1/merge-operations/OPERATION_ID/complete \
  -H "Authorization: Bearer $TOKEN"
```

### Abort Merge
```bash
curl -X POST http://localhost:8080/api/v1/merge-operations/OPERATION_ID/abort \
  -H "Authorization: Bearer $TOKEN"
```

---

## Business Metadata & Access Requests

### Add Business Metadata
```bash
curl -X POST http://localhost:8080/api/v1/assets/ASSET_ID/metadata \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "owner": "data-team",
    "classification": "PII",
    "tags": ["customer", "sensitive"]
  }'
```

### Get Business Metadata
```bash
curl http://localhost:8080/api/v1/assets/ASSET_ID/metadata \
  -H "Authorization: Bearer $TOKEN"
```

### Search Assets
```bash
curl http://localhost:8080/api/v1/assets/search?q=customer \
  -H "Authorization: Bearer $TOKEN"
```

### Request Access to Asset
```bash
curl -X POST http://localhost:8080/api/v1/assets/ASSET_ID/request-access \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "reason": "Need access for analytics project"
  }'
```

### List Access Requests
```bash
curl http://localhost:8080/api/v1/access-requests \
  -H "Authorization: Bearer $TOKEN"
```

### Update Access Request
```bash
curl -X PUT http://localhost:8080/api/v1/access-requests/REQUEST_ID \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "status": "approved"
  }'
```

---

## Audit Logs

### List Audit Events
```bash
curl http://localhost:8080/api/v1/audit \
  -H "Authorization: Bearer $TOKEN"
```

**Response**:
```json
[
  {
    "id": "...",
    "timestamp": "2025-12-18T10:00:00Z",
    "user_id": "...",
    "action": "table.create",
    "resource": "production.analytics.users",
    "details": {}
  }
]
```

---

## Token Management (Admin)

### List User Tokens
```bash
# List all tokens for a specific user (admin only)
curl http://localhost:8080/api/v1/users/USER_ID/tokens \
  -H "Authorization: Bearer $TOKEN"
```

**Response**:
```json
[
  {
    "id": "token-uuid-123",
    "user_id": "user-uuid-456",
    "created_at": "2025-12-19T10:00:00Z",
    "expires_at": "2026-01-19T10:00:00Z",
    "is_valid": true
  }
]
```

### Delete Token
```bash
# Delete a specific token by ID (admin only)
curl -X DELETE http://localhost:8080/api/v1/tokens/TOKEN_ID \
  -H "Authorization: Bearer $TOKEN"
```

---

## System Configuration (Admin)

### Get System Settings
```bash
# Get all system configuration settings (admin only)
curl http://localhost:8080/api/v1/config/settings \
  -H "Authorization: Bearer $TOKEN"
```

**Response**:
```json
{
  "allow_public_signup": false,
  "default_retention_days": 30,
  "default_warehouse_bucket": "my-default-bucket",
  "smtp_host": "smtp.example.com",
  "smtp_port": 587,
  "smtp_user": "noreply@example.com",
  "smtp_password": "***"
}
```

### Update System Settings
```bash
# Update system configuration settings (admin only)
curl -X PUT http://localhost:8080/api/v1/config/settings \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "allow_public_signup": true,
    "default_retention_days": 90,
    "smtp_host": "smtp.newprovider.com"
  }'
```

---

## Federated Catalog Operations

### Sync Federated Catalog
```bash
# Trigger immediate metadata sync for federated catalog
curl -X POST http://localhost:8080/api/v1/federated-catalogs/partner-catalog/sync \
  -H "Authorization: Bearer $TOKEN"
```

**Response**:
```json
{
  "status": "Sync triggered",
  "catalog_name": "partner-catalog",
  "triggered_at": "2025-12-19T10:00:00Z"
}
```

### Get Federated Catalog Stats
```bash
# Get sync statistics and status for federated catalog
curl http://localhost:8080/api/v1/federated-catalogs/partner-catalog/stats \
  -H "Authorization: Bearer $TOKEN"
```

**Response**:
```json
{
  "catalog_name": "partner-catalog",
  "last_synced_at": "2025-12-19T09:00:00Z",
  "sync_status": "success",
  "tables_synced": 42,
  "namespaces_synced": 5,
  "last_error": null,
  "next_scheduled_sync": "2025-12-19T12:00:00Z"
}
```

---

## Data Explorer

### Get Namespace Tree
```bash
# Get hierarchical namespace tree structure for a catalog
curl http://localhost:8080/api/v1/catalogs/production/namespaces/tree \
  -H "Authorization: Bearer $TOKEN"
```

**Response**:
```json
{
  "root": [
    {
      "name": ["analytics"],
      "children": [
        {
          "name": ["analytics", "sales"],
          "children": []
        },
        {
          "name": ["analytics", "marketing"],
          "children": []
        }
      ]
    },
    {
      "name": ["staging"],
      "children": []
    }
  ]
}
```

---

## Complete Examples

### Token Management Workflow
```bash
# 1. List all tokens for a user (admin)
curl http://localhost:8080/api/v1/users/$USER_ID/tokens \
  -H "Authorization: Bearer $TOKEN"

# 2. Delete a specific token
curl -X DELETE http://localhost:8080/api/v1/tokens/$TOKEN_ID \
  -H "Authorization: Bearer $TOKEN"
```

### System Configuration Workflow
```bash
# 1. Get current settings
SETTINGS=$(curl -s http://localhost:8080/api/v1/config/settings \
  -H "Authorization: Bearer $TOKEN")

echo $SETTINGS | jq '.'

# 2. Update specific settings
curl -X PUT http://localhost:8080/api/v1/config/settings \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "allow_public_signup": true,
    "default_retention_days": 60
  }'
```

### Federated Catalog Monitoring
```bash
# 1. Check sync stats
curl http://localhost:8080/api/v1/federated-catalogs/partner-catalog/stats \
  -H "Authorization: Bearer $TOKEN" | jq '.'

# 2. Trigger manual sync if needed
curl -X POST http://localhost:8080/api/v1/federated-catalogs/partner-catalog/sync \
  -H "Authorization: Bearer $TOKEN"

# 3. Wait and check stats again
sleep 30
curl http://localhost:8080/api/v1/federated-catalogs/partner-catalog/stats \
  -H "Authorization: Bearer $TOKEN" | jq '.last_synced_at'
```
