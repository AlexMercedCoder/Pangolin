#!/bin/bash

# Configuration
API_URL="http://localhost:8080/api/v1"
ADMIN_USER="admin"
ADMIN_PASS="password"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Seeding Pangolin Data...${NC}"

# 1. Login
echo "Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/users/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"$ADMIN_USER\", \"password\": \"$ADMIN_PASS\"}")

TOKEN=$(echo $LOGIN_RESPONSE | grep -o '"token":"[^"]*' | cut -d'"' -f4)

if [ -z "$TOKEN" ]; then
    echo -e "${RED}Failed to login. Check server status and credentials.${NC}"
    echo "Response: $LOGIN_RESPONSE"
    exit 1
fi

echo -e "${GREEN}Logged in. Token received.${NC}"

# 2. Create Tenant
echo "Creating Tenant 'demo-tenant'..."
RESPONSE=$(curl -s -X POST "$API_URL/tenants" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "demo-tenant",
    "description": "A demo tenant for verification",
    "properties": {"env": "dev"}
  }')

if echo "$RESPONSE" | grep -q "name"; then
    echo -e "${GREEN}Tenant created.${NC}"
else
    echo -e "${RED}Failed to create tenant.${NC}"
    echo "Response: $RESPONSE"
fi

# 3. Create Warehouse
echo "Creating Warehouse 'demo-warehouse'..."
RESPONSE=$(curl -s -X POST "$API_URL/warehouses" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "demo-warehouse",
    "use_sts": false,
    "storage_config": {
      "type": "s3",
      "bucket": "demo-bucket",
      "region": "us-east-1",
      "endpoint": "http://minio:9000",
      "access_key": "minioadmin",
      "secret_key": "minioadmin"
    }
  }')

if echo "$RESPONSE" | grep -q "name"; then
    echo -e "${GREEN}Warehouse created.${NC}"
else
    echo -e "${RED}Failed to create warehouse.${NC}"
    echo "Response: $RESPONSE"
fi

# 4. Create Catalog
echo "Creating Catalog 'demo-catalog'..."
RESPONSE=$(curl -s -X POST "$API_URL/catalogs" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "demo-catalog",
    "warehouse_name": "demo-warehouse",
    "storage_location": "s3://demo-bucket/demo-catalog",
    "properties": {}
  }')

if echo "$RESPONSE" | grep -q "name"; then
    echo -e "${GREEN}Catalog created.${NC}"
else
    echo -e "${RED}Failed to create catalog.${NC}"
    echo "Response: $RESPONSE"
fi

echo -e "${GREEN}Seeding complete!${NC}"
