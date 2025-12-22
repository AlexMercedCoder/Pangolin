import requests
import uuid
import sys
import time

# Configuration
API_URL = "http://localhost:8080"
ADMIN_TOKEN = sys.argv[1] if len(sys.argv) > 1 else None

if not ADMIN_TOKEN:
    print("Usage: python reproduce_branch_bug.py <token>")
    sys.exit(1)

headers = {
    "Authorization": f"Bearer {ADMIN_TOKEN}",
    "Content-Type": "application/json",
    "X-Pangolin-Tenant": "00000000-0000-0000-0000-000000000000"
}

# 1. Create a catalog
catalog_name = f"test_cat_{uuid.uuid4().hex[:8]}"
print(f"Creating catalog: {catalog_name}")
resp = requests.post(f"{API_URL}/api/v1/catalogs", headers=headers, json={
    "name": catalog_name,
    "catalog_type": "Local"
})
resp.raise_for_status()

# 2. Create branch 'dev'
print("Creating branch 'dev'")
resp = requests.post(f"{API_URL}/api/v1/branches", headers=headers, json={
    "name": "dev",
    "catalog": catalog_name,
    "from_branch": "main"
})
resp.raise_for_status()

# 3. Create table in 'main'
table_name = "test_table"
print(f"Creating table in 'main' ({catalog_name}.default.{table_name})")
resp = requests.post(f"{API_URL}/v1/{catalog_name}/namespaces/default/tables", headers=headers, json={
    "name": table_name,
    "schema": {
        "type": "struct",
        "fields": [
            {"id": 1, "name": "id", "required": True, "type": "int"}
        ]
    }
})
if resp.status_code not in [200, 201]:
    print(f"ERROR: Failed to create table in main: {resp.status_code} {resp.text}")
    sys.exit(1)

# 4. Create table in 'dev'
# The table can have a different schema or just be a separate entity
print(f"Creating table in 'dev' ({catalog_name}.default@{catalog_name}.{table_name})")
resp = requests.post(f"{API_URL}/v1/{catalog_name}/namespaces/default@dev/tables", headers=headers, json={
    "name": table_name,
    "schema": {
        "type": "struct",
        "fields": [
            {"id": 1, "name": "id", "required": True, "type": "int"},
            {"id": 2, "name": "name", "required": False, "type": "string"}
        ]
    }
})

if resp.status_code not in [200, 201]:
    print(f"ERROR: Failed to create table in dev: {resp.status_code} {resp.text}")
    sys.exit(1)

# 5. Verify Isolation
print("\n--- Verifying Isolation ---")

# GET from main
resp_main = requests.get(f"{API_URL}/v1/{catalog_name}/namespaces/default/tables/{table_name}", headers=headers)
# GET from dev
resp_dev = requests.get(f"{API_URL}/v1/{catalog_name}/namespaces/default@dev/tables/{table_name}", headers=headers)

if resp_main.status_code == 200 and resp_dev.status_code == 200:
    meta_main = resp_main.json()
    meta_dev = resp_dev.json()
    
    loc_main = meta_main.get("metadata-location")
    loc_dev = meta_dev.get("metadata-location")
    
    print(f"Main Metadata Location: {loc_main}")
    print(f"Dev Metadata Location:  {loc_dev}")
    
    if loc_main == loc_dev:
        print("\n!!! BUG DETECTED !!!")
        print("Both branches are pointing to the SAME metadata location. Overwrite suspected.")
        sys.exit(1)
    else:
        print("\nSUCCESS: Both branches have unique metadata locations.")
        
        # Verify schema differences if they exist in the metadata bytes actually stored
        # But here we just check if the metadata-location differs which is the first sign of isolation.
else:
    print(f"Verification FAILED to retrieve metadata:")
    print(f"  Main status: {resp_main.status_code} {resp_main.text}")
    print(f"  Dev status:  {resp_dev.status_code} {resp_dev.text}")
    sys.exit(1)
