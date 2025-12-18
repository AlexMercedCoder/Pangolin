#!/bin/bash
API_URL="http://localhost:8080"
ROOT_USER="admin"
ROOT_PASS="password"

echo "1. Login Root"
TOKEN=$(curl -s -X POST $API_URL/api/v1/users/login -H "Content-Type: application/json" -d "{\"username\": \"$ROOT_USER\", \"password\": \"$ROOT_PASS\"}" | jq -r .token)
echo "Root Token: $TOKEN"

echo "2. Create Tenant"
TENANT_ID=$(curl -s -X POST $API_URL/api/v1/tenants -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d '{"name": "curl_tenant"}' | jq -r .id)
echo "Tenant ID: $TENANT_ID"

echo "3. Create User (Role: TenantAdmin)"
# Note: key is "role": "TenantAdmin"
RESP=$(curl -s -w "\nHTTP_STATUS:%{http_code}" -X POST $API_URL/api/v1/users \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"curl_admin\", \"password\": \"password\", \"email\": \"curl@example.com\", \"tenant_id\": \"$TENANT_ID\", \"role\": \"tenant-admin\"}")
echo "Register Response: $RESP"

echo "4. Login User"
LOGIN_RESP=$(curl -s -w "\nHTTP_STATUS:%{http_code}" -X POST $API_URL/api/v1/users/login \
  -H "Content-Type: application/json" \
  -d '{"username": "curl_admin", "password": "password"}')
echo "Login Response: $LOGIN_RESP"
