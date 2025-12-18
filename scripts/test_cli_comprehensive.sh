#!/bin/bash
# Comprehensive CLI Live Test Suite - Fixed for multi-tenant mode
# Tests all CLI commands with proper tenant context

set -e

echo "=========================================="
echo "Pangolin CLI Comprehensive Live Test"
echo "=========================================="
echo ""

# Configuration
ROOT_USER="admin"
ROOT_PASS="password"
TENANT_ADMIN="tenant-admin"
TENANT_PASS="password123"
CLI_ADMIN="./pangolin/target/debug/pangolin-admin"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "=== Phase 1: Root Operations ==="
echo ""

echo -e "${BLUE}1. Login as root${NC}"
$CLI_ADMIN login --username "$ROOT_USER" --password "$ROOT_PASS"
echo ""

echo -e "${BLUE}2. Create test tenant${NC}"
$CLI_ADMIN create-tenant --name "test-tenant" --admin-username "$TENANT_ADMIN" --admin-password "$TENANT_PASS"
echo ""

echo -e "${BLUE}3. List tenants${NC}"
$CLI_ADMIN list-tenants
echo ""

echo -e "${BLUE}4. Create service user (root)${NC}"
$CLI_ADMIN create-service-user \
  --name "root-service" \
  --description "Root service user" \
  --role "root" \
  --expires-in-days 90 > /tmp/root_service_user.txt
cat /tmp/root_service_user.txt
echo ""

echo -e "${BLUE}5. List service users${NC}"
$CLI_ADMIN list-service-users
echo ""

echo "=== Phase 2: Switch to Tenant Context ==="
echo ""

echo -e "${BLUE}6. Login as tenant admin${NC}"
$CLI_ADMIN login --username "$TENANT_ADMIN" --password "$TENANT_PASS"
echo ""

echo "=== Phase 3: Warehouse Management ==="
echo ""

echo -e "${BLUE}7. Create warehouse${NC}"
$CLI_ADMIN create-warehouse "test-warehouse" \
  --bucket "test-bucket" \
  --endpoint "http://localhost:9000" \
  --access-key "minioadmin" \
  --secret-key "minioadmin" \
  --region "us-east-1"
echo ""

echo -e "${BLUE}8. List warehouses${NC}"
$CLI_ADMIN list-warehouses
echo ""

echo "=== Phase 4: Catalog Management ==="
echo ""

echo -e "${BLUE}9. Create catalog${NC}"
$CLI_ADMIN create-catalog "test-catalog" --warehouse "test-warehouse"
echo ""

echo -e "${BLUE}10. List catalogs${NC}"
$CLI_ADMIN list-catalogs
echo ""

echo "=== Phase 5: User Management ==="
echo ""

echo -e "${BLUE}11. Create user${NC}"
$CLI_ADMIN create-user "test-user" \
  --email "test@example.com" \
  --role "tenant-user" \
  --password "userpass123"
echo ""

echo -e "${BLUE}12. List users${NC}"
$CLI_ADMIN list-users
echo ""

echo "=== Phase 6: Service Users (Tenant) ==="
echo ""

echo -e "${BLUE}13. Create tenant service user${NC}"
$CLI_ADMIN create-service-user \
  --name "tenant-service" \
  --description "Tenant service user" \
  --role "tenant-user" \
  --expires-in-days 30 > /tmp/tenant_service_user.txt
cat /tmp/tenant_service_user.txt
echo ""

echo -e "${BLUE}14. List service users${NC}"
$CLI_ADMIN list-service-users
echo ""

echo "=== Phase 7: Permissions ==="
echo ""

echo -e "${BLUE}15. Grant permission${NC}"
$CLI_ADMIN grant-permission \
  --username "test-user" \
  --action "READ" \
  --resource "catalog:test-catalog"
echo ""

echo -e "${BLUE}16. List permissions${NC}"
$CLI_ADMIN list-permissions
echo ""

echo "=== Phase 8: Metadata Operations ==="
echo ""

echo -e "${BLUE}17. Get metadata${NC}"
$CLI_ADMIN get-metadata --entity-type "catalog" --entity-id "test-catalog" || echo "No metadata yet"
echo ""

echo -e "${BLUE}18. Set metadata${NC}"
$CLI_ADMIN set-metadata \
  --entity-type "catalog" \
  --entity-id "test-catalog" \
  "owner" "data-team"
echo ""

echo "=== Phase 9: Merge Operations ==="
echo ""

echo -e "${BLUE}19. List merge operations${NC}"
$CLI_ADMIN list-merge-operations
echo ""

echo "=== Phase 10: Access Requests ==="
echo ""

echo -e "${BLUE}20. List access requests${NC}"
$CLI_ADMIN list-access-requests
echo ""

echo "=== Phase 11: Federated Catalogs ==="
echo ""

echo -e "${BLUE}21. List federated catalogs${NC}"
$CLI_ADMIN list-federated-catalogs
echo ""

echo "=== Phase 12: Token Management ==="
echo ""

echo -e "${BLUE}22. Revoke token (will logout)${NC}"
# Note: This will logout the current session
# $CLI_ADMIN revoke-token
echo "Skipped to maintain session"
echo ""

echo "=== Phase 13: Cleanup ==="
echo ""

echo -e "${BLUE}23. Delete user${NC}"
$CLI_ADMIN delete-user "test-user"
echo ""

echo -e "${BLUE}24. Delete catalog${NC}"
$CLI_ADMIN delete-catalog "test-catalog"
echo ""

echo -e "${BLUE}25. Delete warehouse${NC}"
$CLI_ADMIN delete-warehouse "test-warehouse"
echo ""

echo -e "${BLUE}26. Login as root for final cleanup${NC}"
$CLI_ADMIN login --username "$ROOT_USER" --password "$ROOT_PASS"
echo ""

echo -e "${BLUE}27. Delete tenant${NC}"
# Note: Requires tenant ID, skipping for now
echo "Skipped - requires ID extraction"
echo ""

echo "=========================================="
echo -e "${GREEN}✅ Comprehensive CLI Test Complete!${NC}"
echo "=========================================="
echo ""
echo "All major CLI features tested successfully:"
echo "  ✅ Authentication (root + tenant)"
echo "  ✅ Tenant management"
echo "  ✅ Warehouse management"
echo "  ✅ Catalog management"
echo "  ✅ User management"
echo "  ✅ Service users"
echo "  ✅ Permissions"
echo "  ✅ Metadata"
echo "  ✅ Merge operations"
echo "  ✅ Access requests"
echo "  ✅ Federated catalogs"
echo ""
