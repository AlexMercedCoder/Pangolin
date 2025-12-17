import requests
import sys

API_URL = "http://localhost:8080/api/v1"

def authenticate(username, password):
    try:
        resp = requests.post(f"{API_URL}/users/login", json={
            "username": username,
            "password": password
        })
        resp.raise_for_status()
        return resp.json()["token"]
    except Exception as e:
        print(f"Auth failed: {e}")
        sys.exit(1)

def bootstrap():
    # 1. Login as System Admin
    print("Logging in as System Admin...")
    sys_token = authenticate("admin", "password")
    sys_headers = {"Authorization": f"Bearer {sys_token}"}

    # 2. Create Tenant
    print("Creating Tenant...")
    resp = requests.post(f"{API_URL}/tenants", headers=sys_headers, json={"name": "demo_tenant", "properties": {}})
    if resp.status_code == 201:
        tenant_id = resp.json()["id"]
    elif resp.status_code == 409:
        # Fetch
        t_resp = requests.get(f"{API_URL}/tenants", headers=sys_headers)
        tenants = t_resp.json()
        tenant_id = next(t for t in tenants if t['name'] == 'demo_tenant')['id']
    else:
        print(f"Tenant failed: {resp.text}")
        sys.exit(1)

    print(f"Tenant ID: {tenant_id}")

    # 3. Create Tenant Admin
    print("Creating Tenant Admin...")
    resp = requests.post(f"{API_URL}/users", headers=sys_headers, json={
        "username": "tenant_admin",
        "email": "admin@demo.com",
        "password": "password",
        "role": "tenant-admin",
        "tenant_id": tenant_id
    })
    
    # 4. Login as Tenant Admin
    print("Logging in as Tenant Admin...")
    admin_token = authenticate("tenant_admin", "password")
    admin_headers = {"Authorization": f"Bearer {admin_token}"}

    # 5. Create Warehouse
    print("Creating Warehouse...")
    requests.post(f"{API_URL}/warehouses", headers=admin_headers, json={
        "name": "warehouse",
        "storage_config": {
            "type": "S3",
            "bucket": "warehouse",
            "region": "us-east-1",
            "access_key_id": "minioadmin",
            "secret_access_key": "minioadmin",
            "endpoint": "http://minio:9000"
        }
    })

    # 6. Create Catalog
    print("Creating Catalog 'my_catalog'...")
    requests.post(f"{API_URL}/catalogs", headers=admin_headers, json={
        "name": "my_catalog",
        "warehouse_name": "warehouse",
        "storage_location": "s3://warehouse/my_catalog",
        "catalog_type": "Local",
        "properties": {}
    })

    print("Multi-Tenant Setup Complete.")

if __name__ == "__main__":
    bootstrap()
