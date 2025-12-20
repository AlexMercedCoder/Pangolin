import requests
import time

def setup_federated():
    base_url = "http://localhost:8080"
    
    # Login as admin
    print("Logging in as admin...")
    try:
        auth_response = requests.post(f"{base_url}/api/v1/users/login", json={"username": "admin", "password": "password"})
        auth_response.raise_for_status()
        token = auth_response.json()["token"]
        headers = {"Authorization": f"Bearer {token}"}
    except Exception as e:
        print(f"Login failed: {e}")
        return

    # Create Federated Catalog
    cat_payload = {
        "name": "demo_federated",
        "config": {
            "base_url": "http://localhost:8181", # Dummy URL
            "auth_type": "None",
            "timeout_seconds": 30
        }
    }

    print("Creating Federated Catalog 'demo_federated'...")
    res = requests.post(f"{base_url}/api/v1/federated-catalogs", json=cat_payload, headers=headers)
    
    if res.status_code == 201:
        print("Federated Catalog Created.")
    elif res.status_code == 409:
        print("Federated Catalog already exists.")
    else:
        print(f"Creation failed: {res.status_code} {res.text}")

if __name__ == "__main__":
    # Wait for server to be likely up
    # time.sleep(2) 
    setup_federated()
