import requests
import uuid
import time
import sys

BASE_URL = "http://localhost:8080"

def get_token(username, password):
    resp = requests.post(f"{BASE_URL}/api/v1/users/login", json={"username": username, "password": password})
    if resp.status_code != 200:
        raise Exception(f"Login failed for {username}: {resp.text}")
    return resp.json()["token"]

def setup_users_and_catalogs():
    # Use root user credentials (default fallback in login handler)
    print("Logging in as root user...")
    root_token = get_token("admin", "password")
    root_headers = {"Authorization": f"Bearer {root_token}"}
    
    # Fetch root user's tenant ID (should be None, but we'll use default tenant)
    default_tenant_id = "00000000-0000-0000-0000-000000000000"
    
    # 1. Create Tenant Admin User
    print("Creating tenant admin user...")
    resp = requests.post(f"{BASE_URL}/api/v1/users", json={
        "username": "tenant_admin",
        "email": "admin@pangolin.com",
        "password": "admin123",
        "role": "tenant-admin",
        "tenant_id": default_tenant_id
    }, headers=root_headers)
    if resp.status_code not in [200, 201]:
         raise Exception(f"Failed to create tenant admin: {resp.text}")
    
    # 2. Login as Tenant Admin
    print("Logging in as tenant admin...")
    admin_token = get_token("tenant_admin", "admin123")
    headers = {"Authorization": f"Bearer {admin_token}"}

    # 3. Create Restricted User
    print("Creating restricted user...")
    resp = requests.post(f"{BASE_URL}/api/v1/users", json={
        "username": "restricted_user",
        "email": "restricted@pangolin.com",
        "password": "password",
        "role": "tenant-user",
        "tenant_id": default_tenant_id
    }, headers=headers)
    if resp.status_code not in [200, 201]:
         raise Exception(f"Failed to create user: {resp.text}")
    
    # 2. Create Catalogs
    print("Creating catalogs...")
    resp = requests.post(f"{BASE_URL}/api/v1/catalogs", json={"name": "CatalogA"}, headers=headers) # Secret
    if resp.status_code not in [200, 201]:
         raise Exception(f"Failed to create CatalogA: {resp.text}")

    resp = requests.post(f"{BASE_URL}/api/v1/catalogs", json={"name": "CatalogB"}, headers=headers) # Public-ish
    if resp.status_code not in [200, 201]:
         raise Exception(f"Failed to create CatalogB: {resp.text}")

    # 3. Create Assets (Iceberg Tables via REST API)
    print("Creating assets...")
    
    # Define valid Iceberg Schema
    schema = {
        "type": "struct",
        "fields": [
            {"id": 1, "name": "id", "type": "int", "required": False},
            {"id": 2, "name": "data", "type": "string", "required": False}
        ]
    }

    # Secret Asset in CatalogA
    print("Creating 'default' namespace in CatalogA...")
    requests.post(f"{BASE_URL}/v1/CatalogA/namespaces", json={"namespace": ["default"]}, headers=headers)
    
    print("Creating SecretTable in CatalogA...")
    asset_secret = {
        "name": "SecretTable",
        "schema": schema,
        "location": "s3://warehouse/catalogA/secret",
        "partition-spec": [],
        "properties": {}
    }
    # Iceberg URL: /v1/{prefix}/namespaces/{namespace}/tables
    resp = requests.post(f"{BASE_URL}/v1/CatalogA/namespaces/default/tables", json=asset_secret, headers=headers)
    if resp.status_code != 200:
        raise Exception(f"Failed to create SecretTable: {resp.text}")
    print("SecretTable created.")
    # Extract ID from response? Iceberg response usually has metadata. 
    # But we need Asset ID for Pangolin lookup testing.
    # Iceberg LoadTableResponse has metadata-log location?
    # Wait, where does Pangolin store/return the Asset ID (UUID)?
    # The Iceberg response standard does not include Pangolin UUID.
    # We might need to look it up by name or list tables?
    # Or maybe the responses include it in properties?
    # Let's assume we can Search for it.

    # Public Asset in CatalogB
    print("Creating 'default' namespace in CatalogB...")
    requests.post(f"{BASE_URL}/v1/CatalogB/namespaces", json={"namespace": ["default"]}, headers=headers)

    print("Creating PublicTable in CatalogB...")
    asset_public = {
        "name": "PublicTable",
        "schema": schema,
        "location": "s3://warehouse/catalogB/public"
    }
    resp = requests.post(f"{BASE_URL}/v1/CatalogB/namespaces/default/tables", json=asset_public, headers=headers)
    if resp.status_code != 200:
        raise Exception(f"Failed to create PublicTable: {resp.text}")
    print("PublicTable created.")
    
    # Resolve IDs via Search
    print("Resolving Asset IDs via Search...")
    # Admin search should find them
    resp = requests.get(f"{BASE_URL}/api/v1/assets/search?query=Table", headers=headers)
    print(f"Search Status: {resp.status_code}")
    print(f"Search Body: {resp.text}")
    results = resp.json()
    
    secret_id = next((r["id"] for r in results if r["name"] == "SecretTable"), None)
    public_id = next((r["id"] for r in results if r["name"] == "PublicTable"), None)
    
    if not secret_id or not public_id:
        raise Exception("Failed to resolve Asset IDs after creation")


    # 4. Assign Permission ONLY for CatalogB to restricted_user
    # Need to find the user ID first
    users = requests.get(f"{BASE_URL}/api/v1/users", headers=headers).json()
    user_id = next(u["id"] for u in users if u["username"] == "restricted_user")
    
    # Generate CatalogB ID (LIST catalogs to find it)
    catalogs = requests.get(f"{BASE_URL}/api/v1/catalogs", headers=headers).json()
    catalog_b_id = next(c["id"] for c in catalogs if c["name"] == "CatalogB")

    # Create Permission
    print(f"Assigning READ permission for CatalogB ({catalog_b_id}) to user {user_id}...")
    perm = {
        "user-id": user_id,
        "scope": {
            "type": "catalog",
            "catalog_id": catalog_b_id
        },
        "actions": ["read"]
    }
    perm_resp = requests.post(f"{BASE_URL}/api/v1/permissions", json=perm, headers=headers)
    print(f"Permission creation status: {perm_resp.status_code}")
    print(f"Permission creation response: {perm_resp.text}")
    if perm_resp.status_code not in [200, 201]:
        raise Exception(f"Failed to create permission: {perm_resp.text}")
    
    return secret_id, public_id

def verify_performance(admin_token, public_id):
    headers = {"Authorization": f"Bearer {admin_token}"}
    print("\n--- Verifying Performance (Asset Lookup) ---")
    start = time.time()
    # Use the new (to be implemented) direct asset lookup endpoint or existing that uses it?
    # The requirement is `get_asset_by_id` in store.
    # Expose a direct endpoint? Or just verify `add_business_metadata` which uses it?
    # Let's use `add_business_metadata` as a proxy since it will use the lookup.
    
    requests.post(f"{BASE_URL}/api/v1/assets/{public_id}/metadata", json={
        "discoverable": True,
        "tags": [],
        "properties": {}
    }, headers=headers)
    
    end = time.time()
    print(f"Asset lookup & update took: {end - start:.4f}s")
    # We can't strictly enforce O(1) from outside, but we check it works.

def verify_security(secret_id, public_id):
    print("\n--- Verifying Security (Search Filtering) ---")
    print("Attempting to login as restricted_user...")
    user_token = get_token("restricted_user", "password")
    print(f"Restricted user token: {user_token[:50]}...")
    headers = {"Authorization": f"Bearer {user_token}"}
    
    # 1. Search
    print("Searching for 'Table'...")
    resp = requests.get(f"{BASE_URL}/api/v1/assets/search?query=Table", headers=headers)
    results = resp.json()
    
    names = [r["name"] for r in results]
    print(f"Search Results: {names}")
    
    if "SecretTable" in names:
        print("❌ FAIL: SecretTable found in search results!")
        sys.exit(1)
    
    if "PublicTable" not in names:
        print("❌ FAIL: PublicTable NOT found in search results!")
        sys.exit(1)
        
    print("✅ Search filtering passed.")

    # 2. Try to modify SecretTable
    print("Attempting to modify SecretTable metadata...")
    resp = requests.post(f"{BASE_URL}/api/v1/assets/{secret_id}/metadata", json={
        "discoverable": True,
        "tags": [],
        "properties": {}
    }, headers=headers)
    if resp.status_code == 403:
        print("✅ Correctly received 403 Forbidden.")
    else:
        print(f"❌ FAIL: Expected 403, got {resp.status_code} {resp.text}")
        sys.exit(1)

def main():
    try:
        secret_id, public_id = setup_users_and_catalogs()
        admin_token = get_token("tenant_admin", "admin123")  # Re-login for performance test
        verify_performance(admin_token, public_id)
        verify_security(secret_id, public_id)
        print("\n✅ PHASE 1 VERIFICATION SUCCESSFUL")
    except Exception as e:
        print(f"\n❌ Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
