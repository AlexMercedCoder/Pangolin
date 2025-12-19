import requests

def list_catalogs():
    # Login
    auth_response = requests.post("http://localhost:8080/api/v1/users/login", json={"username": "testuser2", "password": "Password123"})
    auth_response.raise_for_status()
    token = auth_response.json()["token"]
    
    # List Warehouses
    resp = requests.get("http://localhost:8080/api/v1/warehouses", headers={"Authorization": f"Bearer {token}"})
    print(resp.status_code)
    try:
        print(resp.json())
    except:
        print(resp.text)

if __name__ == "__main__":
    list_catalogs()
