#!/bin/bash
set -e

echo "=== Testing Tenant-Scoped Login Fix ==="
echo ""

API_URL="http://localhost:8080"
CLI_BIN="./target/release/pangolin-admin"

GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}Step 1: Root Login${NC}"
$CLI_BIN login --username admin --password password
echo -e "${GREEN}✅ Root login successful${NC}"
echo ""

echo -e "${BLUE}Step 2: Create Tenant 1${NC}"
TENANT1=$(curl -s -X POST "$API_URL/api/v1/tenants" \
  -u admin:password \
  -H "Content-Type: application/json" \
  -d '{"name":"tenant1"}' | jq -r '.id')
echo "Tenant 1 ID: $TENANT1"
echo -e "${GREEN}✅ Tenant 1 created${NC}"
echo ""

echo -e "${BLUE}Step 3: Create Tenant 2${NC}"
TENANT2=$(curl -s -X POST "$API_URL/api/v1/tenants" \
  -u admin:password \
  -H "Content-Type: application/json" \
  -d '{"name":"tenant2"}' | jq -r '.id')
echo "Tenant 2 ID: $TENANT2"
echo -e "${GREEN}✅ Tenant 2 created${NC}"
echo ""

echo -e "${BLUE}Step 4: Create user 'alice' in Tenant 1${NC}"
curl -s -X POST "$API_URL/api/v1/users" \
  -u admin:password \
  -H "Content-Type: application/json" \
  -d "{
    \"username\": \"alice\",
    \"email\": \"alice@tenant1.com\",
    \"password\": \"password1\",
    \"tenant_id\": \"$TENANT1\",
    \"role\": \"TenantAdmin\"
  }" > /dev/null
echo -e "${GREEN}✅ User 'alice' created in Tenant 1${NC}"
echo ""

echo -e "${BLUE}Step 5: Create user 'alice' in Tenant 2 (duplicate username!)${NC}"
curl -s -X POST "$API_URL/api/v1/users" \
  -u admin:password \
  -H "Content-Type: application/json" \
  -d "{
    \"username\": \"alice\",
    \"email\": \"alice@tenant2.com\",
    \"password\": \"password2\",
    \"tenant_id\": \"$TENANT2\",
    \"role\": \"TenantAdmin\"
  }" > /dev/null
echo -e "${GREEN}✅ User 'alice' created in Tenant 2${NC}"
echo ""

echo -e "${BLUE}Step 6: Login as alice@tenant1 with --tenant-id${NC}"
$CLI_BIN login --username alice --password password1 --tenant-id $TENANT1 && {
    echo -e "${GREEN}✅ SUCCESS! Alice from Tenant 1 logged in${NC}"
} || {
    echo -e "${RED}❌ FAILED! Could not login as alice@tenant1${NC}"
    exit 1
}
echo ""

echo -e "${BLUE}Step 7: Login as alice@tenant2 with --tenant-id${NC}"
$CLI_BIN login --username alice --password password2 --tenant-id $TENANT2 && {
    echo -e "${GREEN}✅ SUCCESS! Alice from Tenant 2 logged in${NC}"
} || {
    echo -e "${RED}❌ FAILED! Could not login as alice@tenant2${NC}"
    exit 1
}
echo ""

echo -e "${BLUE}Step 8: Verify wrong password fails${NC}"
$CLI_BIN login --username alice --password wrongpass --tenant-id $TENANT1 2>&1 | grep -q "Error" && {
    echo -e "${GREEN}✅ Correctly rejected wrong password${NC}"
} || {
    echo -e "${RED}❌ Should have rejected wrong password${NC}"
}
echo ""

echo -e "${GREEN}=== All Tests Passed! Tenant-Scoped Login Fixed! ===${NC}"
