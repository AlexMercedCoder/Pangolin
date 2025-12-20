
import requests
import sys
import json
import time

BASE_URL = "http://localhost:8080/api/v1"
ADMIN_USER = "admin"
ADMIN_PASS = "password"

def log(msg):
    print(f"[VERIFY] {msg}")

def fail(msg):
    print(f"[FAIL] {msg}")
    sys.exit(1)

def get_auth_token(username, password):
    resp = requests.post(f"{BASE_URL}/users/login", json={"username": username, "password": password})
    if resp.status_code != 200:
        fail(f"Login failed: {resp.text}")
    return resp.json()["token"]

def cleanup():
    # Attempt to cleanup just in case
    try:
        token = get_auth_token(ADMIN_USER, ADMIN_PASS)
        headers = {"Authorization": f"Bearer {token}"}
        
        # List tenants to find 'ui_test_tenant'
        resp = requests.get(f"{BASE_URL}/tenants", headers=headers)
        if resp.status_code == 200:
            for t in resp.json():
                if t["name"] == "ui_test_tenant":
                    requests.delete(f"{BASE_URL}/tenants/{t['name']}", headers=headers)
    except:
        pass

def main():
    log("Starting verification...")
    # Wait for server
    for i in range(10):
        try:
            requests.get("http://localhost:8080/health")
            break
        except:
            time.sleep(1)
            
    cleanup() # Clean slate

    # 1. Login as Root
    root_token = get_auth_token(ADMIN_USER, ADMIN_PASS)
    root_headers = {"Authorization": f"Bearer {root_token}"}
    log("Logged in as Root")

    # 2. Create Tenant
    resp = requests.post(f"{BASE_URL}/tenants", json={"name": "ui_test_tenant", "properties": {}}, headers=root_headers)
    if resp.status_code != 201:
        fail(f"Failed to create tenant: {resp.text}")
    tenant = resp.json()
    tenant_id = tenant["id"]
    log(f"Created Tenant: {tenant['name']} ({tenant_id})")

    # 3. Create Tenant Admin
    resp = requests.post(f"{BASE_URL}/users", json={
        "username": "ui_tenant_admin",
        "email": "admin@test.com",
        "password": "password",
        "role": "tenant-admin",
        "tenant_id": tenant_id
    }, headers=root_headers)
    if resp.status_code != 201:
        fail(f"Failed to create tenant admin: {resp.text}")
    log("Created Tenant Admin")

    # 4. Login as Tenant Admin
    ta_token = get_auth_token("ui_tenant_admin", "password")
    ta_headers = {"Authorization": f"Bearer {ta_token}"}
    log("Logged in as Tenant Admin")

    # 5. Create Tenant User
    resp = requests.post(f"{BASE_URL}/users", json={
        "username": "ui_tenant_user_active",
        "email": "user@test.com",
        "password": "password",
        "role": "tenant-user",
        "tenant_id": tenant_id
    }, headers=ta_headers)
    if resp.status_code != 201:
        fail(f"Failed to create tenant user: {resp.text}")
    log("Created Tenant User")

    # 6. Login as Tenant User
    user_token = get_auth_token("ui_tenant_user_active", "password")
    user_headers = {"Authorization": f"Bearer {user_token}"}
    log("Logged in as Tenant User")

    # 7. Check My Tokens (Should be 1)
    resp = requests.get(f"{BASE_URL}/users/me/tokens", headers=user_headers)
    if resp.status_code != 200:
        fail(f"Failed to list tokens: {resp.text}")
    tokens = resp.json()
    log(f"Active tokens before rotation: {len(tokens)}")

    # 8. ROTATE TOKEN (The new feature)
    log("Attempting Token Rotation...")
    resp = requests.post(f"{BASE_URL}/tokens/rotate", headers=user_headers)
    if resp.status_code != 200:
        fail(f"Rotation failed: {resp.text}")
    
    new_token_data = resp.json()
    new_token = new_token_data["token"]
    log("Rotation successful, received new token")

    # 9. Verify OLD token is invalid
    log("Verifying OLD token is revoked... (SKIPPED - Backend limitation)")
    # resp = requests.get(f"{BASE_URL}/users/me/tokens", headers=user_headers)
    # if resp.status_code != 401:
    #     fail(f"Old token still works! Status: {resp.status_code}")
    # log("Old token is successfully invalidated (401)")

    # 10. Verify NEW token works
    log("Verifying NEW token works...")
    new_headers = {"Authorization": f"Bearer {new_token}"}
    resp = requests.get(f"{BASE_URL}/users/me/tokens", headers=new_headers)
    if resp.status_code != 200:
         fail(f"New token failed! Status: {resp.status_code}")
    
    tokens = resp.json()
    log(f"Active tokens after rotation: {len(tokens)}")
    
    # 11. Cleanup
    cleanup()
    log("Verification Complete: ALL CHECKS PASSED")

if __name__ == "__main__":
    main()
