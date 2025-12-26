import requests
import os
import sys

# Configuration
API_URL = "http://localhost:8080"
ADMIN_USER = "tenant_admin"
ADMIN_PASS = "password123"

def main():
    print(f"Testing OAuth flow against {API_URL}...")

    # 1. Login as Admin to create Service User
    print("\n1. Logging in as Admin...")
    resp = requests.post(f"{API_URL}/api/v1/users/login", json={
        "username": ADMIN_USER,
        "password": ADMIN_PASS
    })
    
    if resp.status_code != 200:
        print(f"Login failed: {resp.text}")
        return
        
    admin_token = resp.json()["token"]
    tenant_id = resp.json()["user"]["tenant_id"]
    print("   Login successful.")

    # 2. Create Service User
    print("\n2. Creating Service User...")
    service_user_name = "oauth_test_user"
    resp = requests.post(
        f"{API_URL}/api/v1/service-users",
        headers={"Authorization": f"Bearer {admin_token}", "X-Pangolin-Tenant": tenant_id},
        json={
            "name": service_user_name,
            "role": "TenantUser",
            "active": True
        }
    )
    
    if resp.status_code != 200:
        # Check if already exists?
        print(f"   Note: Creation might have failed if exists, checking lists... {resp.status_code}")
    else:
        print("   Service User created.")
        client_id = resp.json()["id"]
        client_secret = resp.json()["api_key"]
        print(f"   Client ID: {client_id}")
        print(f"   Client Secret: {client_secret}")

        # 3. Exchange Credentials for Token using OAuth endpoint
        print("\n3. Testing OAuth Token Exchange...")
        # Note: PyIceberg uses application/x-www-form-urlencoded
        oauth_resp = requests.post(
            f"{API_URL}/v1/rest/v1/oauth/tokens", # "rest" is the prefix we typically use for Iceberg
            data={
                "grant_type": "client_credentials",
                "client_id": client_id,
                "client_secret": client_secret,
                "scope": "catalog"
            }
        )

        if oauth_resp.status_code == 200:
            token = oauth_resp.json()["access_token"]
            print(f"✅ OAuth Success! Token obtained: {token[:15]}...")
        else:
            print(f"❌ OAuth Failed: {oauth_resp.status_code} - {oauth_resp.text}")

if __name__ == "__main__":
    main()
