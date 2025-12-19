import requests
import json
import time

BASE_URL = "http://localhost:8080"
AUTH_URL = f"{BASE_URL}/api/v1/users/login"

def run():
    print("--- Seeding Valid Iceberg Tables ---")
    
    # 1. Login
    session = requests.Session()
    try:
        resp = session.post(AUTH_URL, json={"username": "tenant_admin", "password": "password123"})
        if resp.status_code != 200:
            print(f"Login failed: {resp.status_code} {resp.text}")
            return
        
        data = resp.json()
        token = data['token']
        tenant_id = data['user']['tenant-id']
        headers = {
            "Authorization": f"Bearer {token}",
            "X-Pangolin-Tenant": tenant_id
        }
        print(f"Logged in. Tenant ID: {tenant_id}")
    except Exception as e:
        print(f"Login exception: {e}")
        return

    catalog_name = "test-catalog"
    namespace = "default"
    warehouse_name = "test-warehouse"

    # 1.5 Ensure Warehouse Exists
    print("Ensuring Warehouse exists...")
    wh_payload = {
        "name": warehouse_name,
        "storage_config": {
            "type": "s3",
            "bucket": "test-bucket",
            "region": "us-east-1",
            "access_key": "minioadmin",
            "secret_key": "minioadmin",
            "endpoint": "http://localhost:9000"
        }
    }
    resp = session.post(f"{BASE_URL}/api/v1/warehouses", headers=headers, json=wh_payload)
    if resp.status_code in [200, 201, 409]:
        print("Warehouse ready.")
    else:
        print(f"Warehouse creation failed: {resp.text}")

    # 1.6 Ensure Catalog Exists
    print("Ensuring Catalog exists...")
    cat_payload = {
        "name": catalog_name,
        "warehouse_id": warehouse_name,
        "type": "iceberg"
    }
    resp = session.post(f"{BASE_URL}/api/v1/catalogs", headers=headers, json=cat_payload)
    if resp.status_code in [200, 201, 409]:
        print("Catalog ready.")
    else:
        print(f"Catalog creation failed: {resp.text}")

    # 1.7 Ensure Namespace Exists
    print("Ensuring Namespace 'default' exists...")
    ns_payload = {"namespace": ["default"]}
    resp = session.post(f"{BASE_URL}/v1/{catalog_name}/namespaces", headers=headers, json=ns_payload)
    if resp.status_code in [200, 201, 409]:
        print("Namespace ready.")
    else:
        print(f"Namespace creation failed: {resp.text}")

    # Ensure catalog/namespace exist (reusing logic implicitly or trusting previous steps)
    # 2. Create 'customers' table
    print("Creating 'customers' table...")
    payload_customers = {
        "name": "customers",
        "schema": {
            "type": "struct",
            "fields": [
                {"id": 1, "name": "customer_id", "required": True, "type": "int"},
                {"id": 2, "name": "first_name", "required": False, "type": "string"},
                {"id": 3, "name": "last_name", "required": False, "type": "string"},
                {"id": 4, "name": "email", "required": False, "type": "string"}
            ]
        },
        "location": f"s3://warehouse/{catalog_name}/default/customers",
        "properties": {
            "owner": "sales"
        }
    }
    
    # Use Standard Iceberg Create Table Endpoint
    url = f"{BASE_URL}/v1/{catalog_name}/namespaces/{namespace}/tables"
    resp = session.post(url, headers=headers, json=payload_customers)
    
    if resp.status_code == 200 or resp.status_code == 201:
        print("Success: customers table created.")
    elif resp.status_code == 409:
        print("Info: customers table already exists.")
    else:
        print(f"Failed to create customers: {resp.status_code} {resp.text}")

    # 3. Create 'orders' table
    print("Creating 'orders' table...")
    payload_orders = {
        "name": "orders",
        "schema": {
            "type": "struct",
            "fields": [
                {"id": 1, "name": "order_id", "required": True, "type": "int"},
                {"id": 2, "name": "customer_id", "required": True, "type": "int"},
                {"id": 3, "name": "total", "required": False, "type": "double"},
                {"id": 4, "name": "order_date", "required": False, "type": "date"}
            ]
        },
        "location": f"s3://warehouse/{catalog_name}/default/orders",
        "properties": {
            "owner": "sales",
            "write.format.default": "parquet"
        }
    }
    resp = session.post(url, headers=headers, json=payload_orders)

    if resp.status_code == 200 or resp.status_code == 201:
        print("Success: orders table created.")
    elif resp.status_code == 409:
        print("Info: orders table already exists.")
    else:
        print(f"Failed to create orders: {resp.status_code} {resp.text}")

if __name__ == "__main__":
    run()
