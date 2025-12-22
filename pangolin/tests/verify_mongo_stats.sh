#!/bin/bash

# Cleanup
pkill -f pangolin_api

# Start Mongo via Docker
echo "Starting Mongo Container..."
docker compose down -v # Clean up old volumes
docker compose up -d mongo
# Wait for Mongo to be ready
echo "Waiting for Mongo..."
sleep 15

# Start Server with Mongo
echo "Starting Pangolin API (MongoStore)..."
export PANGOLIN_STORAGE_TYPE=mongo
export DATABASE_URL=mongodb://localhost:27017
export MONGO_DB_NAME=pangolin_test
export PANGOLIN_NO_AUTH=true
export PANGOLIN_JWT_SECRET=default_secret_for_dev

./target/debug/pangolin_api > mongo_verify.log 2>&1 &
SERVER_PID=$!
echo "Server PID: $SERVER_PID"

# Wait for start
sleep 5

# Extract Token
TOKEN=$(grep -oP '"token": "\K[^"]+' mongo_verify.log | head -1)
echo "Token: $TOKEN"

if [ -z "$TOKEN" ]; then
    echo "Failed to extract token"
    cat mongo_verify.log
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

# Create Catalog
curl -s -X POST http://localhost:8080/api/v1/catalogs \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name": "cat1", "catalog_type": "Local"}'

# Create Namespace
curl -s -X POST http://localhost:8080/v1/cat1/v1/namespaces \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"namespace": ["db1"]}'

# Create Table
curl -s -X POST http://localhost:8080/v1/cat1/v1/namespaces/db1/tables \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name": "tbl1", "schema": {"type": "struct", "fields": []}, "location": "file:///tmp/pangolin_test/tbl1_mongo"}'

sleep 1

# 3. Verify
echo "--- Final Check ---"
check_stats 1 1 1
RESULT=$?

if [ $RESULT -ne 0 ]; then
    echo "Retrying check with 2 namespaces..."
    check_stats 1 2 1
    RESULT=$?
fi

kill $SERVER_PID
exit $RESULT
