import requests
import json
import time

BASE_URL = "http://localhost:8080/api/v1"

def login(username, password):
    resp = requests.post(f"{BASE_URL}/users/login", json={"username": username, "password": password})
    if resp.status_code != 200:
        print(f"Login failed: {resp.text}")
        return None
    return resp.json()["token"]

def run_verification():
    print("1. Login as Root to discover state...")
    # Root login
    root_token = login("admin", "password")
    if not root_token:
        print("Failed to login as root. Check server status.")
        return
    headers = {"Authorization": f"Bearer {root_token}"}

    # 2. Find Tenant A
    print("2. Listing Tenants...")
    resp = requests.get(f"{BASE_URL}/tenants", headers=headers)
    if resp.status_code != 200:
        print(f"Failed to list tenants: {resp.text}")
        return
    
    tenants = resp.json()
    tenant_a = next((t for t in tenants if "tenant_a" in t["name"]), None)
    if not tenant_a:
        print("Could not find 'tenant_a' from restore_state.py!")
        return
    
    print(f"Found Tenant: {tenant_a['name']} ({tenant_a['id']})")
    tenant_id = tenant_a['id']
    
    # 3. Find User A and Admin A
    print("3. Listing Users...")
    # Root can list users for a tenant
    # Header format for tenant extension might be needed or passed in query?
    # Actually list_users endpoint takes Extension(TenantId). 
    # Root can assume tenant identity via header X-Pangolin-Tenant
    headers["X-Pangolin-Tenant"] = tenant_id
    
    resp = requests.get(f"{BASE_URL}/users", headers=headers)
    if resp.status_code != 200:
        print(f"Failed to list users: {resp.text}")
        return
        
    users = resp.json()
    user_a = next((u for u in users if "user_a" in u["username"]), None)
    admin_a = next((u for u in users if "admin_a" in u["username"]), None)
    
    if not user_a or not admin_a:
        print("Could not find user_a or admin_a!")
        return
        
    print(f"Found User A: {user_a['username']}")
    print(f"Found Admin A: {admin_a['username']}")
    
    # 4. Login as Admin A to find Asset ID
    print("4. Logging in as Admin A...")
    admin_token = login(admin_a['username'], "password")
    if not admin_token: return
    admin_headers = {"Authorization": f"Bearer {admin_token}"}
    
    print("5. Searching for 'table_allow' as Admin...")
    resp = requests.get(f"{BASE_URL}/assets/search?query=table_allow", headers=admin_headers)
    results = resp.json()
    if not results:
        print("Admin could not find 'table_allow'!")
        return
        
    asset = results[0]
    asset_id = asset['id']
    print(f"Found Asset: {asset['name']} ({asset_id})")
    
    # 5b. Verify Permissions exist? 
    # Logic: restore_state.py SHOULD have granted them? 
    # Wait, restore_state.py creates users/tables but DOES IT GRANT PERMISSIONS?
    # Reviewing restore_state.py logic from memory: It creates users/tables.
    # It does NOT seem to look like it grants specific asset permissions for `table_allow`.
    # It just creates the table. The UI TEST was supposed to grant them.
    # SO, we must GRANT them now as Admin A.
    
    print("6. Granting Asset Permission to User A...")
    # catalog_a should exist
    catalogs = requests.get(f"{BASE_URL}/catalogs", headers=admin_headers).json()
    catalog_a = next((c for c in catalogs if "catalog_a" in c["name"]), None)
    if not catalog_a:
        print("Catalog A not found!")
        return

    grant_payload = {
        "user-id": user_a['id'],
        "scope": {
            "type": "asset",
            "catalog_id": catalog_a['id'],
            "namespace": "data",
            "asset_id": asset_id
        },
        "actions": ["read"]
    }
    
    # Permission endpoints: try POST /api/v1/permissions (create) vs PUT /roles? 
    # Pangolin uses `POST /api/v1/permissions` to grant direct permission if supported,
    # or `POST /api/v1/roles/{role_id}/permissions`?
    # Let's check `authz.rs` or known endpoints. 
    # Actually, the UI uses `POST /api/v1/permissions`.
    
    resp = requests.post(f"{BASE_URL}/permissions", json=grant_payload, headers=admin_headers)
    if resp.status_code not in [200, 201]:
        print(f"Failed to grant permission: {resp.text}")
        return
    print("Permission Granted.")

    # 7. Login as User A and Search
    print("7. Logging in as User A...")
    user_token = login(user_a['username'], "password")
    if not user_token: return
    user_headers = {"Authorization": f"Bearer {user_token}"}
    
    print("8. Searching as User A...")
    resp = requests.get(f"{BASE_URL}/assets/search?query=table_allow", headers=user_headers)
    user_results = resp.json()
    
    print("\n--- RESULTS ---")
    found = False
    for res in user_results:
        print(f"- {res['name']} (ID: {res['id']}, Access: {res['has_access']})")
        if res['id'] == asset_id:
            found = True
            
    if found:
        print("\nSUCCESS: User found the asset via search!")
    else:
        print("\nFAILURE: User DID NOT find the asset via search.")

if __name__ == "__main__":
    run_verification()
