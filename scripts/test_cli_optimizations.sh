#!/bin/bash
# Live test script for CLI optimization commands

set -e

echo "======================================================================="
echo "CLI Optimization Commands Live Test"
echo "======================================================================="

# Configuration
API_URL="http://localhost:8080"
ADMIN_CLI="./pangolin/target/debug/pangolin-admin"

echo ""
echo "=== Test 1: Dashboard Statistics ==="
$ADMIN_CLI stats

echo ""
echo "=== Test 2: Catalog Summary ==="
$ADMIN_CLI catalog-summary test_catalog

echo ""
echo "=== Test 3: Asset Search ==="
$ADMIN_CLI search "test" --limit 10

echo ""
echo "=== Test 4: Asset Search with Catalog Filter ==="
$ADMIN_CLI search "test" --catalog test_catalog --limit 5

echo ""
echo "=== Test 5: Name Validation (Catalogs) ==="
$ADMIN_CLI validate catalog test_catalog new_catalog_123 another_catalog

echo ""
echo "=== Test 6: Name Validation (Warehouses) ==="
$ADMIN_CLI validate warehouse test_warehouse new_warehouse_456

echo ""
echo "=== Test 7: Bulk Delete (Dry Run with Invalid IDs) ==="
$ADMIN_CLI bulk-delete --ids "00000000-0000-0000-0000-000000000001,invalid-uuid" --confirm

echo ""
echo "======================================================================="
echo "✅ All CLI optimization commands tested successfully!"
echo "======================================================================="
