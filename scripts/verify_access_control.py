import requests
import json

BASE_URL = "http://localhost:8080/api/v1"
TENANT_ID = "b601090d-7bb5-40d0-b9db-82677c60daa1"

def verify_access_control():
    # 1. Login as Tenant Admin
    admin_resp = requests.post(f"{BASE_URL}/users/login", json={"username": "tenant_admin", "password": "password"})
    admin_token = admin_resp.json()["token"]
    admin_headers = {"Authorization": f"Bearer {admin_token}", "X-Pangolin-Tenant": TENANT_ID}

    # 2. Login as Limited User
    user_resp = requests.post(f"{BASE_URL}/users/login", json={"username": "limited_user", "password": "password"})
    if not user_resp.ok:
        print(f"Failed to login as limited_user: {user_resp.text}")
        return
    user_token = user_resp.json()["token"]
    user_headers = {"Authorization": f"Bearer {user_token}", "X-Pangolin-Tenant": TENANT_ID}
    
    user_id = user_resp.json()["user"]["id"]

    # 3. List Catalogs as Limited User (Expect 0)
    print("Listing catalogs as Limited User (Before Grant)...")
    resp = requests.get(f"{BASE_URL}/catalogs", headers=user_headers)
    catalogs = resp.json()
    print(f"Catalogs found: {len(catalogs)}")
    if len(catalogs) == 0:
        print("SUCCESS: 0 catalogs visible.")
    else:
        print(f"FAILURE: Expected 0, found {len(catalogs)}")

    # 4. Grant Read Permission on test-catalog
    # Need Catalog ID first
    cat_resp = requests.get(f"{BASE_URL}/catalogs/test-catalog", headers=admin_headers)
    cat_id = cat_resp.json()["id"]

    print("Granting Read permission on test-catalog...")
    grant_payload = {
        "user_id": user_id,
        "permission": "read",
        "scope": {
            "type": "catalog",
            "catalog_id": cat_id
        }
    }
    resp = requests.post(f"{BASE_URL}/permissions", json=grant_payload, headers=admin_headers)
    if resp.ok:
        print("Permission granted.")
    else:
        print(f"Failed to grant permission: {resp.text}")

    # 5. List Catalogs as Limited User (Expect 1)
    print("Listing catalogs as Limited User (After Grant)...")
    resp = requests.get(f"{BASE_URL}/catalogs", headers=user_headers)
    catalogs = resp.json()
    print(f"Catalogs found: {len(catalogs)}")
    if len(catalogs) >= 1: # Might be more if other tests ran, but at least test-catalog
        names = [c["name"] for c in catalogs]
        if "test-catalog" in names:
            print("SUCCESS: test-catalog visible.")
        else:
             print("FAILURE: test-catalog NOT visible.")
    else:
        print("FAILURE: Still 0 catalogs.")

if __name__ == "__main__":
    verify_access_control()
