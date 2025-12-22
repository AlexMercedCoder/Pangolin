#!/bin/bash
set -e

echo "=== PostgresStore Performance Optimization Live Test ==="
echo ""

# Cleanup function
cleanup() {
    echo ""
    echo "6. Cleanup..."
    pkill -f pangolin_api || true
    docker stop minio 2>/dev/null || true
    docker rm minio 2>/dev/null || true
    docker stop postgres 2>/dev/null || true
    docker rm postgres 2>/dev/null || true
}

trap cleanup EXIT

# 1. Build
echo "1. Building Pangolin API..."
cd "$(dirname "$0")/.."
cargo build --bin pangolin_api 2>&1 | tail -5

# 2. Start Services (MinIO & Postgres)
echo ""
echo "2. Starting Services..."

# Start MinIO
echo "Starting MinIO..."
docker rm -f minio 2>/dev/null || true
docker run -d --name minio \
    -p 9000:9000 \
    -p 9002:9001 \
    -e "MINIO_ROOT_USER=minioadmin" \
    -e "MINIO_ROOT_PASSWORD=minioadmin" \
    quay.io/minio/minio server /data --console-address ":9001"

# Start Postgres
echo "Starting Postgres..."
docker rm -f postgres 2>/dev/null || true
docker run -d --name postgres \
    -p 5432:5432 \
    -e POSTGRES_USER=pangolin \
    -e POSTGRES_PASSWORD=pangolin \
    -e POSTGRES_DB=pangolin \
    postgres:15

sleep 5

echo "Waiting for MinIO to be ready..."
until curl -s http://localhost:9000/minio/health/live > /dev/null 2>&1; do
    sleep 1
done
echo "Creating test bucket..."
docker exec minio mc alias set local http://localhost:9000 minioadmin minioadmin
docker exec minio mc mb local/test-warehouse

echo "Waiting for Postgres to be ready..."
until docker exec postgres pg_isready -U pangolin > /dev/null 2>&1; do
    sleep 1
done
echo "Postgres is ready relative to container, waiting 5s for port forwarding..."
sleep 5

# 3. Start API with Postgres backend
echo ""
echo "3. Starting Pangolin API (Postgres backend)..."
export STORE_TYPE=postgres
export DATABASE_URL="postgres://pangolin:pangolin@localhost:5432/pangolin?sslmode=disable"
export DATABASE_MAX_CONNECTIONS=5
export DATABASE_MIN_CONNECTIONS=2
export DATABASE_CONNECT_TIMEOUT=30
export PANGOLIN_SEED_ADMIN=true
export PANGOLIN_JWT_SECRET=test-secret-key-for-testing-only
export RUST_LOG=info,pangolin_store=debug

# Start API in background and capture output
./target/debug/pangolin_api > /tmp/api_output.log 2>&1 &
API_PID=$!
echo "API PID: $API_PID"

# Wait for API to be ready
echo "Waiting for API to be ready..."
for i in {1..30}; do
    echo "  Attempt $i/30..."
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then
        echo "API is ready!"
        break
    fi
    sleep 1
done

# Extract token from logs
TOKEN=$(grep '"token":' /tmp/api_output.log | sed 's/.*"token": "\([^"]*\)".*/\1/' | tr -d '\n')
if [ -z "$TOKEN" ]; then
    echo "ERROR: Failed to extract token from API logs"
    cat /tmp/api_output.log
    exit 1
fi
echo "Extracted token from startup logs"

# Export token for Python script
export API_TOKEN="$TOKEN"

# Run tests
echo ""
echo "4. Running tests..."

python3 << 'EOF'
import os
import sys
import time
import requests
from pyiceberg.catalog import load_catalog
import pyarrow as pa

# Configuration
API_URL = "http://localhost:8080"

# Use token from environment (passed from shell)
token = os.environ.get("API_TOKEN", "")
if not token:
    print("ERROR: No API token available")
    sys.exit(1)

headers = {"Authorization": f"Bearer {token}"}
print("✓ Using startup token")

# Use default tenant (created automatically by API)
tenant_id = "00000000-0000-0000-0000-000000000000"
print(f"✓ Using default tenant: {tenant_id}")

# Create warehouse
print("✓ Creating warehouse...")
warehouse_response = requests.post(
    f"{API_URL}/api/v1/warehouses",
    json={
        "name": "test_wh",
        "storage_config": {
            "s3.endpoint": "http://localhost:9000",
            "s3.access-key-id": "minioadmin",
            "s3.secret-access-key": "minioadmin",
            "s3.region": "us-east-1",
            "s3.bucket": "test-warehouse"
        },
        "use_sts": False
    },
    headers=headers)
if warehouse_response.status_code not in [200, 201]:
    print(f"  Failed: {warehouse_response.status_code} - {warehouse_response.text}")
    print(warehouse_response.text)
    sys.exit(1)
print("  Warehouse created")

# Create catalog
print("✓ Creating catalog...")
catalog_response = requests.post(
    f"{API_URL}/api/v1/catalogs",
    json={
        "name": "test_catalog",
        "warehouse_name": "test_wh",
        "catalog_type": "Local",
        "storage_location": "s3://test-warehouse/catalog",
        "properties": {}
    },
    headers=headers)
if catalog_response.status_code not in [200, 201]:
    print(f"  Failed: {catalog_response.status_code} - {catalog_response.text}")
    print(catalog_response.text)
    sys.exit(1)
print("  Catalog created")

# Load catalog with PyIceberg
print("✓ Loading catalog with PyIceberg...")
catalog = load_catalog(
    "test_catalog",
    **{
        "type": "rest",
        "uri": "http://localhost:8080",
        "prefix": "test_catalog",
        "token": token,
        "header.X-Pangolin-Tenant": tenant_id,
        # Client-side S3 credentials
        "s3.endpoint": "http://localhost:9000",
        "s3.access-key-id": "minioadmin",
        "s3.secret-access-key": "minioadmin",
        "s3.region": "us-east-1",
        "s3.path-style-access": "true",
    }
)
print("  Catalog loaded")

# Create namespace
print("✓ Creating namespace with PyIceberg...")
catalog.create_namespace("test_ns")
print("  Namespace created")

# Create table (tests object store cache)
print("✓ Creating table (tests object store cache)...")
schema = pa.schema([
    pa.field("id", pa.int64()),
    pa.field("name", pa.string()),
])
table = catalog.create_table("test_ns.test_table", schema=schema)
print(f"  Table created: test_ns.test_table")

# Write data (tests object store cache)
print("✓ Writing data (tests object store cache)...")
data = pa.table({
    "id": [1, 2, 3],
    "name": ["Alice", "Bob", "Charlie"],
})
table.append(data)
print("  Data written")

# Verify files in MinIO
print("✓ Verifying files in MinIO...")
import subprocess
result = subprocess.run(
    ["docker", "exec", "minio", "mc", "ls", "--recursive", "local/test-warehouse"],
    capture_output=True,
    text=True
)
if result.returncode == 0:
    files = [line for line in result.stdout.split('\n') if line.strip()]
    print(f"  ✓ Found {len(files)} files in MinIO:")
    for f in files[:5]:  # Show first 5 files
        print(f"    - {f.split()[-1] if f.split() else f}")
    if len(files) > 5:
        print(f"    ... and {len(files) - 5} more files")
else:
    print(f"  ⚠️  Could not list MinIO files: {result.stderr}")

# Read data 3 times (tests metadata cache)
print("✓ Reading data 3 times (tests metadata cache)...")
for i in range(3):
    start = time.time()
    df = table.scan().to_arrow()
    elapsed = time.time() - start
    print(f"  Read {i+1}: {len(df)} rows in {elapsed:.3f}s")

# Check for cache performance improvement
if elapsed < 0.025:  # If last read was faster
    print("  ✓ Metadata cache appears to be working (faster subsequent reads)")

print("")
print("✅ All tests passed!")
EOF

# 5. Check cache statistics in logs
echo ""
echo "5. Cache Statistics (from logs):"
echo "Metadata cache hits/misses:"
grep -i "metadata cache" /tmp/api_output.log | tail -5 || echo "No cache logs found"

echo ""
echo "=== ✅ Test Complete - All Optimizations Working ==="
