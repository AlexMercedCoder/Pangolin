#!/bin/bash
# PyIceberg Integration Test Script

set -e

export TENANT_ID="00000000-0000-0000-0000-000000000001"
BASE_URL="http://localhost:8080"

echo "=== Pangolin PyIceberg Integration Test ==="
echo ""

# Step 1: Create Tenant
echo "Step 1: Creating tenant..."
curl -X POST $BASE_URL/api/v1/tenants \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"Test Corp\",
    \"id\": \"$TENANT_ID\"
  }" 2>/dev/null | jq '.' || echo "Tenant may already exist"

echo ""

# Step 2: Create Warehouse with static credentials
echo "Step 2: Creating warehouse..."
curl -X POST $BASE_URL/api/v1/warehouses \
  -H "X-Pangolin-Tenant: $TENANT_ID" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test_warehouse",
    "use_sts": false,
    "storage_config": {
      "type": "memory",
      "bucket": "test-bucket",
      "region": "us-east-1",
      "access_key_id": "test_key",
      "secret_access_key": "test_secret"
    }
  }' 2>/dev/null | jq '.'

echo ""

# Step 3: Create Catalog
echo "Step 3: Creating catalog..."
curl -X POST $BASE_URL/api/v1/catalogs \
  -H "X-Pangolin-Tenant: $TENANT_ID" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "analytics",
    "warehouse_name": "test_warehouse",
    "storage_location": "memory://test-bucket/analytics"
  }' 2>/dev/null | jq '.'

echo ""

# Step 4: List catalogs to verify
echo "Step 4: Listing catalogs..."
curl -X GET $BASE_URL/api/v1/catalogs \
  -H "X-Pangolin-Tenant: $TENANT_ID" \
  2>/dev/null | jq '.'

echo ""
echo "=== Setup Complete! ==="
echo "Tenant ID: $TENANT_ID"
echo "Catalog Name: analytics"
echo "Catalog URI: http://localhost:8080/v1/analytics"
