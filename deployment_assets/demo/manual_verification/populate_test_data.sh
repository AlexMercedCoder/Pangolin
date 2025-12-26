#!/bin/bash
set -e

echo "=== Populating Test Data ==="

# Create warehouse
echo "Creating warehouse..."
curl -s -X POST "http://localhost:8080/api/v1/warehouses" \
  -u admin:password \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-warehouse",
    "warehouse_type": "s3",
    "storage_config": {
      "bucket": "test-bucket",
      "region": "us-east-1",
      "endpoint": "http://localhost:9000"
    }
  }' | jq .

# Get first tenant ID
TENANT_ID=$(curl -s -u admin:password "http://localhost:8080/api/v1/tenants" | jq -r '.[0].id')
echo "Using tenant: $TENANT_ID"

# Create catalog
echo "Creating catalog..."
curl -s -X POST "http://localhost:8080/api/v1/catalogs" \
  -u admin:password \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"test-catalog\",
    \"warehouse_name\": \"test-warehouse\",
    \"tenant_id\": \"$TENANT_ID\"
  }" | jq .

# Create user
echo "Creating user..."
curl -s -X POST "http://localhost:8080/api/v1/users" \
  -u admin:password \
  -H "Content-Type: application/json" \
  -d "{
    \"username\": \"testuser\",
    \"email\": \"test@example.com\",
    \"password\": \"password\",
    \"tenant_id\": \"$TENANT_ID\",
    \"role\": \"TenantUser\"
  }" | jq .

echo ""
echo "=== Checking Dashboard Stats ==="
curl -s -u admin:password "http://localhost:8080/api/v1/dashboard/stats" | jq .
