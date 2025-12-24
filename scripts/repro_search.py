
import requests
import json
import sys

BASE_URL = "http://localhost:8080"

def login(username, password):
    resp = requests.post(f"{BASE_URL}/api/v1/users/login", json={"username": username, "password": password})
    if resp.status_code != 200:
        print(f"Login failed for {username}: {resp.text}")
        sys.exit(1)
    return resp.json().get("token")

def create_user(admin_token, username, password, role):
    headers = {"Authorization": f"Bearer {admin_token}"}
    payload = {"username": username, "password": password, "role": role}
    # Create user checks if exists? 
    # Just try to create, if fails maybe already exists, that's fine.
    resp = requests.post(f"{BASE_URL}/api/v1/users", json=payload, headers=headers)
    print(f"Create user {username}: {resp.status_code}")
    return resp.status_code

def search(token, query):
    headers = {"Authorization": f"Bearer {token}"}
    resp = requests.get(f"{BASE_URL}/api/v1/search", params={"q": query}, headers=headers)
    if resp.status_code != 200:
        print(f"Search failed: {resp.status_code} {resp.text}")
        return None
    return resp.json()

import uuid

def search_unified(token, query):
    headers = {"Authorization": f"Bearer {token}"}
    resp = requests.get(f"{BASE_URL}/api/v1/search", params={"q": query}, headers=headers)
    if resp.status_code != 200:
        print(f"Unified Search failed: {resp.status_code} {resp.text}")
        return None
    return resp.json()

def search_assets(token, query):
    headers = {"Authorization": f"Bearer {token}"}
    resp = requests.get(f"{BASE_URL}/api/v1/search/assets", params={"q": query}, headers=headers)
    if resp.status_code != 200:
        print(f"Asset Search failed: {resp.status_code} {resp.text}")
        return None
    return resp.json()

def create_warehouse(token, name):
    headers = {"Authorization": f"Bearer {token}"}
    payload = {
        "name": name, 
        "storage_profile": {"type": "S3", "bucket": "test-bucket", "region": "us-east-1"},
        "storage_credential_config": {"mode": "Manual"}
    }
    resp = requests.post(f"{BASE_URL}/api/v1/warehouses", json=payload, headers=headers)
    if resp.status_code not in [201, 200, 409]:
        print(f"Create warehouse failed: {resp.status_code} {resp.text}")

def create_catalog(token, name):
    headers = {"Authorization": f"Bearer {token}"}
    payload = {"name": name, "warehouse_name": "s3-warehouse", "catalog_type": "Local"} 
    # Removed explicit storage_profile to rely on warehouse?
    # Or keep it consistent with validation logic.
    resp = requests.post(f"{BASE_URL}/api/v1/catalogs", json=payload, headers=headers)
    if resp.status_code not in [201, 200, 409]:
        print(f"Create catalog failed: {resp.status_code} {resp.text}")

def create_asset(token, catalog, namespace, name):
    headers = {"Authorization": f"Bearer {token}"}
    # Create namespace
    requests.post(f"{BASE_URL}/v1/{catalog}/namespaces", json={"namespace": [namespace]}, headers=headers)
    # Register asset
    payload = {
        "name": name,
        "kind": "DELTA_TABLE",
        "location": "s3://bucket/path",
        "properties": {}
    }
    resp = requests.post(f"{BASE_URL}/api/v1/catalogs/{catalog}/namespaces/{namespace}/assets", json=payload, headers=headers)
    if resp.status_code not in [201, 200, 409]:
        print(f"Create asset failed: {resp.status_code} {resp.text}")

def main():
    username = "user"
    password = "Password123"
    
    print("1. Login as admin")
    admin_token = login("admin", "password")
    
    print("2. Seeding Data (Warehouse, Catalog, Asset)...")
    create_warehouse(admin_token, "s3-warehouse")
    create_catalog(admin_token, "test_catalog")
    create_asset(admin_token, "test_catalog", "default", "secret_table")
    
    print(f"3. Create and Login as {username}")
    create_user(admin_token, username, password, "TenantUser")
    user_token = login(username, password)
    
    print("\n4. Testing Unified Search (/api/v1/search) for 'secret'")
    results_unified = search_unified(user_token, "secret")
    if results_unified and len(results_unified.get("results", [])) > 0:
        print("❌ FAILURE: Unified search leaked 'secret_table'!")
        for r in results_unified["results"]:
            print(f" - Found: {r.get('name')}")
    else:
        print("✅ SUCCESS: Unified search found 0 results.")

    print("\n5. Testing Asset Search (/api/v1/search/assets) for 'secret'")
    results_assets = search_assets(user_token, "secret")
    if results_assets and len(results_assets.get("results", [])) > 0:
        print("❌ FAILURE: Asset search leaked 'secret_table'!")
        for r in results_assets["results"]:
             print(f" - Found: {r.get('name')}")
    else:
        print("✅ SUCCESS: Asset search found 0 results.")

if __name__ == "__main__":
    main()
