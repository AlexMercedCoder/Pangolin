#!/bin/bash

# Cleanup
rm -f tests/test.db*
pkill -f pangolin_api

# Start Server
echo "Starting Pangolin API (SqliteStore)..."
export PANGOLIN_STORAGE_TYPE=sqlite
export DATABASE_URL=sqlite://tests/test.db
export PANGOLIN_NO_AUTH=true
export PANGOLIN_JWT_SECRET=default_secret_for_dev
./target/debug/pangolin_api > sqlite_verify.log 2>&1 &
SERVER_PID=$!
echo "Server PID: $SERVER_PID"

# Wait for start
sleep 5

# Extract Token
TOKEN=$(grep -oP '"token": "\K[^"]+' sqlite_verify.log | head -1)
echo "Token: $TOKEN"

if [ -z "$TOKEN" ]; then
    echo "Failed to extract token"
    cat sqlite_verify.log
    kill $SERVER_PID
    exit 1
fi

# Function to check stats
check_stats() {
    EXPECTED_CATS=$1
    EXPECTED_NS=$2
    EXPECTED_TABLES=$3
    
    STATS=$(curl -s -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/v1/dashboard/stats)
    
    # Check for error response
    if echo "$STATS" | grep -q "error"; then
        echo "Error response: $STATS"
        return 1
    fi
    
    CATS=$(echo $STATS | jq '.catalogs_count')
    NS=$(echo $STATS | jq '.namespaces_count')
    TABLES=$(echo $STATS | jq '.tables_count')
    
    echo "Stats: Cats=$CATS, NS=$NS, Tables=$TABLES"
    
    if [[ "$CATS" == "$EXPECTED_CATS" && "$NS" == "$EXPECTED_NS" && "$TABLES" == "$EXPECTED_TABLES" ]]; then
        echo "✅ match"
        return 0
    else
        echo "❌ mismatch (Expected $EXPECTED_CATS/$EXPECTED_NS/$EXPECTED_TABLES)"
        return 1
    fi
}

# 1. Initial Check
echo "--- Initial Check ---"
check_stats 0 0 0
if [ $? -ne 0 ]; then kill $SERVER_PID; exit 1; fi

# 2. Setup Resources
echo "--- Creating Resources ---"
# Tenant Admin Token (Auto-provisioned)
# We need to get the token or just use the known secret if simple mode.
# The server logs the token on startup usually or we can rely on NO_AUTH if enabled in main?
# Wait, "WARNING: NO_AUTH MODE ENABLED" was in logs.
# If NO_AUTH, we might need a dummy token.
# Let's assume NO_AUTH is on or we grab the token.
# Actually, the previous run output showed a hardcoded token in logs?
# "token": "eyJ..."
# I'll just use a dummy token if NO_AUTH is enabled, logic in `auth_middleware` allows it?
# In `main.rs`, `NO_AUTH` mode creates a default admin.
# Let's try to just hit endpoints.

# Create Catalog
curl -s -X POST http://localhost:8080/api/v1/catalogs \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name": "cat1", "catalog_type": "Local"}'

# Create Namespace
# Route: /v1/:prefix/v1/namespaces
curl -s -X POST http://localhost:8080/v1/cat1/v1/namespaces \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"namespace": ["db1"]}'

# Create Table
# Route: /v1/:prefix/v1/namespaces/:namespace/tables
curl -s -X POST http://localhost:8080/v1/cat1/v1/namespaces/db1/tables \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name": "tbl1", "schema": {"type": "struct", "fields": []}, "location": "file:///tmp/pangolin_test/tbl1"}'

sleep 1

# 3. Verify
echo "--- Final Check ---"
# 1 Catalog, 1 Explicit Namespace + 1 Default Namespace? 
# Does SQLite implementation create default namespace?
# `pangolin_handlers::create_catalog` creates "default" namespace if it succeeds.
# So expected: 1 Catalog, 2 Namespaces (default, db1), 1 Table.
check_stats 1 2 1
RESULT=$?

kill $SERVER_PID
exit $RESULT
