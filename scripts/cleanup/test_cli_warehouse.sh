#!/bin/bash
set -e

echo "=== CLI Warehouse Creation Test ==="
echo ""

# Configuration
export PANGOLIN_URL="http://localhost:8080"
CLI="./target/debug/pangolin-admin"

echo "1. Creating test tenant..."
$CLI create-tenant --name cli-test-tenant --admin-username cli-admin --admin-password testpass123

echo ""
echo "2. Logging in as tenant admin..."
$CLI login --username cli-admin --password testpass123

echo ""
echo "3. Creating warehouse via CLI with correct property names..."
$CLI create-warehouse cli-test-warehouse \
  --type s3 \
  --bucket cli-test-bucket \
  --access-key minioadmin \
  --secret-key minioadmin \
  --region us-east-1 \
  --endpoint http://localhost:9000

echo ""
echo "4. Listing warehouses..."
$CLI list-warehouses

echo ""
echo "5. Creating catalog..."
$CLI create-catalog cli-test-catalog --warehouse cli-test-warehouse

echo ""
echo "6. Listing catalogs..."
$CLI list-catalogs

echo ""
echo "✅ CLI warehouse creation test completed!"
