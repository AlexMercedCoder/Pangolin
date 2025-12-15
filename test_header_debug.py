import urllib.request
import json
import uuid

BASE_URL = "http://localhost:8080/api/v1"

def login():
    req = urllib.request.Request(
        f"{BASE_URL}/users/login",
        data=json.dumps({"username": "admin", "password": "password"}).encode('utf-8'),
        headers={'Content-Type': 'application/json'}
    )
    with urllib.request.urlopen(req) as response:
        data = json.loads(response.read().decode('utf-8'))
        return data['token']

def list_catalogs_with_header(token, tenant_id):
    req = urllib.request.Request(
        f"{BASE_URL}/catalogs",
        headers={
            'Authorization': f'Bearer {token}',
            'X-Pangolin-Tenant': str(tenant_id)
        }
    )
    with urllib.request.urlopen(req) as response:
        return json.loads(response.read().decode('utf-8'))

token = login()
print(f"Token: {token[:20]}...")

# Test with Nil UUID (default)
nil_uuid = "00000000-0000-0000-0000-000000000000"
print(f"\n1. Listing with Nil UUID ({nil_uuid}):")
catalogs = list_catalogs_with_header(token, nil_uuid)
print(f"   Found {len(catalogs)} catalogs: {[c['name'] for c in catalogs]}")

# Test with random UUID A
uuid_a = str(uuid.uuid4())
print(f"\n2. Listing with UUID A ({uuid_a}):")
catalogs_a = list_catalogs_with_header(token, uuid_a)
print(f"   Found {len(catalogs_a)} catalogs: {[c['name'] for c in catalogs_a]}")

# Test with random UUID B
uuid_b = str(uuid.uuid4())
print(f"\n3. Listing with UUID B ({uuid_b}):")
catalogs_b = list_catalogs_with_header(token, uuid_b)
print(f"   Found {len(catalogs_b)} catalogs: {[c['name'] for c in catalogs_b]}")

print(f"\n=== ANALYSIS ===")
print(f"All three requests returned the same catalogs: {catalogs == catalogs_a == catalogs_b}")
print(f"This means the X-Pangolin-Tenant header is being IGNORED")
