import requests
import json
import sys

BASE_URL = "http://localhost:8080"
AUTH = ("admin", "password")

def get_token():
    resp = requests.post(f"{BASE_URL}/api/v1/users/login", json={"username": "admin", "password": "password"})
    if not resp.ok:
        print(f"Login failed: {resp.text}")
        sys.exit(1)
    return resp.json()["token"]

def verify_service_user():
    token = get_token()
    headers = {"Authorization": f"Bearer {token}"}
    
    # 1. Create Service User
    print("Creating Service User...")
    payload = {
        "name": "test-service-user-fix",
        "role": "tenant-user", # Kebab case
        "description": "Verifying fix"
    }
    resp = requests.post(f"{BASE_URL}/api/v1/service-users", json=payload, headers=headers)
    if not resp.ok:
        print(f"Failed to create service user: {resp.status_code} {resp.text}")
        sys.exit(1)
    
    su_id = resp.json()["service_user_id"]
    print(f"Created Service User: {su_id}")
    
    # 2. List Service Users
    print("Listing Service Users...")
    resp = requests.get(f"{BASE_URL}/api/v1/service-users", headers=headers)
    if not resp.ok:
        print(f"Failed to list service users: {resp.status_code} {resp.text}")
        sys.exit(1)
        
    users = resp.json()
    found = any(u["id"] == su_id for u in users)
    
    if found:
        print("PASS: Service User found in list.")
        # Cleanup
        requests.delete(f"{BASE_URL}/api/v1/service-users/{su_id}", headers=headers)
    else:
        print("FAIL: Service User NOT found in list.")
        sys.exit(1)

if __name__ == "__main__":
    try:
        verify_service_user()
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)
