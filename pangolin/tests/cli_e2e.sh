#!/bin/bash
set -e

# Configuration
TEST_PORT=8081
API_URL="http://localhost:$TEST_PORT"
ADMIN_CLI="./target/debug/pangolin-admin"
USER_CLI="./target/debug/pangolin-user"
SERVER_PID=""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

cleanup() {
    if [ -n "$SERVER_PID" ]; then
        echo "Stopping test server (PID: $SERVER_PID)..."
        kill "$SERVER_PID" || true
        wait "$SERVER_PID" 2>/dev/null || true
    fi
}
trap cleanup EXIT

echo "Building Project..."
cargo build -p pangolin_api -p pangolin_cli_admin -p pangolin_cli_user --quiet

echo -e "${GREEN}Starting Test Server on port $TEST_PORT...${NC}"
export PORT=$TEST_PORT
export PANGOLIN_NO_AUTH=false
export PANGOLIN_ROOT_USER="admin"
export PANGOLIN_ROOT_PASSWORD="password"
export PANGOLIN_STORAGE_TYPE="memory"

cargo run -p pangolin_api --quiet &
SERVER_PID=$!
echo "Server PID: $SERVER_PID"

# Wait for server to be ready
echo "Waiting for server to be ready..."
for i in {1..30}; do
    if curl -s "$API_URL/health" >/dev/null; then
        echo "Server is up!"
        break
    fi
    sleep 1
done

if ! curl -s "$API_URL/health" >/dev/null; then
    echo -e "${RED}Server failed to start${NC}"
    exit 1
fi

echo -e "${GREEN}Starting CLI E2E Tests against $API_URL${NC}"

# 1. Admin: Login
echo "------------------------------------------------"
echo "1. Admin: Login (root)"
$ADMIN_CLI --url "$API_URL" login --username admin --password password

# 2. Admin: Create Tenant with Initial Admin
TENANT_NAME="CliTestTenant_$(date +%s)"
ADMIN_USER="tenant_admin"
ADMIN_PASS="tenant_pass_123"

echo "------------------------------------------------"
echo "2. Admin: Create Tenant '$TENANT_NAME' with Admin '$ADMIN_USER'"
$ADMIN_CLI create-tenant --name "$TENANT_NAME" \
    --admin-username "$ADMIN_USER" \
    --admin-password "$ADMIN_PASS"

# 2.5. Switch to Tenant Admin Identity
echo "------------------------------------------------"
echo "2.5. Login as Tenant Admin '$ADMIN_USER'"
# Login as the new user.
# Note: login command generally infers tenant from username if unique or needs tenant context.
# But passing --url is safe.
# Login should set context to that user's tenant automatically (impl check pending, but likely).
$ADMIN_CLI --url "$API_URL" login --username "$ADMIN_USER" --password "$ADMIN_PASS"

# 3. Tenant Admin: Create Warehouse
WH_NAME="cli_wh_$(date +%s)"
echo "------------------------------------------------"
echo "3. Admin: Create Warehouse '$WH_NAME'"
$ADMIN_CLI create-warehouse "$WH_NAME" \
    --type s3 \
    --bucket "my-bucket" \
    --region "us-east-1" \
    --endpoint "http://localhost:9000" \
    --access-key "minio" \
    --secret-key "minio123"

# 4. Tenant Admin: Create Catalog
CAT_NAME="cli_cat_$(date +%s)"
echo "------------------------------------------------"
echo "4. Admin: Create Catalog '$CAT_NAME'"
$ADMIN_CLI create-catalog "$CAT_NAME" --warehouse "$WH_NAME"

# 5. User CLI: Login
echo "------------------------------------------------"
echo "5. User: Login"
$USER_CLI --url "$API_URL" login --username admin --password password

# 6. User CLI: List Catalogs
echo "------------------------------------------------"
echo "6. User: List Catalogs"
OUTPUT=$($USER_CLI list-catalogs)
echo "$OUTPUT"
if [[ "$OUTPUT" == *"$CAT_NAME"* ]]; then
    echo -e "${GREEN}✓ Catalog found${NC}"
else
    echo -e "${RED}✗ Catalog not found${NC}"
    exit 1
fi

echo "------------------------------------------------"
echo -e "${GREEN}All CLI E2E tests passed!${NC}"
