import requests

BASE_URL = "http://localhost:8080/api/v1"

def create_limited_user():
    # 1. Login as Tenant Admin
    resp = requests.post(f"{BASE_URL}/users/login", json={"username": "tenant_admin", "password": "password"})
    if not resp.ok:
        print(f"Login failed: {resp.text}")
        return
    token = resp.json()["token"]
    tenant_id = "b601090d-7bb5-40d0-b9db-82677c60daa1" # From previous step
    headers = {"Authorization": f"Bearer {token}", "X-Pangolin-Tenant": tenant_id}

    # 2. Create User
    user_payload = {
        "username": "limited_user",
        "email": "limited@test.com",
        "password": "password",
        "role": "tenant-user",
        "tenant_id": tenant_id
    }
    
    resp = requests.post(f"{BASE_URL}/users", json=user_payload, headers=headers)
    if resp.status_code == 201:
        print("Created limited_user")
        print("User ID:", resp.json()["id"])
    elif resp.status_code == 409:
        print("limited_user already exists")
    else:
        print(f"Failed to create user: {resp.text}")

if __name__ == "__main__":
    create_limited_user()
