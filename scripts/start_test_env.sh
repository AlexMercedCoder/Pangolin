#!/bin/bash
export RUST_LOG=info
export PANGOLIN_STORAGE_TYPE=sqlite
export DATABASE_URL=sqlite:///pangolin_test.db
export PANGOLIN_ROOT_USER=admin
export PANGOLIN_ROOT_PASSWORD=password
export AWS_ACCESS_KEY_ID=minioadmin
export AWS_SECRET_ACCESS_KEY=minioadmin
export AWS_REGION=us-east-1
export AWS_ENDPOINT_URL=http://localhost:9000
export PANGOLIN_URL=http://localhost:8080
export S3_ENDPOINT=http://localhost:9000

# Start API in background
echo "Starting Pangolin API..."
cd pangolin && cargo run --bin pangolin_api > ../api.log 2>&1 &
API_PID=$!
echo "API started with PID $API_PID"

# Start UI in background
echo "Starting Pangolin UI..."
cd pangolin_ui && npm run dev -- --port 3000 > ../ui.log 2>&1 &
UI_PID=$!
echo "UI started with PID $UI_PID"

echo "Waiting for services to be ready..."
sleep 10

echo "Environment ready for testing."
echo "API PID: $API_PID"
echo "UI PID: $UI_PID"
echo "Check api.log and ui.log for output."
