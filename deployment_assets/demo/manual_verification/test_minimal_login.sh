#!/bin/bash
# Minimal test to debug tenant-scoped login

API_URL="http://localhost:8080"

echo "=== Minimal Tenant Login Test ==="

# 1. Root login
echo "1. Root login..."
ROOT_TOKEN=$(curl -s -X POST "$API_URL/api/v1/users/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"password","tenant-id":null}' | jq -r '.token')
echo "Root token: ${ROOT_TOKEN:0:20}..."

# 2. Create tenant
echo "2. Creating tenant..."
TENANT_ID=$(curl -s -X POST "$API_URL/api/v1/tenants" \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"test_tenant","properties":{}}' | jq -r '.id')
echo "Tenant ID: $TENANT_ID"

# 3. Create user
echo "3. Creating tenant admin user..."
CREATE_RESPONSE=$(curl -s -X POST "$API_URL/api/v1/users" \
  -H "Authorization: Bearer $ROOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"testadmin\",\"email\":\"admin@test.com\",\"password\":\"testpass123\",\"tenant_id\":\"$TENANT_ID\",\"role\":\"tenant-admin\"}")
echo "Create response: $CREATE_RESPONSE"

# 4. Try to login
echo "4. Attempting tenant-scoped login..."
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/api/v1/users/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"testadmin\",\"password\":\"testpass123\",\"tenant-id\":\"$TENANT_ID\"}")
echo "Login response: $LOGIN_RESPONSE"

# Check if we got a token
TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.token // empty')
if [ -n "$TOKEN" ]; then
  echo "✅ Login successful! Token: ${TOKEN:0:20}..."
else
  echo "❌ Login failed"
  echo "Full response: $LOGIN_RESPONSE"
fi
