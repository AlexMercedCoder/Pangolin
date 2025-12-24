import requests
import json
import time

BASE_URL = "http://localhost:8080"
ADMIN_USER = "admin"
ADMIN_PASS = "password"

def get_token():
    print(f"Logging in as {ADMIN_USER}...")
    resp = requests.post(f"{BASE_URL}/api/v1/users/login", json={
        "username": ADMIN_USER,
        "password": ADMIN_PASS
    })
    resp.raise_for_status()
    print("Login successful.")
    data = resp.json()
    print(f"Login Response Keys: {data.keys()}")
    return data["token"] # Guessing it might be 'token' based on past experience, or will fail again and show keys

def create_warehouse(token):
    print("Creating S3 Warehouse (MinIO)...")
    headers = {"Authorization": f"Bearer {token}"}
    data = {
        "name": "minio-warehouse",
        "storage_type": "s3",
        "storage_config": {
            "s3.bucket": "warehouse",
            "s3.endpoint": "http://localhost:9000",
            "s3.region": "us-east-1",
            "s3.access-key-id": "minioadmin",
            "s3.secret-access-key": "minioadmin"
        }
    }
    resp = requests.post(f"{BASE_URL}/api/v1/warehouses", json=data, headers=headers)
    if resp.status_code == 409:
        print("Warehouse already exists.")
    else:
        resp.raise_for_status()
        print("Warehouse created.")

def create_catalog(token):
    print("Creating Catalog 'dev_catalog'...")
    headers = {"Authorization": f"Bearer {token}"}
    data = {
        "name": "dev_catalog",
        "catalog_type": "Local",
        "warehouse_name": "minio-warehouse"
    }
    resp = requests.post(f"{BASE_URL}/api/v1/catalogs", json=data, headers=headers)
    if resp.status_code == 409:
        print("Catalog already exists.")
    else:
        resp.raise_for_status()
        print("Catalog created.")

def create_namespace(token):
    print("Creating Namespace 'dev'...")
    headers = {"Authorization": f"Bearer {token}"}
    data = {"namespace": ["dev"]}
    resp = requests.post(f"{BASE_URL}/v1/dev_catalog/namespaces", json=data, headers=headers)
    if resp.status_code == 409:
        print("Namespace already exists.")
    else:
        resp.raise_for_status()
        print("Namespace created.")

def main():
    try:
        token = get_token()
        create_warehouse(token)
        create_catalog(token)
        create_namespace(token)
        print("\n✅ Reseed complete! 'dev_catalog' and 'dev' namespace ready.")
    except Exception as e:
        print(f"\n❌ Error during reseed: {e}")

if __name__ == "__main__":
    main()
