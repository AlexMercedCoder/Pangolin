import requests
import os

BASE_URL = "http://localhost:8080/api/v1"
ADMIN_AUTH = ("admin", "password")

def setup():
    # 1. Login to get token (Root or Tenant Admin)
    resp = requests.post(f"{BASE_URL}/users/login", json={"username": "admin", "password": "password"})
    if not resp.ok:
        print(f"Login (Root) failed: {resp.text}")
        return
    login_data = resp.json()
    token = login_data["token"]
    # Extract tenant_id from login response (UserSession usually contains it in user object or claims)
    # The login response has 'user': { ..., 'tenant_id': ... }
    initial_tenant_id = login_data.get('user', {}).get('tenant_id') or login_data.get('user', {}).get('tenant-id')
    headers = {"Authorization": f"Bearer {token}"}

    # 2. Create Tenant (Only if Root)
    tenant_id = initial_tenant_id
    tenant_payload = {
        "name": "TestTenant",
        "organization": "TestOrg",
        "properties": {}
    }
    resp = requests.post(f"{BASE_URL}/tenants", json=tenant_payload, headers=headers)
    if resp.status_code == 201:
        tenant = resp.json()
        print(f"Created Tenant: {tenant['id']}")
        tenant_id = tenant['id']
    elif resp.status_code == 403:
        print("Logged in as Tenant Admin (cannot create tenants). Using current tenant.")
        if not tenant_id:
            print("Error: Could not determine tenant_id from login.")
            return
        print(f"Using default Tenant: {tenant_id}")
    else: # If exists, reuse or fail
        print(f"Tenant creation status: {resp.status_code} {resp.text}")
        if tenant_id:
             print(f"Fallback to existing tenant from login: {tenant_id}")
        else:
             print("Failed to ensure tenant.")
             return
    
    # 3. Create Tenant Admin User
    # Root creates a user for this tenant
    user_payload = {
        "username": "tenant_admin",
        "email": "admin@test.com",
        "password": "password",
        "role": "tenant-admin",
        "tenant_id": tenant_id
    }
    # Add tenant header even though creating user usually implies it via payload, just in case
    # headers["X-Pangolin-Tenant"] = tenant_id 
    
    resp = requests.post(f"{BASE_URL}/users", json=user_payload, headers=headers)
    if resp.status_code == 201:
        print("Created Tenant Admin user")
    elif resp.status_code == 409:
        print("Tenant Admin user already exists")
    else:
        print(f"Failed to create tenant admin: {resp.text}")
        
    # 4. Login as Tenant Admin
    resp = requests.post(f"{BASE_URL}/users/login", json={"username": "tenant_admin", "password": "password"})
    if not resp.ok:
        print(f"Login (TenantAdmin) failed: {resp.text}")
        return
    
    admin_token = resp.json()["token"]
    admin_headers = {
        "Authorization": f"Bearer {admin_token}",
        "X-Pangolin-Tenant": tenant_id
    }

    # 5. Create Warehouse (as Tenant Admin)
    warehouse_payload = {
        "name": "test-warehouse",
        "storage_config": {
            "type": "s3",
            "bucket": "test-bucket",
            "region": "us-east-1",
            "access_key": "minioadmin",
            "secret_key": "minioadmin",
            "endpoint": "http://localhost:9000"
        }
    }
    resp = requests.post(f"{BASE_URL}/warehouses", json=warehouse_payload, headers=admin_headers)
    if resp.status_code in [201, 409]:
        print("Warehouse 'test-warehouse' ready")
    else:
        print(f"Failed to create warehouse: {resp.text}")

    # 6. Create Catalog (as Tenant Admin)
    catalog_payload = {
        "name": "test-catalog",
        "warehouse_id": "test-warehouse",
        "type": "iceberg"
    }
    resp = requests.post(f"{BASE_URL}/catalogs", json=catalog_payload, headers=admin_headers)
    if resp.status_code in [201, 409]:
        print("Catalog 'test-catalog' ready")
    else:
        print(f"Failed to create catalog: {resp.text}")

if __name__ == "__main__":
    setup()
