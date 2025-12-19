import requests

def setup_resources():
    # Login
    auth_response = requests.post("http://localhost:8080/api/v1/users/login", json={"username": "testuser2", "password": "Password123"})
    auth_response.raise_for_status()
    token = auth_response.json()["token"]
    headers = {"Authorization": f"Bearer {token}"}
    
    # Create Warehouse
    wh_payload = {
        "name": "minio-warehouse",
        "use_sts": False,
        "storage_config": {
            "type": "s3",
            "bucket": "warehouse",
            "region": "us-east-1",
            "endpoint": "http://localhost:9000",
            "access_key_id": "minioadmin",
            "secret_access_key": "minioadmin"
        },
        "vending_strategy": None
    }
    
    print("Creating Warehouse...")
    res = requests.post("http://localhost:8080/api/v1/warehouses", json=wh_payload, headers=headers)
    if res.status_code == 201:
        print("Warehouse Created.")
    else:
        print(f"Warehouse creation failed: {res.status_code} {res.text}")
        if "already exists" in res.text:
             print("Assuming exists.")

    # Create Catalog
    cat_payload = {
        "name": "test-catalog",
        "warehouse_name": "minio-warehouse",
        "type": "pangolin"
    }

    print("Creating Catalog...")
    res = requests.post("http://localhost:8080/api/v1/catalogs", json=cat_payload, headers=headers)
    if res.status_code == 201:
        print("Catalog Created.")
    else:
        print(f"Catalog creation failed: {res.status_code} {res.text}")

if __name__ == "__main__":
    setup_resources()
