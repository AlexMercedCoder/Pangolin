import requests
import sys
import os
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, LongType, StringType

API_URL = "http://localhost:8080"
MANAGEMENT_URL = f"{API_URL}/api/v1"

def authenticate(username, password):
    print(f"Authenticating as {username}...")
    try:
        response = requests.post(f"{API_URL}/api/v1/users/login", json={
            "username": username,
            "password": password
        })
        response.raise_for_status()
        data = response.json()
        return data["token"], data.get("tenant_id")
    except Exception as e:
        print(f"Auth failed: {e}")
        # Debug response if possible
        if 'response' in locals():
            print(response.text)
        sys.exit(1)

def debug_list_catalogs(token):
    headers = {"Authorization": f"Bearer {token}"}
    try:
        resp = requests.get(f"{MANAGEMENT_URL}/catalogs", headers=headers)
        resp.raise_for_status()
        print("Existing Catalogs:", resp.json())
    except Exception as e:
        print(f"Failed to list catalogs: {e}")

print(f"Waiting for API at {API_URL}...")
# Simple wait loop
import time
for i in range(10):
    try:
        requests.get(f"{API_URL}/health")
        break
    except:
        print(f"Waiting... {i}")
        time.sleep(1)

# 1. Authenticate as System Admin
print("Authenticating as System Admin...")
# Note: 'admin'/'password' are the default PANGOLIN_ROOT_USER creds
sys_token, _ = authenticate("admin", "password")
print("System Admin Authenticated.")

headers = {"Authorization": f"Bearer {sys_token}"}

# 2. Create Tenant
print("Creating Tenant 'demo_tenant'...")
tenant_payload = {
    "name": "demo_tenant",
    "properties": {}
}
resp = requests.post(f"{MANAGEMENT_URL}/tenants", headers=headers, json=tenant_payload)
if resp.status_code == 201:
    tenant_data = resp.json()
    tenant_id = tenant_data["id"]
    print(f"Tenant created: {tenant_id}")
elif resp.status_code == 409:
    # Try to find existing
    print("Tenant exists. Listing to find ID...")
    resp = requests.get(f"{MANAGEMENT_URL}/tenants", headers=headers)
    resp.raise_for_status()
    tenants = resp.json()
    for t in tenants:
        if t["name"] == "demo_tenant":
            tenant_id = t["id"]
            break
    else:
        print("Could not find tenant ID for 'demo_tenant'.")
        sys.exit(1)
    print(f"Found existing Tenant ID: {tenant_id}")
else:
    print(f"Tenant creation failed: {resp.status_code} {resp.text}")
    sys.exit(1)

# 3. Create Tenant Admin
print("Creating Tenant Admin 'tenant_admin'...")
user_payload = {
    "username": "tenant_admin",
    "email": "admin@demo.com",
    "password": "password",
    "role": "tenant-admin",
    "tenant_id": tenant_id
}
resp = requests.post(f"{MANAGEMENT_URL}/users", headers=headers, json=user_payload)
if resp.status_code in [201, 200]:
    print("Tenant Admin created.")
elif resp.status_code == 409:
    print("Tenant Admin already exists.")
else:
    print(f"User creation failed: {resp.status_code} {resp.text}")
    sys.exit(1)

# 4. Login as Tenant Admin
print("Authenticating as Tenant Admin...")
token, auth_tenant_id = authenticate("tenant_admin", "password")
print(f"Tenant Admin Authenticated. Tenant ID: {auth_tenant_id}")

if str(auth_tenant_id) != str(tenant_id):
    print(f"Warning: Auth Tenant ID {auth_tenant_id} != Created Tenant ID {tenant_id}")

tenant_headers = {"Authorization": f"Bearer {token}"}

# 5. Create Warehouse (as Tenant Admin)
print("Creating Warehouse 'my_warehouse'...")
warehouse_payload = {
    "name": "my_warehouse",
    "use_sts": False,
    "storage_config": {
        "type": "S3",
        "bucket": "warehouse",
        "region": "us-east-1",
        "access_key_id": "minioadmin",
        "secret_access_key": "minioadmin",
        "endpoint": "http://minio:9000"
    }
}
resp = requests.post(f"{MANAGEMENT_URL}/warehouses", headers=tenant_headers, json=warehouse_payload)
if resp.status_code in [200, 201]:
    print("Warehouse created.")
elif resp.status_code == 409:
    print("Warehouse already exists.")
else:
    print(f"Warehouse creation failed: {resp.status_code} {resp.text}")
    sys.exit(1)

# 6. Create Catalog
print("Creating Catalog 'my_catalog'...")
# Ensure catalog name matches what we access later
CATALOG_NAME = "my_catalog"
catalog_payload = {
    "name": CATALOG_NAME,
    "warehouse_name": "my_warehouse",
    "storage_location": "s3://warehouse/my_catalog",
    "catalog_type": "Local",
    "properties": {}
}
resp = requests.post(f"{MANAGEMENT_URL}/catalogs", headers=tenant_headers, json=catalog_payload)
if resp.status_code in [200, 201]:
    print("Catalog created.")
elif resp.status_code == 409:
    print("Catalog already exists.")
else:
    print(f"Catalog creation failed: {resp.status_code} {resp.text}")
    sys.exit(1)

# 7. Use PyIceberg
print(f"Initializing PyIceberg with URI: {API_URL}/v1/{CATALOG_NAME}")
try:
    catalog = load_catalog(
        "local",
        **{
            "type": "rest",
            "uri": f"{API_URL}/v1/{CATALOG_NAME}",
            "token": token,
            "header.X-Pangolin-Tenant": tenant_id,
            "s3.endpoint": "http://localhost:9000",
            "s3.access-key-id": "minioadmin",
            "s3.secret-access-key": "minioadmin",
            "s3.region": "us-east-1",
            "s3.path-style-access": "true"
        }
    )
    print("Listing namespaces...")
    print(catalog.list_namespaces())
    
    try:
        catalog.create_namespace("test_ns")
        print("Namespace 'test_ns' created")
    except Exception as e:
        print(f"Namespace creation note: {e}")
    
    schema = Schema(
        NestedField(1, "id", LongType()),
        NestedField(2, "data", StringType()),
    )
    try:
        table = catalog.create_table("test_ns.test_table", schema)
        print(f"Table created: {table}")
    except Exception as e:
        print(f"Table creation note: {e}")
        # Try loading
        table = catalog.load_table("test_ns.test_table")
        print(f"Table loaded: {table}")
    
except Exception as e:
    print(f"PyIceberg Error: {e}")
    print("Debugging Catalog List:")
    debug_list_catalogs(token)
    sys.exit(1)

print("Test Complete.")
