import requests

def update_warehouse():
    # Login
    auth_response = requests.post("http://localhost:8080/api/v1/users/login", json={"username": "testuser2", "password": "Password123"})
    auth_response.raise_for_status()
    token = auth_response.json()["token"]
    headers = {"Authorization": f"Bearer {token}"}
    
    # Update Warehouse
    wh_payload = {
        "name": "minio-warehouse",
        "use_sts": False,
        "storage_config": {
            "type": "s3",
            "bucket": "warehouse",
            "region": "us-east-1",
            "endpoint": "http://192.168.16.2:9000",
            "access_key_id": "minioadmin",
            "secret_access_key": "minioadmin"
        },
        "vending_strategy": None
    }
    
    print("Updating Warehouse...")
    res = requests.put("http://localhost:8080/api/v1/warehouses/minio-warehouse", json=wh_payload, headers=headers)
    if res.status_code == 200:
        print("Warehouse Updated.")
    else:
        print(f"Warehouse update failed: {res.status_code} {res.text}")

if __name__ == "__main__":
    update_warehouse()
