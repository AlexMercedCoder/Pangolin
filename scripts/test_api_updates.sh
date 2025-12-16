#!/bin/bash
# API Update Endpoints - Manual Test Script
# This script tests the newly implemented update endpoints

set -e

BASE_URL="http://localhost:8080"

echo "=== API Update Endpoints Test Suite ==="
echo ""

# Test 1: Warehouse Update
echo "Test 1: Warehouse Update"
echo "------------------------"

# Create warehouse
echo "1.1 Creating warehouse..."
curl -s -X POST $BASE_URL/api/v1/warehouses \
  -H "Content-Type: application/json" \
  -d '{"name":"test-wh","use_sts":false,"storage_config":{"type":"s3","bucket":"test"}}' | jq '.'

# Update warehouse
echo ""
echo "1.2 Updating warehouse (enable STS)..."
curl -s -X PUT $BASE_URL/api/v1/warehouses/test-wh \
  -H "Content-Type: application/json" \
  -d '{"use_sts":true,"storage_config":{"role_arn":"arn:aws:iam::123:role/test"}}' | jq '.'

# Verify update
echo ""
echo "1.3 Verifying warehouse update..."
USE_STS=$(curl -s $BASE_URL/api/v1/warehouses/test-wh | jq -r '.use_sts')
if [ "$USE_STS" = "true" ]; then
    echo "✅ Warehouse update successful - use_sts is now true"
else
    echo "❌ Warehouse update failed - use_sts is $USE_STS"
fi

echo ""
echo "---"
echo ""

# Test 2: Catalog Update
echo "Test 2: Catalog Update"
echo "----------------------"

# Create catalog
echo "2.1 Creating catalog..."
curl -s -X POST $BASE_URL/api/v1/catalogs \
  -H "Content-Type: application/json" \
  -d '{"name":"test-catalog","warehouse_name":null}' | jq '.'

# Update catalog
echo ""
echo "2.2 Updating catalog (attach warehouse)..."
curl -s -X PUT $BASE_URL/api/v1/catalogs/test-catalog \
  -H "Content-Type: application/json" \
  -d '{"warehouse_name":"test-wh","storage_location":"s3://bucket/catalog"}' | jq '.'

# Verify update
echo ""
echo "2.3 Verifying catalog update..."
WAREHOUSE_NAME=$(curl -s $BASE_URL/api/v1/catalogs/test-catalog | jq -r '.warehouse_name')
if [ "$WAREHOUSE_NAME" = "test-wh" ]; then
    echo "✅ Catalog update successful - warehouse_name is now test-wh"
else
    echo "❌ Catalog update failed - warehouse_name is $WAREHOUSE_NAME"
fi

echo ""
echo "---"
echo ""

# Test 3: Tenant Update/Delete
echo "Test 3: Tenant Update/Delete"
echo "----------------------------"

# List tenants to get an ID
echo "3.1 Listing tenants..."
TENANT_ID=$(curl -s $BASE_URL/api/v1/tenants | jq -r '.[0].id')
echo "Using tenant ID: $TENANT_ID"

# Update tenant
echo ""
echo "3.2 Updating tenant name..."
curl -s -X PUT $BASE_URL/api/v1/tenants/$TENANT_ID \
  -H "Content-Type: application/json" \
  -d '{"name":"updated-tenant"}' | jq '.'

# Verify update
echo ""
echo "3.3 Verifying tenant update..."
TENANT_NAME=$(curl -s $BASE_URL/api/v1/tenants/$TENANT_ID | jq -r '.name')
if [ "$TENANT_NAME" = "updated-tenant" ]; then
    echo "✅ Tenant update successful - name is now updated-tenant"
else
    echo "❌ Tenant update failed - name is $TENANT_NAME"
fi

# Create a new tenant for deletion test
echo ""
echo "3.4 Creating temporary tenant for deletion test..."
NEW_TENANT=$(curl -s -X POST $BASE_URL/api/v1/tenants \
  -H "Content-Type: application/json" \
  -d '{"name":"temp-tenant"}')
NEW_TENANT_ID=$(echo $NEW_TENANT | jq -r '.id')
echo "Created tenant with ID: $NEW_TENANT_ID"

# Delete tenant
echo ""
echo "3.5 Deleting temporary tenant..."
DELETE_RESPONSE=$(curl -s -w "\n%{http_code}" -X DELETE $BASE_URL/api/v1/tenants/$NEW_TENANT_ID)
HTTP_CODE=$(echo "$DELETE_RESPONSE" | tail -n1)

if [ "$HTTP_CODE" = "204" ]; then
    echo "✅ Tenant deletion successful - HTTP 204 No Content"
else
    echo "❌ Tenant deletion failed - HTTP $HTTP_CODE"
fi

# Verify deletion
echo ""
echo "3.6 Verifying tenant deletion..."
VERIFY_RESPONSE=$(curl -s -w "\n%{http_code}" $BASE_URL/api/v1/tenants/$NEW_TENANT_ID)
VERIFY_CODE=$(echo "$VERIFY_RESPONSE" | tail -n1)

if [ "$VERIFY_CODE" = "404" ]; then
    echo "✅ Tenant deletion verified - HTTP 404 Not Found"
else
    echo "❌ Tenant still exists - HTTP $VERIFY_CODE"
fi

echo ""
echo "=== Test Suite Complete ==="
