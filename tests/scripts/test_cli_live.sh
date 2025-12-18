#!/bin/bash
set -e

# Comprehensive CLI Live Test Script
# Tests all CLI features with MinIO and PyIceberg across multiple tenants

echo "=========================================="
echo "Pangolin CLI Live Test"
echo "=========================================="
echo ""

# Configuration
API_URL="http://localhost:8080"
ADMIN_CLI="./pangolin/target/debug/pangolin-admin"
USER_CLI="./pangolin/target/debug/pangolin-user"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_step() {
    echo -e "${BLUE}===> $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

# Step 1: Login as Root Admin
print_step "Step 1: Login as Root Admin"
$ADMIN_CLI --url $API_URL login --username admin --password password
print_success "Root admin logged in"

# Step 2: Create Two Tenants
print_step "Step 2: Creating Tenants"
$ADMIN_CLI --url $API_URL create-tenant --name tenant_alpha --admin-username admin_alpha --admin-password alpha123 || true
$ADMIN_CLI --url $API_URL create-tenant --name tenant_beta --admin-username admin_beta --admin-password beta123 || true
print_success "Created tenant_alpha and tenant_beta"

# Step 3: Login as Tenant Alpha Admin and Create Infrastructure
print_step "Step 3: Setting up Tenant Alpha Infrastructure"
$ADMIN_CLI --url $API_URL login --username admin_alpha --password alpha123

# Create warehouse for Tenant Alpha
$ADMIN_CLI --url $API_URL create-warehouse warehouse_alpha --type s3 \
$ADMIN_CLI --url $API_URL create-warehouse warehouse_alpha --type s3 \
    --bucket warehouse \
    --access-key minioadmin \
    --secret-key minioadmin \
    --region us-east-1 \
    --endpoint http://localhost:9000 || true

# Create catalog for Tenant Alpha
$ADMIN_CLI --url $API_URL create-catalog catalog_alpha --warehouse warehouse_alpha || true
print_success "Created warehouse_alpha and catalog_alpha"

# Step 4: Create Users in Tenant Alpha
print_step "Step 4: Creating Users in Tenant Alpha"
$ADMIN_CLI --url $API_URL create-user alice_alpha \
    --email alice@alpha.com \
    --role tenant-user \
    --password alice123 || true

$ADMIN_CLI --url $API_URL create-user bob_alpha \
    --email bob@alpha.com \
    --role tenant-user \
    --password bob123 || true

print_success "Created alice_alpha and bob_alpha"

# Step 5: Grant Permissions to Alice
print_step "Step 5: Granting Permissions to Alice"
$ADMIN_CLI --url $API_URL grant-permission alice_alpha read,write,experimental-branching catalog:catalog_alpha || true
print_success "Granted read/write/branching permissions to alice_alpha on catalog_alpha"

# Step 6: Setup Tenant Beta (for Federation Testing)
print_step "Step 6: Setting up Tenant Beta Infrastructure"
$ADMIN_CLI --url $API_URL login --username admin_beta --password beta123

$ADMIN_CLI --url $API_URL create-warehouse warehouse_beta --type s3 \
$ADMIN_CLI --url $API_URL create-warehouse warehouse_beta --type s3 \
    --bucket warehouse \
    --access-key minioadmin \
    --secret-key minioadmin \
    --region us-east-1 \
    --endpoint http://localhost:9000 || true

$ADMIN_CLI --url $API_URL create-catalog catalog_beta --warehouse warehouse_beta || true
print_success "Created warehouse_beta and catalog_beta"

# Step 7: Test Token Generation (as alice_alpha)
print_step "Step 7: Testing Token Generation"
print_info "Logging in as alice_alpha..."

# Login as alice_alpha and generate token
TOKEN_OUTPUT=$($USER_CLI --url $API_URL login --username alice_alpha --password alice123 2>&1)
LOGIN_EXIT_CODE=$?
echo "DEBUG: Login Output: $TOKEN_OUTPUT"

if [ $LOGIN_EXIT_CODE -ne 0 ]; then
    echo "❌ Failed to login as alice_alpha"
    exit 1
fi

TOKEN=$($USER_CLI --url $API_URL get-token --description "PyIceberg automation test" --expires-in 3600 2>&1 | grep "Token:" | sed 's/\x1b\[[0-9;]*m//g' | awk '{print $2}')

if [ -n "$TOKEN" ]; then
    print_success "Token generated successfully"
    print_info "Token: ${TOKEN:0:50}..."
else
    echo "Failed to generate token"
    exit 1
fi

# DEBUG: Decode token to check tenant_id
echo "DEBUG: Token Payload:"
echo "$TOKEN" | cut -d. -f2 | base64 -d 2>/dev/null || echo "Base64 decode failed"
echo ""

# Step 8: Test Federated Catalog Creation
print_step "Step 8: Testing Federated Catalog Creation"

print_info "Generating token for admin_beta..."
# Login as admin_beta to generate token
$USER_CLI --url $API_URL login --username admin_beta --password beta123
if [ $? -ne 0 ]; then
    echo "❌ Failed to login as admin_beta"
    exit 1
fi

BETA_TOKEN_OUTPUT=$($USER_CLI --url $API_URL get-token --description "Federation" --expires-in 3600 2>&1)
echo "DEBUGGING TOKEN OUTPUT (RAW):" > /tmp/token_debug.log
echo "$BETA_TOKEN_OUTPUT" >> /tmp/token_debug.log
BETA_TOKEN=$(echo "$BETA_TOKEN_OUTPUT" | grep --color=never "Token:" | sed 's/\x1b\[[0-9;]*m//g' | awk '{print $2}')
echo "DEBUGGING TOKEN (CLEAN):" >> /tmp/token_debug.log
echo "'$BETA_TOKEN'" >> /tmp/token_debug.log

if [ -z "$BETA_TOKEN" ]; then
    echo "❌ Failed to generate Beta token"
    exit 1
fi
print_info "Generated Beta Token: ${BETA_TOKEN:0:10}..."

# Switch back to tenant_alpha as admin (CRITICAL if CLIs share config)
print_info "Switching back to admin_alpha..."
$ADMIN_CLI --url $API_URL login --username admin_alpha --password alpha123
if [ $? -ne 0 ]; then
    echo "❌ Failed to login as admin_alpha"
    exit 1
fi

# Create federated catalog in tenant_alpha pointing to tenant_beta's catalog
$ADMIN_CLI --url $API_URL create-federated-catalog fed_catalog_beta \
    --base-url "$API_URL/v1/catalog_beta" \
    --storage-location "s3://warehouse-alpha/fed_beta" \
    --auth-type BearerToken \
    --token "$BETA_TOKEN" \
    --timeout 30

print_success "Created federated catalog fed_catalog_beta"

# Step 8.5: Create Table in Catalog Beta (for Federation Test)
print_step "Step 8.5: Creating Table in Catalog Beta"
cat > /tmp/setup_beta_table.py <<'PYEOF'
from pyiceberg.catalog import load_catalog
import sys
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, LongType
import pyarrow as pa

if len(sys.argv) < 2:
    print("Usage: python setup_beta_table.py <token>")
    sys.exit(1)

token = sys.argv[1]

# Connect to catalog_beta
catalog = load_catalog(
    "catalog_beta",
    **{
        "uri": "http://localhost:8080/v1/catalog_beta",
        "token": token,
        "s3.endpoint": "http://localhost:9000",
        "s3.region": "us-east-1",
        "s3.access-key-id": "minioadmin",
        "s3.secret-access-key": "minioadmin"
    }
)

# Create namespace
try:
    catalog.create_namespace("beta_ns")
    print("✓ Created namespace beta_ns")
except Exception:
    pass

# Create table
schema = Schema(
    NestedField(1, "id", LongType(), required=True),
    NestedField(2, "data", StringType(), required=True),
)

try:
    table = catalog.create_table("beta_ns.fed_source_table", schema=schema)
    print("✓ Created table beta_ns.fed_source_table")
except Exception:
    table = catalog.load_table("beta_ns.fed_source_table")

# Write data
data = pa.table({
    "id": [100, 200],
    "data": ["Federated", "Data"]
})
table.append(data)
print("✓ Wrote data to beta table")
PYEOF

python3 /tmp/setup_beta_table.py "$BETA_TOKEN"
print_success "Created table in catalog_beta"

# Step 9: List and Test Federated Catalogs
print_step "Step 9: Testing Federated Catalog Operations"
$ADMIN_CLI --url $API_URL list-federated-catalogs
$ADMIN_CLI --url $API_URL test-federated-catalog fed_catalog_beta

# Step 9.5: Verify Federated Access via PyIceberg
print_step "Step 9.5: Verifying Federated Access via PyIceberg"
cat > /tmp/test_federated_access.py <<'PYEOF'
from pyiceberg.catalog import load_catalog
import sys

if len(sys.argv) < 2:
    print("Usage: python test_federated_access.py <token>")
    sys.exit(1)

token = sys.argv[1]

# Connect to fed_catalog_beta
# This catalog is in tenant_alpha but proxies to tenant_beta
catalog = load_catalog(
    "fed_catalog_beta",
    **{
        "uri": "http://localhost:8080/v1/fed_catalog_beta",
        "token": token,
        "s3.endpoint": "http://localhost:9000",
        "s3.region": "us-east-1",
        "s3.access-key-id": "minioadmin",
        "s3.secret-access-key": "minioadmin"
    }
)

print("✓ Connected to fed_catalog_beta")

# List tables (should verify connection)
tables = catalog.list_tables("beta_ns")
print(f"✓ Found tables: {tables}")

# Read table
table = catalog.load_table("beta_ns.fed_source_table")
df = table.scan().to_arrow()
print(f"✓ Read {len(df)} rows from federated table")
print(df)
PYEOF

# Use ALICE's token (from tenant_alpha) to access the federated catalog in tenant_alpha
# The server should proxy this request to tenant_beta using the stored federation credentials
python3 /tmp/test_federated_access.py "$TOKEN"
print_success "Federated table access verified"

# Step 10: Test Enhanced Branching
print_step "Step 10: Testing Enhanced Branching"
# Login as alice_alpha
$USER_CLI --url $API_URL login --username alice_alpha --password alice123

# Create feature branch
$USER_CLI --url $API_URL create-branch --catalog catalog_alpha feature-branch-1 \
    --branch-type feature

# Create experiment branch with partial assets (will need tables first)
print_info "Creating experiment branch (partial branching will be tested after table creation)"

# List branches
$USER_CLI --url $API_URL list-branches --catalog catalog_alpha
print_success "Enhanced branching tested"

# Step 11: Create PyIceberg Test Script
print_step "Step 11: Creating PyIceberg Test Script"
cat > /tmp/test_pyiceberg_cli.py <<'PYEOF'
from pyiceberg.catalog import load_catalog
import sys

# Get token from command line
if len(sys.argv) < 2:
    print("Usage: python test_pyiceberg_cli.py <token>")
    sys.exit(1)

token = sys.argv[1]

# Connect to catalog_alpha
catalog = load_catalog(
    "catalog_alpha",
    **{
        "uri": "http://localhost:8080/v1/catalog_alpha",
        "token": token,
        "s3.endpoint": "http://localhost:9000",
        "s3.region": "us-east-1",
        "s3.access-key-id": "minioadmin",
        "s3.secret-access-key": "minioadmin"
    }
)

print("✓ Connected to catalog_alpha")

# Create namespace
try:
    catalog.create_namespace("test_ns")
    print("✓ Created namespace test_ns")
except Exception as e:
    print(f"ℹ Namespace might already exist: {e}")

# Create table
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, LongType

schema = Schema(
    NestedField(1, "id", LongType(), required=True),
    NestedField(2, "name", StringType(), required=True),
)

try:
    table = catalog.create_table("test_ns.test_table", schema=schema)
    print("✓ Created table test_ns.test_table")
except Exception as e:
    print(f"ℹ Table might already exist: {e}")
    table = catalog.load_table("test_ns.test_table")

# Write data
import pyarrow as pa
data = pa.table({
    "id": [1, 2, 3],
    "name": ["Alice", "Bob", "Charlie"]
})

table.append(data)
print("✓ Wrote data to table")

# Read data
df = table.scan().to_arrow()
print(f"✓ Read {len(df)} rows from table")
print(df)

print("\n✅ All PyIceberg operations successful!")
PYEOF

print_success "PyIceberg test script created"

# Step 12: Run PyIceberg Test
print_step "Step 12: Running PyIceberg Integration Test"
python3 /tmp/test_pyiceberg_cli.py "$TOKEN"
print_success "PyIceberg integration test passed"

# Step 13: Test Partial Branching with Assets
print_step "Step 13: Testing Partial Branching"
$USER_CLI --url $API_URL create-branch --catalog catalog_alpha experiment-branch-1 \
    --branch-type experiment \
    --assets "test_table"

print_success "Partial branching tested"

# Step 14: Test Permission Listing
print_step "Step 14: Testing Permission Management"
$ADMIN_CLI --url $API_URL login --username admin_alpha --password alpha123
$ADMIN_CLI --url $API_URL list-permissions --user alice_alpha
print_success "Permission listing tested"

# Step 15: Cleanup Test
print_step "Step 15: Testing Cleanup Operations"
print_info "Deleting federated catalog..."
echo "yes" | $ADMIN_CLI --url $API_URL delete-federated-catalog fed_catalog_beta
print_success "Federated catalog deleted"

# Final Summary
echo ""
echo "=========================================="
echo "CLI Live Test Summary"
echo "=========================================="
print_success "✓ Root admin login"
print_success "✓ Tenant creation (tenant_alpha, tenant_beta)"
print_success "✓ Warehouse creation"
print_success "✓ Catalog creation"
print_success "✓ User creation"
print_success "✓ Permission granting"
print_success "✓ Token generation"
print_success "✓ Federated catalog creation"
print_success "✓ Federated catalog listing"
print_success "✓ Federated catalog testing"
print_success "✓ Enhanced branching (type + partial assets)"
print_success "✓ PyIceberg integration (create/write/read)"
print_success "✓ Cleanup operations"
echo ""
print_success "All CLI features tested successfully!"
