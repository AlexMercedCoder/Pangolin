#!/bin/bash

# Configuration
API_URL="http://localhost:8080"
ADMIN_USER="admin"
ADMIN_PASS="password"

echo "1. Logging in as Root ($ADMIN_USER)..."
LOGIN_RESP=$(curl -s -X POST "$API_URL/api/v1/users/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"$ADMIN_USER\", \"password\": \"$ADMIN_PASS\"}")

ROOT_TOKEN=$(echo $LOGIN_RESP | python3 -c "import sys, json; print(json.load(sys.stdin)['token'])")

if [ -z "$ROOT_TOKEN" ] || [ "$ROOT_TOKEN" == "None" ]; then
    echo "Failed to login as root. Response: $LOGIN_RESP"
    exit 1
fi
echo "Root Token acquired."

echo "2. Creating Tenant 'manual_test_tenant'..."
TENANT_RESP=$(curl -s -X POST "$API_URL/api/v1/tenants" \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name": "manual_test_tenant", "provider": "local"}')

# Extract ID properly. The response structure for create_tenant might be the Tenant object directly.
# Let's assume standard response based on models.
TENANT_ID=$(echo $TENANT_RESP | python3 -c "import sys, json; print(json.load(sys.stdin)['id'])")

if [ -z "$TENANT_ID" ] || [ "$TENANT_ID" == "None" ]; then
    echo "Failed to create tenant. Response: $TENANT_RESP"
    exit 1
fi
echo "Tenant Created: $TENANT_ID"

echo "3. Creating Tenant Admin 'manual_admin'..."
ADMIN_CREATE_RESP=$(curl -s -X POST "$API_URL/api/v1/users" \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"manual_admin\", \"email\": \"admin@manual.test\", \"password\": \"password\", \"role\": \"tenant-admin\", \"tenant_id\": \"$TENANT_ID\"}")

echo "Tenant Admin Creation Response: $ADMIN_CREATE_RESP"

echo "4. Logging in as 'manual_admin'..."
ADMIN_LOGIN_RESP=$(curl -s -X POST "$API_URL/api/v1/users/login" \
  -H "Content-Type: application/json" \
  -d '{"username": "manual_admin", "password": "password"}')

ADMIN_TOKEN=$(echo $ADMIN_LOGIN_RESP | python3 -c "import sys, json; print(json.load(sys.stdin)['token'])")

if [ -z "$ADMIN_TOKEN" ] || [ "$ADMIN_TOKEN" == "None" ]; then
    echo "Failed to login as manual_admin. Response: $ADMIN_LOGIN_RESP"
    exit 1
fi
echo "Tenant Admin Token acquired."

echo "5. Creating Tenant User 'manual_user'..."
USER_CREATE_RESP=$(curl -s -X POST "$API_URL/api/v1/users" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"manual_user\", \"email\": \"user@manual.test\", \"password\": \"password\", \"role\": \"tenant-user\", \"tenant_id\": \"$TENANT_ID\"}")

echo "Tenant User Creation Response: $USER_CREATE_RESP"

echo ""
echo "=== SEEDING COMPLETE ==="
echo "Tenant: manual_test_tenant ($TENANT_ID)"
echo "Admin: manual_admin / password"
echo "User: manual_user / password"
