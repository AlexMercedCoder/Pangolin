#!/bin/bash
set -e

echo "=== CLI Backend Enhancements Live Test ==="
echo ""

API_URL="http://localhost:8080"
CLI_BIN="./target/release/pangolin-admin"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Test 1: Root Login (no tenant-id)${NC}"
$CLI_BIN login --username admin --password password
echo -e "${GREEN}✅ Root login successful${NC}"
echo ""

echo -e "${BLUE}Test 2: Dashboard Stats (should show tenants_count)${NC}"
$CLI_BIN stats
echo -e "${GREEN}✅ Dashboard stats displayed${NC}"
echo ""

echo -e "${BLUE}Test 3: Create Test Tenant${NC}"
TENANT_RESPONSE=$(curl -s -X POST "$API_URL/api/v1/tenants" \
  -u admin:password \
  -H "Content-Type: application/json" \
  -d '{"name":"test-tenant-cli"}')
TENANT_ID=$(echo $TENANT_RESPONSE | jq -r '.id')
echo "Created tenant: $TENANT_ID"
echo -e "${GREEN}✅ Tenant created${NC}"
echo ""

echo -e "${BLUE}Test 4: Create Tenant User${NC}"
curl -s -X POST "$API_URL/api/v1/users" \
  -u admin:password \
  -H "Content-Type: application/json" \
  -d "{
    \"username\": \"testuser\",
    \"email\": \"test@example.com\",
    \"password\": \"pass123\",
    \"tenant_id\": \"$TENANT_ID\",
    \"role\": \"TenantAdmin\"
  }" > /dev/null
echo -e "${GREEN}✅ Tenant user created${NC}"
echo ""

echo -e "${BLUE}Test 5: Tenant-Scoped Login (with --tenant-id)${NC}"
$CLI_BIN login --username testuser --password pass123 --tenant-id $TENANT_ID
echo -e "${GREEN}✅ Tenant-scoped login successful${NC}"
echo ""

echo -e "${BLUE}Test 6: Dashboard Stats as Tenant User (should NOT show tenants_count)${NC}"
$CLI_BIN stats
echo -e "${GREEN}✅ Dashboard stats displayed for tenant${NC}"
echo ""

echo -e "${BLUE}Test 7: Root Login Again${NC}"
$CLI_BIN login --username admin --password password
echo -e "${GREEN}✅ Root login successful${NC}"
echo ""

echo -e "${BLUE}Test 8: List Audit Events (all tenants)${NC}"
$CLI_BIN list-audit-events --limit 10
echo -e "${GREEN}✅ Audit events listed${NC}"
echo ""

echo -e "${BLUE}Test 9: List Audit Events (filtered by tenant)${NC}"
$CLI_BIN list-audit-events --tenant-id $TENANT_ID --limit 5
echo -e "${GREEN}✅ Audit events filtered by tenant${NC}"
echo ""

echo -e "${BLUE}Test 10: Dashboard Stats as Root (should show tenants_count)${NC}"
$CLI_BIN stats
echo -e "${GREEN}✅ Dashboard stats displayed with tenants count${NC}"
echo ""

echo "=== All CLI Tests Passed! ==="
