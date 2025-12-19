import requests
import json
import uuid

# Configuration
API_URL = "http://localhost:8080/api/v1"
# Retrieve Tenant ID from somewhere or use the one from previous setups
# For robustness, we'll try to get it from a login if possible, or use a known one.
# But often Tenant Creation returns the ID.
# Let's assume we can use the default Tenant Admin logic.

def login(username, password):
    resp = requests.post(f"{API_URL}/users/login", json={"username": username, "password": password})
    if resp.ok:
        return resp.json()
    return None

def main():
    print("starting RBAC verification...")
    
    # 1. Login as Tenant Admin
    print("Logging in as Tenant Admin...")
    admin_creds = login("tenant_admin", "password")
    
    if not admin_creds:
        print("[FAIL] Could not login as tenant_admin. Run setup_test_env.py first.")
        return

    print(f"Login response keys: {admin_creds.keys()}")
    if "user" in admin_creds:
        print(f"User object: {admin_creds['user']}")
        
    admin_token = admin_creds["token"]
    
    # Try to get tenant_id from user object, or default to a known one if we can list tenants (as Root)
    tenant_id = None
    if "tenant_id" in admin_creds.get("user", {}):
        tenant_id = admin_creds["user"]["tenant_id"]
    elif "tenantId" in admin_creds.get("user", {}):
        tenant_id = admin_creds["user"]["tenantId"]
    elif "tenant-id" in admin_creds.get("user", {}):
        tenant_id = admin_creds["user"]["tenant-id"]
        
    if not tenant_id:
        # If root, we might not have a tenant_id in User object or it's None. 
        # But we need A tenant to test RBAC on.
        # Let's List Tenants and pick the first one.
        print("Fetching tenants list...")
        t_resp = requests.get(f"{API_URL}/tenants", headers={"Authorization": f"Bearer {admin_token}"})
        if t_resp.ok:
            tenants = t_resp.json()
            if tenants:
                tenant_id = tenants[0]["id"]
                print(f"Using first available tenant: {tenant_id}")
            else:
                 print("[FAIL] No tenants found.")
                 return
        else:
             print(f"[FAIL] Could not list tenants: {t_resp.text}")
             return

    # If tenant_id is still None (unlikely), error out
    if not tenant_id:
         print("[FAIL] Could not determine Tenant ID.")
         return

    admin_headers = {"Authorization": f"Bearer {admin_token}", "X-Pangolin-Tenant": tenant_id}

    # 2. Create Test Catalogs
    cat1_name = f"rbac-allowed-{uuid.uuid4().hex[:8]}"
    cat2_name = f"rbac-denied-{uuid.uuid4().hex[:8]}"
    
    print(f"Creating catalogs: {cat1_name}, {cat2_name}")
    c1_resp = requests.post(f"{API_URL}/catalogs", json={"name": cat1_name, "catalog_type": "Local", "warehouse_name": "test-warehouse"}, headers=admin_headers)
    if not c1_resp.ok:
        print(f"[FAIL] Failed to create {cat1_name}: {c1_resp.status_code} {c1_resp.text}")
        
    c2_resp = requests.post(f"{API_URL}/catalogs", json={"name": cat2_name, "catalog_type": "Local", "warehouse_name": "test-warehouse"}, headers=admin_headers)
    if not c2_resp.ok:
         print(f"[FAIL] Failed to create {cat2_name}: {c2_resp.status_code} {c2_resp.text}")

    # 3. Create Limited User
    user_name = f"rbac-user-{uuid.uuid4().hex[:8]}"
    print(f"Creating limited user: {user_name}")
    user_payload = {
        "username": user_name,
        "password": "password123", # Password used for new user login
        "role": "tenant-user",
        "tenant_id": tenant_id,
        "email": f"{user_name}@test.com"
    }
    resp = requests.post(f"{API_URL}/users", json=user_payload, headers=admin_headers)
    if not resp.ok:
        print(f"[FAIL] Creating user failed: {resp.text}")
        return
    user_id = resp.json()["id"]

    # 4. Grant READ on cat1
    cat1_resp = requests.get(f"{API_URL}/catalogs/{cat1_name}", headers=admin_headers)
    if not cat1_resp.ok:
         print(f"[FAIL] Could not get catalog {cat1_name}")
         return
    cat1_id = cat1_resp.json()["id"]

    print(f"Granting READ on {cat1_name} ({cat1_id}) to {user_name}")
    grant_payload = {
        "user_id": user_id,
        "actions": ["Read"],
        "scope": {
            "type": "catalog",
            "catalog_id": cat1_id
        }
    }
    # Note: Using "actions" list as per Permission struct, or "permission" helper? 
    # The API might expect "permission" (Action enum string) or "actions".
    # Checking `PermissionRequest` struct in `permission_handlers.rs` would be good.
    # But usually it's "action": "Read" or "actions": ["Read"]. 
    # Let's try "action": "Read" based on verify_access_control.py using "permission": "read".
    # Wait, verify_access_control.py used "permission": "read". 
    # Let's check `permission_handlers.rs` struct if possible.
    # Assuming "permission": "Read" works based on previous script.
    grant_payload_v2 = {
        "user-id": user_id,
        "actions": ["read"],
        "scope": {
            "type": "catalog",
            "catalog_id": cat1_id
        }
    }
    resp = requests.post(f"{API_URL}/permissions", json=grant_payload_v2, headers=admin_headers)
    if not resp.ok:
        print(f"Grant failed: {resp.text}. Trying 'read' lowercase.")
        grant_payload_v2["permission"] = "read"
        resp = requests.post(f"{API_URL}/permissions", json=grant_payload_v2, headers=admin_headers)
        if not resp.ok:
             print(f"[FAIL] Grant really failed: {resp.text}")
             return

    # 5. Login as Limited User
    print("Logging in as Limited User...")
    user_creds = login(user_name, "password123")
    if not user_creds:
        print("[FAIL] User login failed")
        return
    user_token = user_creds["token"]
    user_headers = {"Authorization": f"Bearer {user_token}", "X-Pangolin-Tenant": tenant_id}

    # 6. Verify List Catalogs
    print("Verifying List Catalogs...")
    list_resp = requests.get(f"{API_URL}/catalogs", headers=user_headers)
    if not list_resp.ok:
        print(f"[FAIL] List catalogs failed: {list_resp.text}")
        return
        
    catalogs = list_resp.json()
    cat_names = [c["name"] for c in catalogs]
    print(f"Visible catalogs: {cat_names}")
    
    if cat1_name in cat_names:
        print(f"[PASS] {cat1_name} is visible.")
    else:
        print(f"[FAIL] {cat1_name} is NOT visible.")
        
    if cat2_name not in cat_names:
        print(f"[PASS] {cat2_name} is NOT visible (Correct).")
    else:
        print(f"[FAIL] {cat2_name} IS visible (Security Vulnerability!).")

if __name__ == "__main__":
    main()
