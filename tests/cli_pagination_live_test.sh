#!/bin/bash
set -e

# Cleanup function
cleanup() {
    echo "Stopping Pangolin server..."
    if [ -n "$SERVER_PID" ]; then
        kill $SERVER_PID
    fi
    rm -f pangolin_api pangolin-admin pangolin-user
}
trap cleanup EXIT

echo "Building binaries..."
cd pangolin
cargo build --bin pangolin_api --bin pangolin-admin --bin pangolin-user
cd ..
cp pangolin/target/debug/pangolin_api ./pangolin_server_bin
cp pangolin/target/debug/pangolin-admin .
cp pangolin/target/debug/pangolin-user .

# Start Server
echo "Starting Pangolin Server (Memory Store)..."
export PANGOLIN_STORAGE_TYPE=memory
export PORT=8085
export RUST_LOG=info
export PANGOLIN_SEED_ADMIN=true
export PANGOLIN_ADMIN_USER=root
export PANGOLIN_ADMIN_PASSWORD=rootpass
./pangolin_server_bin > server.log 2>&1 &
SERVER_PID=$!

echo "Waiting for server to start..."
sleep 5

# Setup Admin CLI env
export PANGOLIN_URL="http://localhost:8085"

# Login as Root
echo "Logging in as Root..."
./pangolin-admin login --username "root" --password "rootpass" --tenant-id "00000000-0000-0000-0000-000000000000"

# Create Tenants (20 tenants)
echo "Creating 20 tenants..."
# Create first one with output to verify user creation
# Create first one with output to verify user creation AND capture output
OUTPUT=$(./pangolin-admin create-tenant --name "Tenant1" --admin-username "admin1" --admin-password "password1")
echo "$OUTPUT"

# Capture Tenant ID (UUID format)
TENANT1_ID=$(echo "$OUTPUT" | grep -oE '[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}' | head -n 1)
echo "Captured Tenant1 ID: $TENANT1_ID"

if [ -z "$TENANT1_ID" ]; then
    echo "Error: Failed to capture Tenant ID"
    exit 1
fi

for i in {2..20}; do
    ./pangolin-admin create-tenant --name "Tenant$i" --admin-username "admin$i" --admin-password "password$i" > /dev/null
done

# Test Admin Pagination
echo "Testing Admin Pagination (List Tenants)..."

# Limit 5, Offset 0
OUTPUT_L5_O0=$(./pangolin-admin list-tenants --limit 5 --offset 0)
COUNT_L5_O0=$(echo "$OUTPUT_L5_O0" | grep "Tenant" | wc -l)
echo "Limit 5, Offset 0 count: $COUNT_L5_O0"
if [ "$COUNT_L5_O0" -ne 5 ]; then
    echo "FAILED: Expected 5 tenants, got $COUNT_L5_O0"
    exit 1
fi

# Limit 5, Offset 5
OUTPUT_L5_O5=$(./pangolin-admin list-tenants --limit 5 --offset 5)
COUNT_L5_O5=$(echo "$OUTPUT_L5_O5" | grep "Tenant" | wc -l)
echo "Limit 5, Offset 5 count: $COUNT_L5_O5"
if [ "$COUNT_L5_O5" -ne 5 ]; then
    echo "FAILED: Expected 5 tenants (page 2), got $COUNT_L5_O5"
    exit 1
fi

# Verify no overlap
FIRST_P1=$(echo "$OUTPUT_L5_O0" | grep "Tenant" | head -n 1)
FIRST_P2=$(echo "$OUTPUT_L5_O5" | grep "Tenant" | head -n 1)

if [ "$FIRST_P1" == "$FIRST_P2" ]; then
    echo "FAILED: Page 1 and Page 2 start with same item: $FIRST_P1"
    exit 1
fi

echo "Admin Pagination Passed!"

# Test User CLI Pagination
echo "Testing User CLI Pagination..."

# Create 10 Catalogs using Admin CLI (Logged in as Tenant Admin)
echo "Creating 10 catalogs for Tenant1..."
./pangolin-admin login --username "admin1" --password "password1" --tenant-id "$TENANT1_ID" || { echo "Error: Admin Login Failed"; exit 1; }

# Create Warehouse first
./pangolin-admin create-warehouse "Warehouse1" --type "s3" --bucket "b1"

for i in {1..10}; do
    ./pangolin-admin create-catalog "Catalog$i" --warehouse "Warehouse1" > /dev/null
done

# Now list catalogs with User CLI
# Login as Tenant User (admin1) with Tenant ID
./pangolin-user login --username "admin1" --password "password1" --tenant-id "$TENANT1_ID" || { echo "Error: User Login Failed"; exit 1; }

# List Catalogs Limit 3, Offset 0
OUTPUT_UC_L3_O0=$(./pangolin-user list-catalogs --limit 3 --offset 0)
COUNT_UC_L3_O0=$(echo "$OUTPUT_UC_L3_O0" | grep "Catalog" | wc -l)
echo "User CLI Limit 3, Offset 0 count: $COUNT_UC_L3_O0"

if [ "$COUNT_UC_L3_O0" -ne 3 ]; then
    echo "FAILED: Expected 3 catalogs, got $COUNT_UC_L3_O0"
    exit 1
fi

# List Catalogs Limit 3, Offset 3
OUTPUT_UC_L3_O3=$(./pangolin-user list-catalogs --limit 3 --offset 3)
COUNT_UC_L3_O3=$(echo "$OUTPUT_UC_L3_O3" | grep "Catalog" | wc -l)
echo "User CLI Limit 3, Offset 3 count: $COUNT_UC_L3_O3"

if [ "$COUNT_UC_L3_O3" -ne 3 ]; then
    echo "FAILED: Expected 3 catalogs (page 2), got $COUNT_UC_L3_O3"
    exit 1
fi

echo "User CLI Pagination Passed!"

echo "ALL TESTS PASSED"
