
import requests
import json
from pyiceberg.catalog import load_catalog
import pyarrow as pa
import time

API_URL = "http://localhost:8080"
MANAGEMENT_URL = f"{API_URL}/api/v1"
TENANT_ID = "00000000-0000-0000-0000-000000000000"
CATALOG_NAME = "my_catalog"

# Headers for management API (Tenant ID required)
headers = {
    "X-Pangolin-Tenant": TENANT_ID,
    "Content-Type": "application/json"
}

print(f"Waiting for API at {API_URL}...")
for i in range(10):
    try:
        requests.get(f"{API_URL}/health")
        break
    except requests.exceptions.ConnectionError:
        print(f"Waiting... {i}")
        time.sleep(2)

# 1. Create a Warehouse (needed for Catalog)
print("Creating Warehouse 'my_warehouse'...")
warehouse_payload = {
    "name": "my_warehouse",
    "storage_type": "s3",
    "storage_config": {
        "endpoint": "http://minio:9000", # Internal Docker URL
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
    print(f"Warehouse creation status: {resp.status_code} (might already exist)")

# 2. Create the Catalog
print(f"Creating Catalog '{CATALOG_NAME}'...")
catalog_payload = {
    "name": CATALOG_NAME,
    "role_arn": None, # Not used for now
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
    # Proceed anyway, maybe it exists

# 3. Use PyIceberg
# Note: For PyIceberg running on HOST, we need to point S3 to localhost:9000
# But the URL in the RestCatalog config (returned by API) might point to internal docker DNS
# checking main.rs/iceberg_handlers, the config endpoint returns what we specified in environment S3_ENDPOINT?
# OR we override it in load_catalog properties.

print("Initializing PyIceberg...")

catalog = load_catalog(
    "local",
    **{
        "type": "rest",
        "uri": f"{API_URL}/v1/{CATALOG_NAME}",
        "header.X-Pangolin-Tenant": TENANT_ID,
        # Override S3 endpoint for localhost access
        "s3.endpoint": "http://localhost:9000", 
        "s3.access-key-id": "minioadmin",
        "s3.secret-access-key": "minioadmin",
        "s3.region": "us-east-1"
    }
)

print("Listing namespaces...")
try:
    print(catalog.list_namespaces())
except Exception as e:
    print(f"List namespaces failed: {e}")
    exit(1)

print("Creating namespace 'test_ns'...")
try:
    catalog.create_namespace("test_ns")
except Exception:
    pass

print("Creating table 'test_ns.test_table'...")
schema = pa.schema([
    ("id", pa.int64()),
    ("data", pa.string())
])

try:
    auth_table = catalog.create_table(
        "test_ns.test_table",
        schema=schema
    )
    print(f"Table created: {auth_table}")
except Exception as e:
    print(f"Table creation/load failed: {e}")
    try:
        auth_table = catalog.load_table("test_ns.test_table")
        print(f"Table loaded: {auth_table}")
    except Exception as e2:
        print(f"Final failure: {e2}")

print("Test Complete.")
