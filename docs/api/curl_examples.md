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
curl -X POST http://localhost:8080/api/v1/auth/login \
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
