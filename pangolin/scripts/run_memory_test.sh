#!/bin/bash
set -e

# Configuration
MINIO_CONTAINER="pangolin-minio"
API_PORT=8080

# 1. Start MinIO
echo "Starting MinIO..."
if [ "$(docker ps -q -f name=$MINIO_CONTAINER)" ]; then
    echo "MinIO already running."
elif [ "$(docker ps -aq -f name=$MINIO_CONTAINER)" ]; then
    echo "Starting existing MinIO container..."
    docker start $MINIO_CONTAINER
else
    echo "Creating and starting new MinIO container..."
    docker run -d -p 9000:9000 -p 9001:9001 \
        -e "MINIO_ROOT_USER=minioadmin" \
        -e "MINIO_ROOT_PASSWORD=minioadmin" \
        --name $MINIO_CONTAINER \
        minio/minio server /data --console-address ":9001"
fi

# Wait for MinIO (simple sleep, or curl check)
echo "Waiting for MinIO..."
sleep 5

# 2. Start Pangolin API (MemoryStore)
echo "Starting Pangolin API (MemoryStore)..."
# Unset DATABASE_URL to force MemoryStore
export DATABASE_URL=
export PANGOLIN_NO_AUTH=true
# Start in background, redirect logs
cargo run -p pangolin_api --bin pangolin_api > api_test.log 2>&1 &
API_PID=$!
echo "API PID: $API_PID"

# Wait for API to be ready
echo "Waiting for API to be ready..."
RETRIES=30
while [ $RETRIES -gt 0 ]; do
    if curl -s http://localhost:$API_PORT/health > /dev/null; then
        echo "API is ready!"
        break
    fi
    echo "Waiting for API... ($RETRIES)"
    sleep 2
    RETRIES=$((RETRIES-1))
done

if [ $RETRIES -eq 0 ]; then
    echo "API failed to start. Logs:"
    cat api_test.log
    kill $API_PID
    exit 1
fi

# 3. Run Test Script
echo "Running Python Test Script..."
# Ensure venv or dependencies - assuming user env has them or we just run usage
# Use python3 and hope dependencies (pyiceberg, requests, boto3) are installed.
# If not, we might fail. The user context says "The user's OS version is linux." and likely has dev env.
if python3 scripts/test_memory_live_minio.py; then
    echo "Test passed!"
    RESULT=0
else
    echo "Test failed!"
    RESULT=1
fi

# 4. Cleanup
echo "Stopping API..."
kill $API_PID
# Optional: Stop MinIO? User might want to inspect it. keeping it running.
# docker stop $MINIO_CONTAINER

exit $RESULT
