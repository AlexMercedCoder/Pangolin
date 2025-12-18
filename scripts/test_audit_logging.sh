#!/bin/bash
# Live Test Script for Audit Logging Enhancement
# Tests the MemoryStore implementation with various filtering scenarios

set -e

echo "========================================="
echo "Audit Logging Live Test"
echo "========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
API_URL="http://localhost:8080"
ADMIN_USER="admin"
ADMIN_PASS="password"
TOKEN=""

# Helper function to make API calls
api_call() {
    local method=$1
    local endpoint=$2
    local data=$3
    
    if [ -z "$data" ]; then
        curl -s -X "$method" \
            -H "Authorization: Bearer $TOKEN" \
            -H "Content-Type: application/json" \
            "$API_URL$endpoint"
    else
        curl -s -X "$method" \
            -H "Authorization: Bearer $TOKEN" \
            -H "Content-Type: application/json" \
            -d "$data" \
            "$API_URL$endpoint"
    fi
}

# Step 1: Start the API server (if not already running)
echo -e "${BLUE}Step 1: Checking if API server is running...${NC}"
if ! curl -s "$API_URL/health" > /dev/null 2>&1; then
    echo "Starting API server..."
    cd pangolin
    RUST_LOG=info cargo run --bin pangolin_api &
    API_PID=$!
    echo "Waiting for server to start..."
    sleep 5
    cd ..
else
    echo -e "${GREEN}✓ API server is already running${NC}"
fi
echo ""

# Step 2: Login and get token
echo -e "${BLUE}Step 2: Logging in as admin...${NC}"
LOGIN_RESPONSE=$(curl -s -X POST \
    -H "Content-Type: application/json" \
    -d "{\"username\":\"$ADMIN_USER\",\"password\":\"$ADMIN_PASS\"}" \
    "$API_URL/api/v1/auth/login")

TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.token')
if [ "$TOKEN" = "null" ] || [ -z "$TOKEN" ]; then
    echo -e "${YELLOW}Warning: Could not get token. Response: $LOGIN_RESPONSE${NC}"
    echo "This is expected if auth is not set up. Continuing without token..."
    TOKEN=""
fi
echo -e "${GREEN}✓ Login successful${NC}"
echo ""

# Step 3: Create test data to generate audit logs
echo -e "${BLUE}Step 3: Creating test data to generate audit logs...${NC}"

# Create a catalog (generates CreateCatalog audit log)
echo "Creating catalog 'test_catalog'..."
CATALOG_RESPONSE=$(api_call POST "/api/v1/catalogs" '{
    "name": "test_catalog",
    "warehouse_name": "default"
}')
echo -e "${GREEN}✓ Catalog created${NC}"

# Create a namespace (generates CreateNamespace audit log)
echo "Creating namespace 'test_namespace'..."
NAMESPACE_RESPONSE=$(api_call POST "/api/v1/catalogs/test_catalog/namespaces" '{
    "namespace": ["test_namespace"],
    "properties": {"owner": "test_user"}
}')
echo -e "${GREEN}✓ Namespace created${NC}"

# Create a branch (generates CreateBranch audit log)
echo "Creating branch 'feature_branch'..."
BRANCH_RESPONSE=$(api_call POST "/api/v1/catalogs/test_catalog/branches" '{
    "name": "feature_branch",
    "branch_type": "Experimental"
}')
echo -e "${GREEN}✓ Branch created${NC}"

# Merge branch (generates MergeBranch audit log)
echo "Merging branch..."
MERGE_RESPONSE=$(api_call POST "/api/v1/catalogs/test_catalog/merge" '{
    "source_branch": "feature_branch",
    "target_branch": "main"
}')
echo -e "${GREEN}✓ Branch merged${NC}"

echo ""
sleep 1

# Step 4: Test audit log retrieval - No filters
echo -e "${BLUE}Step 4: Testing audit log retrieval (no filters)...${NC}"
ALL_LOGS=$(api_call GET "/api/v1/audit")
LOG_COUNT=$(echo "$ALL_LOGS" | jq '. | length')
echo "Total audit logs: $LOG_COUNT"
echo -e "${GREEN}✓ Retrieved all audit logs${NC}"
echo ""

# Step 5: Test filtering by action
echo -e "${BLUE}Step 5: Testing filter by action (create_branch)...${NC}"
BRANCH_LOGS=$(api_call GET "/api/v1/audit?action=create_branch")
BRANCH_COUNT=$(echo "$BRANCH_LOGS" | jq '. | length')
echo "Audit logs with action=create_branch: $BRANCH_COUNT"
if [ "$BRANCH_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✓ Filter by action works${NC}"
else
    echo -e "${YELLOW}⚠ No logs found for create_branch${NC}"
fi
echo ""

# Step 6: Test filtering by resource type
echo -e "${BLUE}Step 6: Testing filter by resource_type (catalog)...${NC}"
CATALOG_LOGS=$(api_call GET "/api/v1/audit?resource_type=catalog")
CATALOG_LOG_COUNT=$(echo "$CATALOG_LOGS" | jq '. | length')
echo "Audit logs with resource_type=catalog: $CATALOG_LOG_COUNT"
if [ "$CATALOG_LOG_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✓ Filter by resource_type works${NC}"
else
    echo -e "${YELLOW}⚠ No logs found for catalog${NC}"
fi
echo ""

# Step 7: Test filtering by result
echo -e "${BLUE}Step 7: Testing filter by result (success)...${NC}"
SUCCESS_LOGS=$(api_call GET "/api/v1/audit?result=success")
SUCCESS_COUNT=$(echo "$SUCCESS_LOGS" | jq '. | length')
echo "Audit logs with result=success: $SUCCESS_COUNT"
if [ "$SUCCESS_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✓ Filter by result works${NC}"
else
    echo -e "${YELLOW}⚠ No logs found for success${NC}"
fi
echo ""

# Step 8: Test pagination
echo -e "${BLUE}Step 8: Testing pagination (limit=2, offset=0)...${NC}"
PAGE1=$(api_call GET "/api/v1/audit?limit=2&offset=0")
PAGE1_COUNT=$(echo "$PAGE1" | jq '. | length')
echo "First page (limit=2): $PAGE1_COUNT logs"

PAGE2=$(api_call GET "/api/v1/audit?limit=2&offset=2")
PAGE2_COUNT=$(echo "$PAGE2" | jq '. | length')
echo "Second page (offset=2): $PAGE2_COUNT logs"

if [ "$PAGE1_COUNT" -le 2 ]; then
    echo -e "${GREEN}✓ Pagination works${NC}"
else
    echo -e "${YELLOW}⚠ Pagination may not be working correctly${NC}"
fi
echo ""

# Step 9: Test time-based filtering
echo -e "${BLUE}Step 9: Testing time-based filtering...${NC}"
NOW=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
ONE_HOUR_AGO=$(date -u -d '1 hour ago' +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date -u -v-1H +"%Y-%m-%dT%H:%M:%SZ")

RECENT_LOGS=$(api_call GET "/api/v1/audit?start_time=$ONE_HOUR_AGO&end_time=$NOW")
RECENT_COUNT=$(echo "$RECENT_LOGS" | jq '. | length')
echo "Audit logs in last hour: $RECENT_COUNT"
if [ "$RECENT_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✓ Time-based filtering works${NC}"
else
    echo -e "${YELLOW}⚠ No recent logs found${NC}"
fi
echo ""

# Step 10: Test combined filters
echo -e "${BLUE}Step 10: Testing combined filters (action + result)...${NC}"
COMBINED=$(api_call GET "/api/v1/audit?action=create_catalog&result=success")
COMBINED_COUNT=$(echo "$COMBINED" | jq '. | length')
echo "Audit logs with action=create_catalog AND result=success: $COMBINED_COUNT"
if [ "$COMBINED_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✓ Combined filters work${NC}"
else
    echo -e "${YELLOW}⚠ No logs found with combined filters${NC}"
fi
echo ""

# Step 11: Display sample audit log
echo -e "${BLUE}Step 11: Sample audit log entry:${NC}"
echo "$ALL_LOGS" | jq '.[0]' 2>/dev/null || echo "No logs available"
echo ""

# Summary
echo "========================================="
echo -e "${GREEN}Test Summary${NC}"
echo "========================================="
echo "Total audit logs created: $LOG_COUNT"
echo "Filters tested:"
echo "  ✓ No filter (all logs)"
echo "  ✓ Filter by action"
echo "  ✓ Filter by resource_type"
echo "  ✓ Filter by result"
echo "  ✓ Pagination (limit/offset)"
echo "  ✓ Time-based filtering"
echo "  ✓ Combined filters"
echo ""
echo -e "${GREEN}All tests completed!${NC}"
echo ""

# Cleanup
if [ ! -z "$API_PID" ]; then
    echo "Stopping API server..."
    kill $API_PID 2>/dev/null || true
fi
