import requests
import json
import time

BASE_URL = "http://localhost:8080"
AUTH_URL = f"{BASE_URL}/api/v1/users/login"

def run():
    print("--- Ensuring Test Data Exists ---")
    
    # 1. Login as Tenant Admin
    print("Logging in as tenant_admin...")
    session = requests.Session()
    try:
        resp = session.post(AUTH_URL, json={"username": "tenant_admin", "password": "password"})
        if resp.status_code != 200:
            print(f"Login failed: {resp.status_code} {resp.text}")
            return
        
        data = resp.json()
        token = data['token']
        tenant_id = data['user']['tenant-id']
        headers = {
            "Authorization": f"Bearer {token}",
            "X-Pangolin-Tenant": tenant_id
        }
        print(f"Logged in. Tenant ID: {tenant_id}")
    except Exception as e:
        print(f"Login exception: {e}")
        return

    # 2. Check/Create Catalog
    catalog_name = "test-catalog"
    print(f"Checking catalog '{catalog_name}'...")
    # List catalogs
    resp = session.get(f"{BASE_URL}/api/v1/catalogs", headers=headers)
    catalogs = resp.json()
    if not any(c['name'] == catalog_name for c in catalogs):
        print(f"Catalog '{catalog_name}' not found. Creating...")
        # Create catalog logic if needed, but assuming it exists from previous steps
        # For brevity, if missing, we just warn, or try to create
        create_resp = session.post(f"{BASE_URL}/api/v1/catalogs", headers=headers, json={
            "name": catalog_name,
            "warehouse": "warehouse", 
            "type": "rest"
        })
        if create_resp.status_code in [200, 201]:
             print("Catalog created.")
        else:
             print(f"Failed to create catalog: {create_resp.text}")
             return
    else:
        print("Catalog exists.")

    # 3. Create Namespace (using Iceberg REST API via Pangolin)
    # Endpoint: POST /v1/{catalog}/namespaces
    print("Creating namespace 'default'...")
    ns_resp = session.post(
        f"{BASE_URL}/v1/{catalog_name}/namespaces",
        headers=headers,
        json={"namespace": ["default"]}
    )
    if ns_resp.status_code in [200, 201, 409]: # 409 = already exists
        print("Namespace 'default' ready.")
    else:
        print(f"Failed to create namespace: {ns_resp.status_code} {ns_resp.text}")
        # Continue anyway, might verify list

    # 4. Create Table
    print("Creating table 'test_table'...")
    table_payload = {
        "name": "test_table",
        "schema": {
            "type": "struct",
            "fields": [
                {"id": 1, "name": "id", "required": True, "type": "int"},
                {"id": 2, "name": "data", "required": False, "type": "string"}
            ]
        },
        "location": f"s3://warehouse/{catalog_name}/default/test_table" # simplified
    }
    
    # We use the standard Iceberg create table endpoint
    table_resp = session.post(
        f"{BASE_URL}/v1/{catalog_name}/namespaces/default/tables",
        headers=headers,
        json=table_payload
    )
    
    if table_resp.status_code in [200, 201]:
        print("Table 'test_table' created successfully.")
    elif table_resp.status_code == 409:
        print("Table 'test_table' already exists.")
    else:
        print(f"Failed to create table: {table_resp.status_code} {table_resp.text}")

if __name__ == "__main__":
    run()
