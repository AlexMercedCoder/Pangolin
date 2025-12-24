#!/bin/bash
set -e

# Configuration
API_URL="http://pangolin-api:8080"
ADMIN_CLI="pangolin-admin"
USER_CLI="pangolin-user"

echo "[CLI TEST] Starting CLI Validation..."

# 1. Login as Root
echo "[CLI TEST] Logging in as Root..."
$ADMIN_CLI login --url $API_URL --username admin --password password
echo "[CLI TEST] Logged in."

# 2. List Tenants
echo "[CLI TEST] Listing Tenants..."
$ADMIN_CLI list-tenants

# 3. Create Tenant (if not exists)
# In Auth test run, tenant 'AcmeCorp' might already exist.
# We'll try to list and grep.
echo "[CLI TEST] Checking for AcmeCorp..."
if $ADMIN_CLI list-tenants | grep -q "AcmeCorp"; then
    echo "[CLI TEST] AcmeCorp exists."
else
    echo "[CLI TEST] Creating AcmeCorp..."
    $ADMIN_CLI create-tenant --name "AcmeCorp" --org "Acme"
fi

# 4. Get User Token (as Tenant Admin)
# We need to act as Tenant Admin to manage resources.
# CLI Login supports --tenant-id.
# We need the Tenant ID.
TENANT_ID=$($ADMIN_CLI list-tenants | grep "AcmeCorp" | awk '{print $1}')
echo "[CLI TEST] Tenant ID: $TENANT_ID"

echo "[CLI TEST] Logging in as Tenant Admin..."
$ADMIN_CLI login --url $API_URL --username admin_acme --password password123 --tenant-id $TENANT_ID
echo "[CLI TEST] Logged in as Tenant Admin."

# 5. List Catalogs
echo "[CLI TEST] Listing Catalogs..."
$ADMIN_CLI list-catalogs

# 6. User CLI Test
echo "[CLI TEST] Switching to User CLI..."
# User CLI also needs login
$USER_CLI login --url $API_URL --username admin_acme --password password123 --tenant-id $TENANT_ID

echo "[CLI TEST] Listing Tables..."
$USER_CLI list-tables --catalog analytics --namespace reports

echo "[CLI TEST] CLI Validation Passed!"
