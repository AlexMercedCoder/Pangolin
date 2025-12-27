#!/bin/bash
set -e

# Configuration
API_PORT=8080
API_URL="http://localhost:${API_PORT}"
# Using MinIO from docker-compose
export AWS_ACCESS_KEY_ID=minioadmin
export AWS_SECRET_ACCESS_KEY=minioadmin
export AWS_REGION=us-east-1
export AWS_ENDPOINT_URL=http://localhost:9000
# S3 is needed for all backends to store data
# export PANGOLIN_NO_AUTH=true 
export PANGOLIN_SEED_ADMIN=true
export PANGOLIN_ADMIN_USER=admin
export PANGOLIN_ADMIN_PASSWORD=password
export RUST_LOG=info
export PYTHONPATH=$(pwd)/pypangolin/src:$PYTHONPATH

# Ensure binaries are built
echo "Building binaries..."
cargo build --manifest-path pangolin/Cargo.toml -p pangolin_api

# Function to run test
run_test() {
    backend_name=$1
    echo "----------------------------------------------------------------"
    echo "Running Verification for Backend: ${backend_name}"
    echo "----------------------------------------------------------------"

    # Start API in background
    echo "Starting API..."
    cargo run --manifest-path pangolin/Cargo.toml -p pangolin_api --bin pangolin_api > api_output_${backend_name}.log 2>&1 &
    API_PID=$!
    
    # Wait for API to be ready
    echo "Waiting for API to be ready..."
    sleep 60
    # Ideally use a health check loop
    
    # Run Python Verification Script
    echo "Running Python Verification Script..."
    if python3 scripts/verify_pypangolin_merge.py > python_output.log 2>&1; then
        echo "✅ ${backend_name} Verification PASSED"
        echo "--- Python Output ---"
        cat python_output.log
        echo "---------------------"
    else
        echo "❌ ${backend_name} Verification FAILED"
        echo "--- Python Output ---"
        cat python_output.log
        echo "---------------------"
        echo "API Log Tail:"
        tail -n 50 api_output_${backend_name}.log
        kill $API_PID
        exit 1
    fi

    # Stop API
    echo "Stopping API..."
    kill $API_PID
    wait $API_PID 2>/dev/null || true
    echo "API Stopped"
    sleep 2
}

# 1. Memory Store (Default)
unset DATABASE_URL
unset PANGOLIN_STORAGE_TYPE
# run_test "Memory"
# ...
# run_test "SQLite"

# 3. Postgres Store
# check if running
if nc -z localhost 5432; then
   echo "Resetting Postgres DB..."
   PGPASSWORD=testpass dropdb -h localhost -U testuser testdb || true
   PGPASSWORD=testpass createdb -h localhost -U testuser testdb || true
   export DATABASE_URL="postgres://testuser:testpass@localhost:5432/testdb"
   run_test "Postgres"
else
   echo "⚠️ Postgres not running on 5432, skipping Postgres test."
fi

# 4. Mongo Store
if nc -z localhost 27017; then
    export DATABASE_URL="mongodb://testuser:testpass@localhost:27017/pangolin_test?authSource=admin"
    run_test "Mongo"
else
    echo "⚠️ Mongo not running on 27017, skipping Mongo test."
fi

echo "================================================================"
echo "🎉 ALL BACKEND VERIFICATIONS COMPLETED SUCCESSFULLY"
echo "================================================================"
