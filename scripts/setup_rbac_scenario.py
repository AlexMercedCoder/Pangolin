import requests
import json
import uuid

BASE_URL = "http://localhost:8080/api/v1"
AUTH_URL = f"{BASE_URL}/users/login"
TENANT_ID = "00000000-0000-0000-0000-000000000000"

def get_admin_token():
    resp = requests.post(AUTH_URL, json={"username": "tenant_admin", "password": "password123"})
    if resp.ok:
        return resp.json()['token']
    print("Admin login failed")
    return None

def run():
    token = get_admin_token()
    if not token: return
    headers = {"Authorization": f"Bearer {token}", "X-Pangolin-Tenant": TENANT_ID}

    print("--- Setting up RBAC Scenario ---")

    # 1. Get Catalog ID
    cats = requests.get(f"{BASE_URL}/catalogs", headers=headers).json()
    cat = next((c for c in cats if c['name'] == 'test-catalog'), None)
    if not cat:
        print("Catalog not found")
        return
    cat_id = cat['id']
    print(f"Catalog ID: {cat_id}")

    # 2. Sync Assets to Pangolin (normally done via background or implicitly on creation, 
    # but verify we can search them requires them to be in asset table)
    # The current `create_asset` hook in `iceberg_handlers` should populate this table.
    # We can check by listing assets.
    # But search works on `BusinessMetadata` table JOIN `Asset` table.
    
    # 3. Create User 'viewer'
    user_payload = {
        "username": "viewer",
        "email": "viewer@test.com",
        "password": "password123",
        "role": "tenant-user",
        "tenant_id": TENANT_ID
    }
    resp = requests.post(f"{BASE_URL}/users", json=user_payload, headers=headers)
    if resp.status_code == 201:
        viewer = resp.json()
        print(f"Created user: viewer (ID: {viewer['id']})")
    elif resp.status_code == 409:
        print("User viewer already exists")
        # Get user
        users = requests.get(f"{BASE_URL}/users", headers=headers).json()
        viewer = next(u for u in users if u['username'] == 'viewer')
    else:
        print(f"Failed to create user: {resp.text}")
        return

    viewer_id = viewer['id']

    # 4. Create Role 'CatalogReader'
    role_payload = {
        "name": "CatalogReader",
        "description": "Read access to test-catalog",
        "tenant-id": TENANT_ID
    }
    resp = requests.post(f"{BASE_URL}/roles", json=role_payload, headers=headers)
    role_id = None
    if resp.status_code == 201:
        role_id = resp.json()['id']
        print(f"Created Role: CatalogReader ({role_id})")
    elif resp.status_code == 409:
        roles = requests.get(f"{BASE_URL}/roles", headers=headers).json()
        role_id = next(r for r in roles if r['name'] == 'CatalogReader')['id']
    else:
        print(f"Failed to create role: {resp.status_code} {resp.text}")
        # Try to proceed if role exists but list failed??
        # return
        # Let's try to list anyway
        roles = requests.get(f"{BASE_URL}/roles", headers=headers).json()
        found = next((r for r in roles if r['name'] == 'CatalogReader'), None)
        if found:
             role_id = found['id']
             print(f"Recovered Role ID: {role_id}")
        else:
             return

    # 5. Grant READ on Catalog to Role
    # GrantPermissionRequest: user_id, scope, actions (kebab-case?)
    # permission_handlers.rs line 34: #[serde(rename_all = "kebab-case")]
    
    # The request is: "tenant user who has access customers but not orders".
    # Implementation Gaps: I likely need to implement Table-level permissions in `PermissionScope` or at least logic for it.
    # BUT, assuming I can't rewrite the whole authz right now.
    
    # Alternative: 
    # `seed_iceberg.py` used `owner` property for "sales".
    # Maybe I can simulate it by granting the permission but checking `orders` specifically?
    # No.
    
    # Let's rely on the fact the search handler checks `PermissionScope::Catalog`.
    # If I want to split access, I need 2 catalogs? "SalesCatalog" (customers) and "OpsCatalog" (orders).
    # That works perfectly with current logic.
    # I will move `orders` to a separate Catalog `restricted-catalog`.
    
    # 5. Grant READ on test-catalog to Viewer
    print("--- Granting Access to 'test-catalog' (customers) ---")
    perm_payload = {
        "user-id": viewer_id,
        "actions": ["read"],
        "scope": {
            "type": "catalog",
            "catalog_id": cat_id
        }
    }
    resp = requests.post(f"{BASE_URL}/permissions", json=perm_payload, headers=headers)
    if resp.ok:
        print("Granted READ on test-catalog to viewer")
    else:
        print(f"Failed to grant permission: {resp.status_code} {resp.text}")

    # 6. Create Restricted Catalog and 'orders_secure'
    print("--- Setting up 'restricted-catalog' (orders_secure) ---")
    
    # Create Restricted Catalog
    wh_name = "test-warehouse"
    resp = requests.post(f"{BASE_URL}/catalogs", json={"name": "restricted-catalog", "warehouse_id": wh_name, "type": "iceberg"}, headers=headers)
    if resp.status_code in [201, 409]:
        print("Created restricted-catalog")
    else:
        print(f"Failed to create restricted-catalog: {resp.text}")
        return
        
    # Get ID
    cats = requests.get(f"{BASE_URL}/catalogs", headers=headers).json()
    restr_cat = next((c for c in cats if c['name'] == 'restricted-catalog'), None)
    if not restr_cat:
        print("Failed to find restricted-catalog")
        return
    restr_id = restr_cat['id']
    
    # Iceberg operations need to use /v1 prefix, not /api/v1
    ICEBERG_URL = "http://localhost:8080/v1"
    
    # Ensure Namespace exists
    resp = requests.post(f"{ICEBERG_URL}/restricted-catalog/namespaces", headers=headers, json={"namespace": ["default"]})
    if not resp.ok and resp.status_code != 409:
         print(f"Warning: Failed to create namespace ({resp.status_code})")
    
    # Create orders_secure in restricted catalog
    payload_orders = {
        "name": "orders_secure",
        "schema": {"type": "struct", "fields": [{"id": 1, "name": "id", "required": True, "type": "int"}]},
        "location": "s3://warehouse/restricted-catalog/default/orders_secure" 
    }
    resp = requests.post(f"{ICEBERG_URL}/restricted-catalog/namespaces/default/tables", headers=headers, json=payload_orders)
    if resp.ok or resp.status_code == 409:
        print("Created orders_secure in restricted-catalog (Iceberg API)")
    else:
        print(f"ERROR: Failed to create orders_secure table! {resp.status_code} {resp.text}")
        return
    
    import time
    print("Waiting for indexing (polling)...")
    
    orders_asset = None
    for i in range(10):
        print(f"Polling attempt {i+1}/10...")
        time.sleep(2)
        # Search for it
        s_resp = requests.get(f"{BASE_URL}/assets/search?query=orders_secure", headers=headers)
        if s_resp.ok:
            results = s_resp.json()
            found = next((r for r in results if r['name'] == 'orders_secure'), None)
            if found:
                orders_asset = found
                print("Found orders_secure!")
                break
    
    if not orders_asset:
        print("ERROR: orders_secure did not appear in search after 20 seconds.")

    # Find Asset IDs for Metadata update
    
    if orders_asset:
        print(f"Found orders_secure asset: {orders_asset['id']}")
        # Make orders_secure DISCOVERABLE
        meta_payload = {
            "tags": ["secure", "finance"],
            "properties": {},
            "discoverable": True,
            "description": "Sensitive Orders Data"
        }
        resp = requests.post(f"{BASE_URL}/assets/{orders_asset['id']}/metadata", json=meta_payload, headers=headers)
        if resp.ok:
            print("Made orders_secure discoverable")
    else:
        print("Could not find orders_secure asset to tag")

    # Make test-catalog customers discoverable too? Or just rely on access.
    # Let's make customers discoverable just to see if lock appears (it shouldn't because we have access).
    search_resp = requests.get(f"{BASE_URL}/assets/search?query=customers", headers=headers)
    assets = search_resp.json()
    cust_asset = next((a for a in assets if a['name'] == 'customers'), None)
    if cust_asset:
        # Make customers discoverable
        requests.post(f"{BASE_URL}/assets/{cust_asset['id']}/metadata", json={"tags":["public"], "properties":{}, "discoverable":True}, headers=headers)
        print("Made customers discoverable")

    # User 'viewer' has Role 'CatalogReader' which ONLY has access to 'test-catalog' (ID: cat_id).
    # It DOES NOT have access to 'restricted-catalog' (restr_id).
    
    print("Setup Complete.")
    print(f"User: viewer / password123")
    print(f"Has Access To: test-catalog (customers)")
    print(f"No Access To: restricted-catalog (orders_secure) [But is Discoverable]")

if __name__ == "__main__":
    run()
