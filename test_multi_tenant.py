
import requests
from pyiceberg.catalog import load_catalog
import pyarrow as pa
import time

API_URL = "http://localhost:8080"
MANAGEMENT_URL = f"{API_URL}/api/v1"
USERNAME = "admin"
PASSWORD = "password"
CATALOG_NAME = "my_catalog"

print(f"Waiting for API at {API_URL}...")
for i in range(10):
    try:
        requests.get(f"{API_URL}/health")
        break
    except requests.exceptions.ConnectionError:
        print(f"Waiting... {i}")
        time.sleep(2)

print("Authenticating...")
try:
    # 1. Login to get token
    # Login endpoint: /api/v1/users/login (based on lib.rs line 138)
    # Checks user_handlers.rs for payload: likely {username, password, tenant_id?}
    # main.rs created 'admin' in default tenant.
    
    response = requests.post(f"{API_URL}/api/v1/users/login", json={
        "username": USERNAME,
        "password": PASSWORD
    })
    
    if response.status_code != 200:
        print(f"Login failed: {response.text}")
        exit(1)
    
    data = response.json()
    token = data.get("token")
    # Default admin is in default tenant usually? 
    # Or in main.rs: "Tenant ID: 0000..." for default.
    # The login response probably refers to the User's tenant.
    # Let's decode or assume default.
    tenant_id = "00000000-0000-0000-0000-000000000000" 
    
    print(f"Logged in. Token: {token[:10]}...")

except Exception as e:
    print(f"Auth failed: {e}")
    exit(1)

headers = {
    "Authorization": f"Bearer {token}",
    "X-Pangolin-Tenant": tenant_id,
    "Content-Type": "application/json"
}

# 2. Create Warehouse
print("Creating Warehouse 'my_warehouse'...")
warehouse_payload = {
    "name": "my_warehouse",
    "storage_type": "s3",
    "storage_config": {
        "endpoint": "http://minio:9000",
        "bucket": "warehouse",
        "region": "us-east-1",
        "access_key_id": "minioadmin",
        "secret_access_key": "minioadmin"
    }
}
resp = requests.post(f"{MANAGEMENT_URL}/warehouses", headers=headers, json=warehouse_payload)
if resp.status_code in [200, 201]:
    print("Warehouse created.")
else:
    print(f"Warehouse creation status: {resp.status_code}")

# 3. Create Catalog
print(f"Creating Catalog '{CATALOG_NAME}'...")
catalog_payload = {
    "name": CATALOG_NAME,
    "role_arn": None,
    "catalog_type": "managed",
    "warehouse_name": "my_warehouse"
}
resp = requests.post(f"{MANAGEMENT_URL}/catalogs", headers=headers, json=catalog_payload)
if resp.status_code in [200, 201]:
    print("Catalog created.")
elif resp.status_code == 409:
    print("Catalog already exists.")
else:
    print(f"Catalog creation failed: {resp.status_code} {resp.text}")

# 4. Use PyIceberg
print("Initializing PyIceberg...")
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
        "s3.region": "us-east-1"
    }
)

print("Listing namespaces...")
print(catalog.list_namespaces())

print("Creating namespace 'multi_ns'...")
try:
    catalog.create_namespace("multi_ns")
except Exception:
    pass

print("Creating table 'multi_ns.multi_table'...")
schema = pa.schema([
    ("id", pa.int64()),
    ("info", pa.string())
])

try:
    tbl = catalog.create_table("multi_ns.multi_table", schema=schema)
    print(f"Table created: {tbl}")
except Exception:
    tbl = catalog.load_table("multi_ns.multi_table")
    print(f"Table loaded: {tbl}")

print("Test Complete.")
