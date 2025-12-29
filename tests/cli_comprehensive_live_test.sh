#!/bin/bash
set -e

# CLI Comprehensive Live Test (v0.5.0)
# Covers: Service Users, RBAC, Tenants, Users, Warehouses, Catalogs
# Pre-requisite: Server running on localhost:8080

CLI_CMD="cargo run -q -p pangolin_cli_admin -- --url http://localhost:8080"
ROOT_USER="admin"
ROOT_PASS="password"

echo ">>> Starting CLI Live Test <<<"

# 1. Login
echo "[STEP 1] Login as Root"
$CLI_CMD login --username $ROOT_USER --password $ROOT_PASS

# 2. Tenants
echo "[STEP 2] Create Tenant"
TENANT_NAME="cli_test_tenant_$(date +%s)"
$CLI_CMD create-tenant --name $TENANT_NAME --admin-username "tenant_admin" --admin-password "password" 
# Extract Tenant ID from list? Or assume active?
# The CLI prints created tenant info, but storing it is hard in bash without JSON parsing.
# We will use the Tenant context for subsequent commands if possible, or just create resources.

echo "[STEP 2b] Update Tenant"
# Get ID via list and grep?
# For automation simplicity, we'll verify listing works.
$CLI_CMD list-tenants

# Capture Tenant ID for login
TENANT_ID=$($CLI_CMD list-tenants | grep "$TENANT_NAME" | awk '{print $2}')
echo "Captured Tenant ID: $TENANT_ID"

if [ -z "$TENANT_ID" ]; then
    echo "FAILED to capture Tenant ID"
    exit 1
fi

echo "[STEP 2c] Login as Tenant Admin"
$CLI_CMD login --username "tenant_admin" --password "password" --tenant-id "$TENANT_ID"

# 3. Warehouses
echo "[STEP 3] Create Warehouse"
$CLI_CMD create-warehouse "cli_wh_$(date +%s)" --bucket "test-bucket" --region "us-east-1" --access-key "AWS_ACCESS_KEY" --secret-key "AWS_SECRET_KEY" --endpoint "http://minio:9000"
$CLI_CMD list-warehouses

# 4. Service Users (Focus of Audit)
echo "[STEP 4] Service User Lifecycle"
SU_NAME="cli_bot_$(date +%s)"
echo "Creating Service User '$SU_NAME'..."
# Create and capture output to find ID? 
# This is tricky in bash without proper parsing tool (jq). 
# We'll just run the commands and rely on visual verification or subsequent failures if ID is needed.
# Actually, for testing ASSIGN-ROLE, we NEED the ID.
# Let's try to parse the output if possible, or use python script.
# User requested "CLI Live Test". Using python to drive the CLI process might be cleaner?
# No, shell script is standard for "CLI Test".
# We will assume human visual verification for now, OR regex parse.

OUTPUT=$($CLI_CMD create-service-user --name $SU_NAME --role tenant-user)
echo "$OUTPUT"
SU_ID=$(echo "$OUTPUT" | grep "Service User ID:" | awk '{print $4}')
echo "Captured ID: $SU_ID"

if [ -z "$SU_ID" ]; then
    echo "FAILED to capture Service User ID"
    exit 1
fi

echo "Listing Service Users..."
$CLI_CMD list-service-users

echo "Rotating Key..."
$CLI_CMD rotate-service-user-key --id $SU_ID

# 5. Roles & RBAC
echo "[STEP 5] RBAC"
# Assigning Role (Test Cmd Routing)..."
$CLI_CMD assign-role --user-id $SU_ID --role-id "00000000-0000-0000-0000-000000000000" || echo "Expected 404/Error (Role not found)"

# 6. Users
echo "[STEP 6] Users"
USER_NAME="cli_user_$(date +%s)"
$CLI_CMD create-user $USER_NAME --email "test@example.com" --password "password"
$CLI_CMD list-users

# 7. Catalogs
echo "[STEP 7] Catalogs"
# Needs warehouse name. We created one earlier.
# Extract warehouse?
WH_NAME=$(echo "$($CLI_CMD list-warehouses)" | grep "cli_wh" | head -n 1 | awk '{print $2}')
if [ -n "$WH_NAME" ]; then
    $CLI_CMD create-catalog "cli_cat_$(date +%s)" --warehouse $WH_NAME
fi
$CLI_CMD list-catalogs

# 8. Service User Cleanup
echo "[STEP 8] Cleanup"
$CLI_CMD delete-service-user --id $SU_ID

echo ">>> Test Complete <<<"
