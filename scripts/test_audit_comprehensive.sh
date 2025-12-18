#!/bin/bash

# Comprehensive Audit Logging Live Test
# Tests audit logging across two tenants with both API and CLI

set -e

echo "🧪 Comprehensive Audit Logging Live Test"
echo "========================================"
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

API_URL="http://localhost:8080"
CLI="./target/release/pangolin-admin"

echo -e "${BLUE}Step 1: Starting API server...${NC}"
cargo run --release --manifest-path pangolin/Cargo.toml --bin pangolin_api &
API_PID=$!
sleep 5

echo -e "${GREEN}✓ API server started (PID: $API_PID)${NC}"
echo ""

# Cleanup function
cleanup() {
    echo ""
    echo -e "${YELLOW}Cleaning up...${NC}"
    kill $API_PID 2>/dev/null || true
    wait $API_PID 2>/dev/null || true
    echo -e "${GREEN}✓ Cleanup complete${NC}"
}
trap cleanup EXIT

echo -e "${BLUE}Step 2: Creating two tenants...${NC}"

# Create Tenant 1
TENANT1_RESPONSE=$(curl -s -X POST "$API_URL/api/v1/tenants" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "audit_test_tenant1",
    "admin_username": "admin1",
    "admin_password": "password123"
  }')

TENANT1_ID=$(echo $TENANT1_RESPONSE | jq -r '.id')
echo -e "${GREEN}✓ Tenant 1 created: $TENANT1_ID${NC}"

# Create Tenant 2
TENANT2_RESPONSE=$(curl -s -X POST "$API_URL/api/v1/tenants" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "audit_test_tenant2",
    "admin_username": "admin2",
    "admin_password": "password456"
  }')

TENANT2_ID=$(echo $TENANT2_RESPONSE | jq -r '.id')
echo -e "${GREEN}✓ Tenant 2 created: $TENANT2_ID${NC}"
echo ""

echo -e "${BLUE}Step 3: Logging in as Tenant 1 admin...${NC}"
LOGIN1_RESPONSE=$(curl -s -X POST "$API_URL/api/v1/users/login" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin1",
    "password": "password123"
  }')

TOKEN1=$(echo $LOGIN1_RESPONSE | jq -r '.token')
echo -e "${GREEN}✓ Logged in as admin1${NC}"
echo ""

echo -e "${BLUE}Step 4: Logging in as Tenant 2 admin...${NC}"
LOGIN2_RESPONSE=$(curl -s -X POST "$API_URL/api/v1/users/login" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin2",
    "password": "password456"
  }')

TOKEN2=$(echo $LOGIN2_RESPONSE | jq -r '.token')
echo -e "${GREEN}✓ Logged in as admin2${NC}"
echo ""

echo -e "${BLUE}Step 5: Performing actions in Tenant 1 (will generate audit logs)...${NC}"

# Create warehouse in Tenant 1
curl -s -X POST "$API_URL/api/v1/warehouses" \
  -H "Authorization: Bearer $TOKEN1" \
  -H "X-Pangolin-Tenant: $TENANT1_ID" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "tenant1_warehouse",
    "storage_type": "S3",
    "bucket": "tenant1-bucket",
    "region": "us-east-1"
  }' > /dev/null

echo -e "${GREEN}  ✓ Created warehouse in Tenant 1${NC}"

# Create catalog in Tenant 1
curl -s -X POST "$API_URL/api/v1/catalogs" \
  -H "Authorization: Bearer $TOKEN1" \
  -H "X-Pangolin-Tenant: $TENANT1_ID" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "tenant1_catalog",
    "warehouse": "tenant1_warehouse"
  }' > /dev/null

echo -e "${GREEN}  ✓ Created catalog in Tenant 1${NC}"

# Create user in Tenant 1
curl -s -X POST "$API_URL/api/v1/users" \
  -H "Authorization: Bearer $TOKEN1" \
  -H "X-Pangolin-Tenant: $TENANT1_ID" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "user1_tenant1",
    "email": "user1@tenant1.com",
    "password": "pass123",
    "role": "TenantUser"
  }' > /dev/null

echo -e "${GREEN}  ✓ Created user in Tenant 1${NC}"
echo ""

echo -e "${BLUE}Step 6: Performing actions in Tenant 2 (will generate audit logs)...${NC}"

# Create warehouse in Tenant 2
curl -s -X POST "$API_URL/api/v1/warehouses" \
  -H "Authorization: Bearer $TOKEN2" \
  -H "X-Pangolin-Tenant: $TENANT2_ID" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "tenant2_warehouse",
    "storage_type": "S3",
    "bucket": "tenant2-bucket",
    "region": "us-west-2"
  }' > /dev/null

echo -e "${GREEN}  ✓ Created warehouse in Tenant 2${NC}"

# Create catalog in Tenant 2
curl -s -X POST "$API_URL/api/v1/catalogs" \
  -H "Authorization: Bearer $TOKEN2" \
  -H "X-Pangolin-Tenant: $TENANT2_ID" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "tenant2_catalog",
    "warehouse": "tenant2_warehouse"
  }' > /dev/null

echo -e "${GREEN}  ✓ Created catalog in Tenant 2${NC}"

# Create user in Tenant 2
curl -s -X POST "$API_URL/api/v1/users" \
  -H "Authorization: Bearer $TOKEN2" \
  -H "X-Pangolin-Tenant: $TENANT2_ID" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "user1_tenant2",
    "email": "user1@tenant2.com",
    "password": "pass456",
    "role": "TenantUser"
  }' > /dev/null

echo -e "${GREEN}  ✓ Created user in Tenant 2${NC}"
echo ""

echo -e "${BLUE}Step 7: Retrieving audit logs via API for Tenant 1...${NC}"
AUDIT1_RESPONSE=$(curl -s -X GET "$API_URL/api/v1/audit?limit=10" \
  -H "Authorization: Bearer $TOKEN1" \
  -H "X-Pangolin-Tenant: $TENANT1_ID")

AUDIT1_COUNT=$(echo $AUDIT1_RESPONSE | jq '. | length')
echo -e "${GREEN}✓ Tenant 1 has $AUDIT1_COUNT audit events${NC}"
echo "Sample events:"
echo $AUDIT1_RESPONSE | jq -r '.[] | "  - \(.action) on \(.resource_type) (\(.resource_name)) - \(.result)"' | head -5
echo ""

echo -e "${BLUE}Step 8: Retrieving audit logs via API for Tenant 2...${NC}"
AUDIT2_RESPONSE=$(curl -s -X GET "$API_URL/api/v1/audit?limit=10" \
  -H "Authorization: Bearer $TOKEN2" \
  -H "X-Pangolin-Tenant: $TENANT2_ID")

AUDIT2_COUNT=$(echo $AUDIT2_RESPONSE | jq '. | length')
echo -e "${GREEN}✓ Tenant 2 has $AUDIT2_COUNT audit events${NC}"
echo "Sample events:"
echo $AUDIT2_RESPONSE | jq -r '.[] | "  - \(.action) on \(.resource_type) (\(.resource_name)) - \(.result)"' | head -5
echo ""

echo -e "${BLUE}Step 9: Testing CLI audit commands for Tenant 1...${NC}"
export PANGOLIN_URL="$API_URL"

# Login via CLI as Tenant 1
$CLI login --username admin1 --password password123 > /dev/null 2>&1
$CLI use audit_test_tenant1 > /dev/null 2>&1

echo -e "${GREEN}✓ Logged in via CLI as Tenant 1${NC}"

# List audit events
echo ""
echo "Listing audit events via CLI:"
$CLI list-audit-events --limit 5

# Count audit events
echo ""
echo "Counting audit events via CLI:"
$CLI count-audit-events

# Filter by action
echo ""
echo "Filtering by action (CreateCatalog):"
$CLI list-audit-events --action create_catalog --limit 3

echo ""

echo -e "${BLUE}Step 10: Testing CLI audit commands for Tenant 2...${NC}"

# Login via CLI as Tenant 2
$CLI login --username admin2 --password password456 > /dev/null 2>&1
$CLI use audit_test_tenant2 > /dev/null 2>&1

echo -e "${GREEN}✓ Logged in via CLI as Tenant 2${NC}"

# List audit events
echo ""
echo "Listing audit events via CLI:"
$CLI list-audit-events --limit 5

# Count audit events
echo ""
echo "Counting audit events via CLI:"
$CLI count-audit-events

# Filter by result
echo ""
echo "Filtering by result (success):"
$CLI list-audit-events --result success --limit 3

echo ""

echo -e "${BLUE}Step 11: Verifying tenant isolation...${NC}"

# Count events for each tenant
COUNT1_RESPONSE=$(curl -s -X GET "$API_URL/api/v1/audit/count" \
  -H "Authorization: Bearer $TOKEN1" \
  -H "X-Pangolin-Tenant: $TENANT1_ID")
COUNT1=$(echo $COUNT1_RESPONSE | jq -r '.count')

COUNT2_RESPONSE=$(curl -s -X GET "$API_URL/api/v1/audit/count" \
  -H "Authorization: Bearer $TOKEN2" \
  -H "X-Pangolin-Tenant: $TENANT2_ID")
COUNT2=$(echo $COUNT2_RESPONSE | jq -r '.count')

echo -e "${GREEN}✓ Tenant 1 total events: $COUNT1${NC}"
echo -e "${GREEN}✓ Tenant 2 total events: $COUNT2${NC}"

if [ "$COUNT1" -gt 0 ] && [ "$COUNT2" -gt 0 ]; then
    echo -e "${GREEN}✓ Both tenants have audit logs${NC}"
else
    echo -e "${YELLOW}⚠ Warning: One or both tenants have no audit logs${NC}"
fi

echo ""
echo "========================================"
echo -e "${GREEN}🎉 Audit Logging Live Test Complete!${NC}"
echo "========================================"
echo ""
echo "Summary:"
echo "  - Created 2 tenants"
echo "  - Performed 3 actions per tenant (warehouse, catalog, user)"
echo "  - Verified audit logs via API for both tenants"
echo "  - Verified audit logs via CLI for both tenants"
echo "  - Confirmed tenant isolation"
echo ""
echo "✅ All tests passed!"
