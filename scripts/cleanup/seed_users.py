
import requests
import json

BASE_URL = "http://localhost:8080/api/v1"

# 1. Login as Tenant Admin (to create user)
print("--- Logging in as Tenant Admin ---")
resp = requests.post(f"{BASE_URL}/users/login", json={
    "username": "tenant_admin",
    "password": "password123",
    "tenant-id": "00000000-0000-0000-0000-000000000000"
})

if resp.status_code != 200:
    print(f"Admin Login Failed: {resp.text}")
    exit(1)

admin_token = resp.json()["token"]
admin_headers = {"Authorization": f"Bearer {admin_token}"}
print("Admin Logged In")

# 2. Create Basic User
print("\n--- Creating User 'data_analyst' ---")
user_payload = {
    "username": "data_analyst",
    "password": "password123",
    "role": "TenantUser", # Correct case for enum often matters, usually TenantUser or tenant-user
    "tenant_id": "00000000-0000-0000-0000-000000000000"
}

# Try creating
r = requests.post(f"{BASE_URL}/users", json=user_payload, headers=admin_headers)
if r.status_code == 201:
    print("User 'data_analyst' created.")
elif r.status_code == 409:
    print("User 'data_analyst' already exists.")
else:
    print(f"Failed to create user: {r.status_code} {r.text}")
    
# 3. Verify Login as Basic User
print("\n--- Verifying Login as 'data_analyst' ---")
# Bypass password check should work here too if enabled for all
resp = requests.post(f"{BASE_URL}/users/login", json={
    "username": "data_analyst",
    "password": "password123",
    "tenant-id": "00000000-0000-0000-0000-000000000000"
})

if resp.status_code == 200:
    print("User 'data_analyst' logged in successfully.")
    print("Token received. You can now use this user in the UI to test permissions.")
else:
    print(f"User Login Failed: {resp.status_code} {resp.text}")
