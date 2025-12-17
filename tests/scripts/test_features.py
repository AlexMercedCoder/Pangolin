
import requests
import json
import time
import sys

# Configuration
API_URL = "http://localhost:8080"
TENANT_ID = "00000000-0000-0000-0000-000000000000"
USER_ID = "15da64d6-98d1-4a2e-94ee-b3467cca7a55" # ID from prev logs or dynamic fetch
CATALOG_NAME = "feature_catalog"

headers = {
    "Content-Type": "application/json",
    "X-Pangolin-Tenant": TENANT_ID
}

def wait_for_api():
    print(f"Waiting for API at {API_URL}...")
    for _ in range(30):
        try:
            resp = requests.get(f"{API_URL}/health")
            if resp.status_code == 200:
                print("API is up!")
                return
        except requests.exceptions.ConnectionError:
            pass
        time.sleep(1)
    print("API failed to start.")
    sys.exit(1)

def setup_catalog():
    print("Creating Warehouse and Catalog...")
    # Warehouse
    requests.post(f"{API_URL}/api/v1/warehouses", json={
        "name": "feature_warehouse",
        "storage_config": {
            "type": "s3",
            "s3.endpoint": "http://localhost:9000",
            "s3.bucket": "warehouse"
        }
    }, headers=headers)
    
    # Catalog
    requests.post(f"{API_URL}/api/v1/catalogs", json={
        "name": CATALOG_NAME,
        "catalog_type": "managed",
        "warehouse_name": "feature_warehouse"
    }, headers=headers)

def test_branching():
    print("\n--- Testing Branching ---")
    
    # Create Branch
    payload = {
        "name": "dev_branch",
        "branch_type": "experimental",
        "catalog": CATALOG_NAME,
        "assets": []
    }
    resp = requests.post(f"{API_URL}/api/v1/branches", json=payload, headers=headers)
    if resp.status_code in [200, 201]:
        print("Branch 'dev_branch' created.")
    else:
        print(f"Failed to create branch: {resp.status_code} {resp.text}")
        sys.exit(1)

    # List Branches
    resp = requests.get(f"{API_URL}/api/v1/branches?catalog={CATALOG_NAME}", headers=headers)
    if resp.status_code == 200:
        branches = resp.json()
        print(f"List Branches: {[b['name'] for b in branches]}")
        if not any(b['name'] == 'dev_branch' for b in branches):
            print("Target branch not found in list.")
            sys.exit(1)
    else:
        print(f"Failed to list branches: {resp.status_code} {resp.text}")
        sys.exit(1)

def test_permissions():
    print("\n--- Testing Permissions ---")
    
    # Create Role
    role_payload = {
        "name": "DataAnalyst",
        "description": "Can read data",
        "tenant-id": TENANT_ID
    }
    resp = requests.post(f"{API_URL}/api/v1/roles", json=role_payload, headers=headers)
    if resp.status_code in [200, 201]:
        role = resp.json()
        role_id = role['id']
        print(f"Role 'DataAnalyst' created (ID: {role_id}).")
    else:
        print(f"Failed to create role: {resp.status_code} {resp.text}")
        sys.exit(1)
        
    # Get Current User ID (Tenant Admin)
    resp = requests.get(f"{API_URL}/api/v1/users/me", headers=headers)
    if resp.status_code == 200:
       user_id = resp.json()['id']
       print(f"Target User ID: {user_id}")
    else:
       # Fallback to listing users
       resp = requests.get(f"{API_URL}/api/v1/users", headers=headers)
       user_id = resp.json()[0]['id']
       print(f"Fetched User ID from list: {user_id}")

    # Assign Role
    assign_payload = {"role-id": role_id}
    resp = requests.post(f"{API_URL}/api/v1/users/{user_id}/roles", json=assign_payload, headers=headers)
    if resp.status_code in [200, 201]:
        print(f"Role assigned to user {user_id}.")
    else:
        print(f"Failed to assign role: {resp.status_code} {resp.text}")
        sys.exit(1)

    # Verify Assignment
    resp = requests.get(f"{API_URL}/api/v1/users/{user_id}/roles", headers=headers)
    if resp.status_code == 200:
        roles = resp.json()
        print(f"User Roles: {[r['name'] for r in roles]}")
        if not any(r['id'] == role_id for r in roles):
             print("Assigned role not found on user.")
             sys.exit(1)
    else:
         print(f"Failed to get user roles: {resp.status_code} {resp.text}")

def test_business_metadata():
    print("\n--- Testing Business Metadata ---")
    asset_id = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"
    
    # 1. Add Metadata
    payload = {
        "description": "Test metadata description",
        "tags": ["pii", "bronze"],
        "properties": {"data_steward": "alex"},
        "discoverable": True
    }
    
    resp = requests.post(f"{API_URL}/api/v1/assets/{asset_id}/metadata", json=payload, headers=headers)
    if resp.status_code == 200:
        print("Business Metadata added.")
    else:
        print(f"Failed to add metadata: {resp.status_code} {resp.text}")
        sys.exit(1)

    # 2. Get Metadata
    resp = requests.get(f"{API_URL}/api/v1/assets/{asset_id}/metadata", headers=headers)
    if resp.status_code == 200:
         meta = resp.json()['metadata']
         print(f"Retrieved Metadata: Tags={meta['tags']}, Desc={meta['description']}")
         if "pii" not in meta['tags']:
             print("Tags mismatch.")
             sys.exit(1)
    else:
        print(f"Failed to get metadata: {resp.status_code} {resp.text}")
        sys.exit(1)

if __name__ == "__main__":
    wait_for_api()
    setup_catalog()
    test_branching()
    test_permissions() # This will get the user ID for us
    test_business_metadata()
    print("\nSUCCESS: All features verified!")
