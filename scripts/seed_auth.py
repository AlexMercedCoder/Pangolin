
import requests
import json
import time

API_URL = "http://localhost:8080/api/v1"
# Login as Root (configured via env vars)
ROOT_USER = "admin"
ROOT_PASS = "password123"

def login(username, password, tenant_id=None):
    payload = {"username": username, "password": password}
    if tenant_id:
        payload["tenant_id"] = tenant_id
        
    print(f"Logging in as {username}...")
    resp = requests.post(f"{API_URL}/users/login", json=payload)
    if resp.status_code != 200:
        print(f"Login failed: {resp.text}")
        return None
    return resp.json().get("token")

def create_tenant(token, name, description):
    headers = {"Authorization": f"Bearer {token}"}
    payload = {"name": name, "description": description}
    print(f"Creating tenant {name}...")
    resp = requests.post(f"{API_URL}/tenants", json=payload, headers=headers)
    if resp.status_code == 201:
        return resp.json()
    print(f"Create tenant failed: {resp.text}")
    return None

def create_user(token, username, password, tenant_id):
    headers = {"Authorization": f"Bearer {token}"}
    payload = {
        "username": username,
        "password": password,
        "tenant_id": tenant_id,
        "role": "tenant-admin",
        "email": f"{username}@example.com"
    }
    print(f"Creating user {username}...")
    resp = requests.post(f"{API_URL}/users", json=payload, headers=headers)
    if resp.status_code == 201:
        return resp.json()
    print(f"Create user failed: {resp.text}")
    return None

def main():
    # Wait for server to be ready
    print("Waiting for server...")
    time.sleep(5)
    
    # 1. Login as Root
    root_token = login(ROOT_USER, ROOT_PASS)
    if not root_token:
        print("Root login failed. Aborting.")
        return

    # 2. Create Tenant A
    tenant_a = create_tenant(root_token, "Acme Corp", "Primary seeding tenant")
    if not tenant_a: return
    tenant_a_id = tenant_a['id']

    # 3. Create Tenant Admin for A
    create_user(root_token, "acme_admin", "password123", tenant_a_id)

    # 4. Create Tenant B
    tenant_b = create_tenant(root_token, "Stark Ind", "Secondary seeding tenant")
    if not tenant_b: return
    tenant_b_id = tenant_b['id']

    # 5. Create Tenant Admin for B
    create_user(root_token, "stark_admin", "password123", tenant_b_id)

    print("\n--- SEEDING COMPLETE ---")
    print("Credentials:")
    print(f"1. Root: {ROOT_USER} / {ROOT_PASS}")
    print(f"2. Acme Admin: acme_admin / password123 (Tenant: Acme Corp)")
    print(f"3. Stark Admin: stark_admin / password123 (Tenant: Stark Ind)")

if __name__ == "__main__":
    main()
