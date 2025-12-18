#!/bin/bash
# Test script for update commands

set -e

echo "==================================="
echo "CLI Update Commands Test"
echo "==================================="
echo ""

# Configuration
ADMIN_USER="admin"
ADMIN_PASS="password"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Step 1: Login as admin${NC}"
./pangolin/target/debug/pangolin-admin login --username "$ADMIN_USER" --password "$ADMIN_PASS"
echo ""

echo -e "${BLUE}Step 2: Create test tenant${NC}"
./pangolin/target/debug/pangolin-admin create-tenant --name "test-update-tenant"
TENANT_LIST=$(./pangolin/target/debug/pangolin-admin list-tenants)
TENANT_ID=$(echo "$TENANT_LIST" | grep "test-update-tenant" | awk '{print $1}')
echo "Created tenant ID: $TENANT_ID"
echo ""

echo -e "${BLUE}Step 3: Update tenant name${NC}"
./pangolin/target/debug/pangolin-admin update-tenant --id "$TENANT_ID" --name "updated-tenant-name"
echo ""

echo -e "${BLUE}Step 4: Verify tenant update${NC}"
./pangolin/target/debug/pangolin-admin list-tenants | grep "updated-tenant-name"
echo ""

echo -e "${BLUE}Step 5: Create test warehouse${NC}"
./pangolin/target/debug/pangolin-admin create-warehouse "test-warehouse" --bucket "test-bucket"
WAREHOUSE_LIST=$(./pangolin/target/debug/pangolin-admin list-warehouses)
WAREHOUSE_ID=$(echo "$WAREHOUSE_LIST" | grep "test-warehouse" | awk '{print $1}')
echo "Created warehouse ID: $WAREHOUSE_ID"
echo ""

echo -e "${BLUE}Step 6: Update warehouse name${NC}"
./pangolin/target/debug/pangolin-admin update-warehouse --id "$WAREHOUSE_ID" --name "updated-warehouse"
echo ""

echo -e "${BLUE}Step 7: Verify warehouse update${NC}"
./pangolin/target/debug/pangolin-admin list-warehouses | grep "updated-warehouse"
echo ""

echo -e "${BLUE}Step 8: Create test catalog${NC}"
./pangolin/target/debug/pangolin-admin create-catalog "test-catalog" --warehouse "updated-warehouse"
CATALOG_LIST=$(./pangolin/target/debug/pangolin-admin list-catalogs)
CATALOG_ID=$(echo "$CATALOG_LIST" | grep "test-catalog" | awk '{print $1}')
echo "Created catalog ID: $CATALOG_ID"
echo ""

echo -e "${BLUE}Step 9: Update catalog name${NC}"
./pangolin/target/debug/pangolin-admin update-catalog --id "$CATALOG_ID" --name "updated-catalog"
echo ""

echo -e "${BLUE}Step 10: Verify catalog update${NC}"
./pangolin/target/debug/pangolin-admin list-catalogs | grep "updated-catalog"
echo ""

echo -e "${BLUE}Step 11: Cleanup${NC}"
./pangolin/target/debug/pangolin-admin delete-catalog "updated-catalog"
./pangolin/target/debug/pangolin-admin delete-warehouse "updated-warehouse"
./pangolin/target/debug/pangolin-admin delete-tenant "$TENANT_ID"
echo ""

echo -e "${GREEN}==================================="
echo "All update command tests passed!"
echo "===================================${NC}"
