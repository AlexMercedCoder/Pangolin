#!/bin/bash
set -e

# Cleanup function
cleanup() {
    echo "Stopping server..."
    if [ -n "$SERVER_PID" ]; then
        kill $SERVER_PID || true
    fi
}
trap cleanup EXIT

# Clean previous runs
rm -f server.log

# Start Server with MongoStore
echo "Starting Pangolin API (MongoStore)..."
export PANGOLIN_STORAGE_TYPE=mongo
# Use docker mongo on port 27017
export DATABASE_URL=mongodb://localhost:27017/pangolin
export PANGOLIN_NO_AUTH=true
export PANGOLIN_SEED_ADMIN=true
export RUST_LOG=info
cargo run -p pangolin_api --bin pangolin_api > server.log 2>&1 &
SERVER_PID=$!

# Wait for startup
timeout 60 sh -c 'until grep -q "listening on" server.log; do sleep 1; done'

if ! ps -p $SERVER_PID > /dev/null; then
    echo "Server failed to start. Check logs:"
    cat server.log
    exit 1
fi

# Extract Admin Credentials from Log
echo "Extracting credentials..."
TOKEN=$(grep -oP '(?<="token": ")[^"]+' server.log | head -n 1)
TENANT_ID=$(grep -oP '(?<="header.X-Pangolin-Tenant": ")[^"]+' server.log | head -n 1)

if [ -z "$TOKEN" ]; then
    echo "Failed to extract token. Logs:"
    cat server.log
    exit 1
fi

echo "Token: ${TOKEN:0:20}..."
echo "Tenant: $TENANT_ID"
BASE_URL="http://127.0.0.1:8080/api/v1"
AUTH_HEADER="Authorization: Bearer $TOKEN"
TENANT_HEADER="X-Pangolin-Tenant: $TENANT_ID"

echo "---------------------------------------------------"
echo "1. Testing System Config"
echo "GET /config/settings"
curl -s -X GET "$BASE_URL/config/settings" -H "$AUTH_HEADER" -H "$TENANT_HEADER" | jq .

echo "PUT /config/settings"
curl -s -X PUT "$BASE_URL/config/settings" \
    -H "$AUTH_HEADER" -H "$TENANT_HEADER" \
    -H "Content-Type: application/json" \
    -d '{"allow_public_signup": true, "default_user_role": "TenantUser"}' | jq .

echo "---------------------------------------------------"
echo "2. Testing Token Management"
echo "GET /users/me/tokens"
curl -s -X GET "$BASE_URL/users/me/tokens" -H "$AUTH_HEADER" -H "$TENANT_HEADER" | jq .

echo "POST /tokens/rotate"
ROTATED_RESP=$(curl -s -X POST "$BASE_URL/tokens/rotate" -H "$AUTH_HEADER" -H "$TENANT_HEADER")
echo "Rotate Response: $ROTATED_RESP"
NEW_TOKEN=$(echo $ROTATED_RESP | jq -r .token)

if [ "$NEW_TOKEN" != "null" ]; then
    echo "Rotation successful. Updating token..."
    AUTH_HEADER="Authorization: Bearer $NEW_TOKEN"
else
    echo "Rotation failed!"
fi

echo "---------------------------------------------------"
echo "3. Testing Federated Catalog (Mock/Schema)"
echo "POST /federated-catalogs/fed_cat/sync"
curl -s -X POST "$BASE_URL/federated-catalogs/fed_cat/sync" -H "$AUTH_HEADER" -H "$TENANT_HEADER" | jq .

echo "GET /federated-catalogs/fed_cat/stats"
curl -s -X GET "$BASE_URL/federated-catalogs/fed_cat/stats" -H "$AUTH_HEADER" -H "$TENANT_HEADER" | jq .

echo "---------------------------------------------------"
echo "4. Testing Data Explorer (Tree)"
# Create Warehouse & Catalog first
SUFFIX=$RANDOM
echo "Creating Warehouse wh_$SUFFIX..."
curl -s -X POST "$BASE_URL/warehouses" \
    -H "$AUTH_HEADER" -H "$TENANT_HEADER" \
    -H "Content-Type: application/json" \
    -d '{"name": "wh_'"$SUFFIX"'", "bucket": "test-bucket", "region": "us-east-1", "access_key": "x", "secret_key": "y", "storage_type": "s3", "vending_strategy": "Direct"}' | jq .

echo "Creating Catalog cat_$SUFFIX..."
curl -s -X POST "$BASE_URL/catalogs" \
    -H "$AUTH_HEADER" -H "$TENANT_HEADER" \
    -H "Content-Type: application/json" \
    -d '{"name": "cat_'"$SUFFIX"'", "type": "iceberg", "warehouse_name": "wh_'"$SUFFIX"'"}' | jq .

echo "Creating Namespace..."
curl -s -X POST "http://127.0.0.1:8080/v1/cat_$SUFFIX/namespaces" \
    -H "$AUTH_HEADER" -H "$TENANT_HEADER" \
    -H "Content-Type: application/json" \
    -d '{"namespace": ["sales"]}' | jq .

echo "GET /catalogs/cat_$SUFFIX/namespaces/tree"
curl -s -X GET "$BASE_URL/catalogs/cat_$SUFFIX/namespaces/tree" -H "$AUTH_HEADER" -H "$TENANT_HEADER" | jq .

echo "---------------------------------------------------"
echo "5. Testing Branching (Rebase)"
echo "Creating 'main' branch in cat_$SUFFIX..."
curl -s -X POST "$BASE_URL/branches" \
    -H "$AUTH_HEADER" -H "$TENANT_HEADER" \
    -H "Content-Type: application/json" \
    -d '{"name": "main", "catalog": "cat_'"$SUFFIX"'", "branch_type": "experimental"}' | jq .

echo "Creating 'dev' branch from 'main'..."
curl -s -X POST "$BASE_URL/branches" \
    -H "$AUTH_HEADER" -H "$TENANT_HEADER" \
    -H "Content-Type: application/json" \
    -d '{"name": "dev", "catalog": "cat_'"$SUFFIX"'", "from_branch": "main"}' | jq .

echo "POST /branches/main/rebase"
curl -s -X POST "$BASE_URL/branches/main/rebase" \
    -H "$AUTH_HEADER" -H "$TENANT_HEADER" \
    -H "Content-Type: application/json" \
    -d '{"name": "ignored", "catalog": "cat_'"$SUFFIX"'", "source_branch": "dev"}' | jq .

echo "---------------------------------------------------"
echo "Tests Completed."
