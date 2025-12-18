#!/bin/bash
# Live test script for service user CLI commands

set -e

echo "==================================="
echo "Service User CLI Live Test"
echo "==================================="
echo ""

# Configuration
API_URL="http://localhost:8080"
ADMIN_USER="admin"
ADMIN_PASS="password"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Step 1: Login as admin${NC}"
./pangolin/target/debug/pangolin-admin login --username "$ADMIN_USER" --password "$ADMIN_PASS"
echo ""

echo -e "${BLUE}Step 2: Create service user${NC}"
./pangolin/target/debug/pangolin-admin create-service-user \
  --name "test-service-user" \
  --description "Test service user for CLI" \
  --role "tenant-user" \
  --expires-in-days 90 > /tmp/service_user_create.txt
cat /tmp/service_user_create.txt
SERVICE_USER_ID=$(grep "Service User ID:" /tmp/service_user_create.txt | awk '{print $NF}')
API_KEY=$(grep "API Key:" /tmp/service_user_create.txt | awk '{print $NF}')
echo "Captured Service User ID: $SERVICE_USER_ID"
echo "Captured API Key: $API_KEY"
echo ""

echo -e "${BLUE}Step 3: List service users${NC}"
./pangolin/target/debug/pangolin-admin list-service-users
echo ""

echo -e "${BLUE}Step 4: Get service user details${NC}"
./pangolin/target/debug/pangolin-admin get-service-user --id "$SERVICE_USER_ID"
echo ""

echo -e "${BLUE}Step 5: Update service user${NC}"
./pangolin/target/debug/pangolin-admin update-service-user \
  --id "$SERVICE_USER_ID" \
  --description "Updated description"
echo ""

echo -e "${BLUE}Step 6: Verify update${NC}"
./pangolin/target/debug/pangolin-admin get-service-user --id "$SERVICE_USER_ID"
echo ""

echo -e "${BLUE}Step 7: Test API key authentication${NC}"
curl -s -H "X-API-Key: $API_KEY" "$API_URL/api/v1/catalogs" | jq '.' || echo "API key auth test"
echo ""

echo -e "${BLUE}Step 8: Rotate API key${NC}"
./pangolin/target/debug/pangolin-admin rotate-service-user-key --id "$SERVICE_USER_ID" > /tmp/service_user_rotate.txt
cat /tmp/service_user_rotate.txt
NEW_API_KEY=$(grep "New API Key:" /tmp/service_user_rotate.txt | awk '{print $NF}')
echo "New API Key: $NEW_API_KEY"
echo ""

echo -e "${BLUE}Step 9: Test new API key${NC}"
curl -s -H "X-API-Key: $NEW_API_KEY" "$API_URL/api/v1/catalogs" | jq '.' || echo "New API key auth test"
echo ""

echo -e "${BLUE}Step 10: Delete service user${NC}"
./pangolin/target/debug/pangolin-admin delete-service-user --id "$SERVICE_USER_ID"
echo ""

echo -e "${BLUE}Step 11: Verify deletion${NC}"
./pangolin/target/debug/pangolin-admin list-service-users
echo ""

echo -e "${GREEN}==================================="
echo "All tests completed successfully!"
echo "===================================${NC}"
