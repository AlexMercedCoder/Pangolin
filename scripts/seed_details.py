
import requests
import time
import sys

API_URL = "http://localhost:8080/api/v1"
ROOT_USER = "admin"
ROOT_PASS = "password"  # Matches docker-compose.yml

def login(username, password, tenant_id=None):
    payload = {"username": username, "password": password}
    if tenant_id:
        payload["tenant_id"] = tenant_id

    try:
        resp = requests.post(f"{API_URL}/users/login", json=payload)
        if resp.status_code == 200:
            return resp.json().get("token")
    except Exception as e:
        print(f"Connection error: {e}")
    return None

def main():
    print("Waiting for API to be healthy...")
    for i in range(30):
        try:
            if login(ROOT_USER, ROOT_PASS):
                print("API is up!")
                break
        except:
            pass
        time.sleep(1)
        print(".", end="", flush=True)

    root_token = login(ROOT_USER, ROOT_PASS)
    if not root_token:
        print("Root login failed.")
        sys.exit(1)

    # Create Tenant
    headers = {"Authorization": f"Bearer {root_token}"}
    tenant_name = "TroubleshootCorp"
    
    # 1. Create Tenant
    print(f"Creating tenant {tenant_name}...")
    resp = requests.post(
        f"{API_URL}/tenants", 
        json={"name": tenant_name, "description": "Troubleshoot Tenant"}, 
        headers=headers
    )
    if resp.status_code != 201:
        print(f"Failed to create tenant: {resp.text}")
        sys.exit(1)
    
    tenant_id = resp.json()['id']
    
    # 2. Create Tenant Admin
    admin_user = "trouble_admin"
    admin_pass = "password123"
    
    print(f"Creating admin {admin_user}...")
    resp = requests.post(
        f"{API_URL}/users",
        json={
            "username": admin_user,
            "password": admin_pass,
            "tenant_id": tenant_id,
            "role": "TenantAdmin",
            "email": "admin@troubleshoot.com"
        },
        headers=headers
    )
    
    if resp.status_code != 201:
        print(f"Failed to create admin: {resp.text}")
        sys.exit(1)

    print("\n" + "="*50)
    print("ENVIRONMENT READY 🚀")
    print(f"Tenant:         {tenant_name}")
    print(f"Tenant ID:      {tenant_id}")
    print(f"Admin User:     {admin_user}")
    print(f"Admin Password: {admin_pass}")
    print("="*50)

if __name__ == "__main__":
    main()
