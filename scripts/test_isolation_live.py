
import urllib.request
import json
import uuid
import sys

BASE_URL = "http://localhost:8080/api/v1"

def login():
    req = urllib.request.Request(
        f"{BASE_URL}/users/login",
        data=json.dumps({"username": "admin", "password": "password"}).encode('utf-8'),
        headers={'Content-Type': 'application/json'}
    )
    try:
        with urllib.request.urlopen(req) as response:
            data = json.loads(response.read().decode('utf-8'))
            return data['token']
    except Exception as e:
        print(f"Login failed: {e}")
        try:
            # Try getting error body
            if hasattr(e, 'read'):
                print(e.read().decode())
        except:
            pass
        sys.exit(1)

def create_catalog(token, tenant_id, name):
    payload = {
        "name": name,
        "warehouse_name": None,
        "storage_location": "s3://bucket/path"
    }
    req = urllib.request.Request(
        f"{BASE_URL}/catalogs",
        data=json.dumps(payload).encode('utf-8'),
        headers={
            'Content-Type': 'application/json',
            'Authorization': f'Bearer {token}',
            'X-Pangolin-Tenant': str(tenant_id)
        }
    )
    try:
        with urllib.request.urlopen(req) as response:
            return response.status
    except urllib.error.HTTPError as e:
        print(f"Create catalog failed: {e.code} {e.read().decode()}")
        return e.code

def list_catalogs(token, tenant_id):
    req = urllib.request.Request(
        f"{BASE_URL}/catalogs",
        headers={
            'Authorization': f'Bearer {token}',
            'X-Pangolin-Tenant': str(tenant_id)
        }
    )
    try:
        with urllib.request.urlopen(req) as response:
            data = json.loads(response.read().decode('utf-8'))
            return data
    except urllib.error.HTTPError as e:
        print(f"List catalogs failed: {e.code} {e.read().decode()}")
        return []

token = login()
print(f"Logged in. Token: {token[:10]}...")

tenant_a = uuid.uuid4()
tenant_b = uuid.uuid4()
print(f"Tenant A: {tenant_a}")
print(f"Tenant B: {tenant_b}")

print("Creating 'cat_a' in Tenant A...")
create_catalog(token, tenant_a, "cat_a")

print("Listing Tenant A...")
list_a = list_catalogs(token, tenant_a)
print(f"Tenant A catalogs: {[c['name'] for c in list_a]}")

print("Listing Tenant B...")
list_b = list_catalogs(token, tenant_b)
print(f"Tenant B catalogs: {[c['name'] for c in list_b]}")

if len(list_a) >= 1 and len(list_b) == 0:
    print("SUCCESS: Isolation verified.")
elif len(list_a) > 0 and len(list_b) > 0:
    print("FAILURE: Tenant B sees catalogs!")
else:
    print("FAILURE: Catalog creation might have failed (A is empty).")
