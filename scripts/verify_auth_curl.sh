#!/bin/bash
echo ">>> Testing X-API-Key Auth via Curl <<<"

# 1. Login
echo "Logging in..."
LOGIN_RES=$(curl -s -X POST http://localhost:8080/api/v1/users/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "password"}')
TOKEN=$(echo $LOGIN_RES | grep -o '"token":"[^"]*"' | cut -d":" -f2 | tr -d '"')

if [ -z "$TOKEN" ]; then
  echo "Login failed. Response: $LOGIN_RES"
  exit 1
fi
echo "Got Token."

# 2. Create Service User
echo "Creating Service User..."
SU_NAME="curl_bot_$(date +%s)"
SU_RES=$(curl -s -X POST http://localhost:8080/api/v1/service-users \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"name\": \"$SU_NAME\", \"role\": \"tenant-user\"}")

KEY=$(echo $SU_RES | grep -o '"api_key":"[^"]*"' | cut -d":" -f2 | tr -d '"')
ID=$(echo $SU_RES | grep -o '"service_user_id":"[^"]*"' | cut -d":" -f2 | tr -d '"')

if [ -z "$KEY" ]; then
  # Try fallback field mapping if needed, but grep should catch it if present
  # Actually json parsing with grep is brittle. Trying simple grep for key.
  KEY=$(echo $SU_RES | grep -o 'pgl_[a-zA-Z0-9]*')
fi

if [ -z "$KEY" ]; then
    echo "Failed to get API Key. Response: $SU_RES"
    exit 1
fi

echo "Got API Key: $KEY"

# 3. Test Auth
echo "Testing Warehouse List with API Key..."
# Use temp file to capture stderr for verbose headers
CURL_OUT=$(curl -v -s -w "\nHTTP_CODE:%{http_code}" -X GET http://localhost:8080/api/v1/warehouses \
  -H "X-API-Key: $KEY" 2>curl_verbose.log)

HTTP_CODE=$(echo "$CURL_OUT" | grep "HTTP_CODE" | cut -d":" -f2)
BODY=$(echo "$CURL_OUT" | grep -v "HTTP_CODE")

echo "Response Code: $HTTP_CODE"
echo "Response Body: $BODY"
echo "--- Curl Verbose Log ---"
cat curl_verbose.log
echo "------------------------"

# Cleanup
echo "Cleaning up..."
curl -s -X DELETE "http://localhost:8080/api/v1/service-users/$ID" \
  -H "Authorization: Bearer $TOKEN"

if [ "$HTTP_CODE" == "200" ]; then
    echo "✅ Success! Server accepts API Key."
else
    echo "❌ Failure! Server rejected API Key."
fi
