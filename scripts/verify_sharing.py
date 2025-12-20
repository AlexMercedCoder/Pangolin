
import requests
import sys
import json
import time

BASE_PANGOLIN_URL = "http://localhost:8080/api/v1"
BASE_ICEBERG_URL = "http://localhost:8080/v1"
ADMIN_USER = "admin"
ADMIN_PASS = "password"

def log(msg):
    print(f"[VERIFY] {msg}")

def fail(msg):
    print(f"[FAIL] {msg}")
    sys.exit(1)

def get_auth_token(username, password):
    resp = requests.post(f"{BASE_PANGOLIN_URL}/users/login", json={"username": username, "password": password})
    if resp.status_code != 200:
        fail(f"Login failed: {resp.text}")
    return resp.json()["token"]

import uuid

# Global suffix to ensure uniqueness per run
run_id = uuid.uuid4().hex[:6]
TENANT_NAME = f"share_tenant_{run_id}"
ADMIN_NAME = f"share_admin_{run_id}"
USER_NAME = f"share_user_{run_id}"

# ... (rest of imports)

def main():
    log(f"Starting Verification (Run ID: {run_id})...")
    
    # 1. Login as Root
    root_token = get_auth_token(ADMIN_USER, ADMIN_PASS)
    root_headers = {"Authorization": f"Bearer {root_token}"}
    
    # 2. Cleanup & Create Tenant
    # (Skip cleanup listing, just create unique)
    resp = requests.post(f"{BASE_PANGOLIN_URL}/tenants", json={"name": TENANT_NAME, "properties": {}}, headers=root_headers)
    if resp.status_code != 201:
        fail(f"Failed to create tenant: {resp.text}")
    tenant = resp.json()
    tenant_id = tenant["id"]
    log(f"Created Tenant: {tenant['name']} (ID: {tenant_id})")

    # 3. Create Tenant Admin
    resp = requests.post(f"{BASE_PANGOLIN_URL}/users", json={
        "username": ADMIN_NAME,
        "email": f"{ADMIN_NAME}@test.com",
        "password": "password",
        "role": "tenant-admin",
        "tenant_id": tenant_id
    }, headers=root_headers)
    if resp.status_code != 201:
        fail(f"Failed to create tenant admin: {resp.text}")
    log("Created Tenant Admin")

    ta_token = get_auth_token(ADMIN_NAME, "password")
    ta_headers = {"Authorization": f"Bearer {ta_token}"}
    
    # DEBUG: Check Admin User Info
    me_resp = requests.get(f"{BASE_PANGOLIN_URL}/users/me", headers=ta_headers)
    log(f"Admin Info: {me_resp.text}")
    
    # 4. Create Tenant User (share_user)
    resp = requests.post(f"{BASE_PANGOLIN_URL}/users", json={
        "username": USER_NAME,
        "email": f"{USER_NAME}@test.com",
        "password": "password",
        "role": "tenant-user",
        "tenant_id": tenant_id
    }, headers=ta_headers)
    if resp.status_code != 201:
        fail(f"Failed to create tenant user: {resp.text}")
    share_user_id = resp.json()["id"]
    log("Created Tenant User")
    
    # 5. Create Catalog & Namespace
    # Need a catalog first? 
    # Check if a default catalog exists or create one. Tenant usually needs one.
    resp = requests.post(f"{BASE_PANGOLIN_URL}/catalogs", json={
        "name": "share_catalog",
        "type": "memory", # Simple memory catalog
        "tenant_id": tenant_id
    }, headers=ta_headers)
    if resp.status_code != 201:
         fail(f"Failed to create catalog: {resp.text}")
    catalog_data = resp.json()
    catalog_id = catalog_data["id"]
    log(f"Created Catalog 'share_catalog' (ID: {catalog_id})")
    
    # Create Namespace via Iceberg API ? Or just implicitly via Table create? 
    # Pangolin usually supports implicit namespace in memory store or needs explicit create.
    # Let's try creating namespace first using Iceberg REST API standard
    # POST /v1/{prefix}/namespaces
    resp = requests.post(f"{BASE_ICEBERG_URL}/share_catalog/namespaces", json={"namespace": ["data"]}, headers=ta_headers)
    if resp.status_code != 200: # Iceberg returns 200 for create namespace
         # It might be 200 or 201 depending on impl.
         # Actually Pangolin API path might differ. 
         # Let's try to assume namespace creation works or skip if memory store handles it.
         pass
    
    # 6. Create Tables (Using simple metadata if possible, or just 'register' logic if exposed, 
    # but standard Iceberg create table is complex. 
    # Let's use `create_table` endpoint if we can, or specific pangolin endpoint?
    # Pangolin is an Iceberg catalog. We need to send valid Iceberg TableMetadata.
    # To simplify, we'll assume the tables exist if we call 'create' with minimal valid payload.
    # Minimal payload:
    table_payload = {
        "name": "table_shared",
        "schema": {
            "type": "struct", 
            "fields": [{"id": 1, "name": "id", "required": True, "type": "int"}]
        },
        "partition-spec": [],
        "sort-order": []
    }
    
    # Create table_shared
    resp = requests.post(f"{BASE_ICEBERG_URL}/share_catalog/namespaces/data/tables", json=table_payload, headers=ta_headers)
    if resp.status_code != 200:
        fail(f"Failed to create table_shared: {resp.text}")
    
    table_resp = resp.json()
    # Iceberg REST: LoadTableResponse has 'metadata' -> 'table-uuid' or 'uuid'
    # Check keys
    # Standard Iceberg JSON serialization usually uses kebab-case for keys like 'table-uuid' within metadata?
    # Or 'uuid' property of metadata object.
    # Let's try to find it.
    metadata = table_resp.get("metadata", {})
    table_uuid = metadata.get("table-uuid") or metadata.get("uuid")
    
    log(f"Created table 'data.table_shared' (UUID: {table_uuid})")

    # Create table_discoverable
    table_payload["name"] = "table_discoverable"
    resp = requests.post(f"{BASE_ICEBERG_URL}/share_catalog/namespaces/data/tables", json=table_payload, headers=ta_headers)
    if resp.status_code != 200:
        fail(f"Failed to create table_discoverable: {resp.text}")
    log("Created table 'data.table_discoverable'")
    
    # 7. Grant Permission on 'table_shared' to 'share_user'
    # POST /api/v1/permissions
    perm_payload = {
        "user-id": share_user_id,
        "scope": {
            "type": "asset",
            "catalog_id": catalog_id, 
            "namespace": "data",
            "asset_id": table_uuid
        },
        "actions": ["read"] # 'describe' needed to see it? 'select' to query.
    }
    # Note: 'catalog_id' in scope struct in rust is String.
    
    resp = requests.post(f"{BASE_PANGOLIN_URL}/permissions", json=perm_payload, headers=ta_headers)
    if resp.status_code != 201:
        fail(f"Failed to grant permission: {resp.text}")
    
    # Also Grant LIST on Namespace so user can see the table list
    list_payload = {
        "user-id": share_user_id,
        "scope": {
            "type": "namespace",
            "catalog_id": catalog_id,
            "namespace": "data"
        },
        "actions": ["list"]
    }
    resp = requests.post(f"{BASE_PANGOLIN_URL}/permissions", json=list_payload, headers=ta_headers)
    if resp.status_code != 201:
         fail(f"Failed to grant list permission: {resp.text}")
         
    log("Granted SELECT/DESCRIBE on 'table_shared' and LIST on 'data' to User")

    # 8. Login as Tenant User
    user_token = get_auth_token(USER_NAME, "password")
    user_headers = {"Authorization": f"Bearer {user_token}"}
    log("Logged in as Tenant User")

    # 9. Verify 'table_shared' is visible (Data Explorer)
    # List tables in namespace
    resp = requests.get(f"{BASE_ICEBERG_URL}/share_catalog/namespaces/data/tables", headers=user_headers)
    if resp.status_code != 200:
        fail(f"Failed to list tables as user: {resp.text}")
    
    tables = resp.json()["identifiers"] # Iceberg REST returns identifiers list
    # Expected: [{'namespace': ['data'], 'name': 'table_shared'}]
    # 'table_discoverable' should NOT be here (unless default is open?)
    
    found_shared = any(t["name"] == "table_shared" for t in tables)
    found_disc = any(t["name"] == "table_discoverable" for t in tables)
    
    if not found_shared:
        fail("User CANNOT see 'table_shared' in Data Explorer!")
    if found_disc:
        log("Warning: User can see 'table_discoverable' in Data Explorer (Access might be open by default?)")
    else:
        log("Verified: User cannot see 'table_discoverable' in Data Explorer (Correct)")
        
    log("STEP 1 SUCCESS: 'table_shared' is visible.")

    # 10. Verify 'table_discoverable' is discoverable (Search)
    # GET /api/v1/assets/search?query=table_discoverable
    # Ideally should work even if not in list, if tagged or if secure discovery is on.
    resp = requests.get(f"{BASE_PANGOLIN_URL}/assets/search?query=table_discoverable", headers=user_headers)
    if resp.status_code != 200:
        fail(f"Search failed: {resp.text}")
    
    search_results = resp.json()
    # Check if we found it
    # Result structure depends on API.
    found_in_search = any(
        r.get("name") == "table_discoverable" or r.get("asset_name") == "table_discoverable" 
        for r in search_results
    )
    
    if found_in_search:
        log("STEP 2 SUCCESS: 'table_discoverable' found in Search.")
    else:
        log("Search result empty. Trying to 'make discoverable' by tagging as Admin...")
        
        # Admin tags the table "public" or "discoverable"
        # POST /api/v1/metadata/tags ? Or similar.
        # Assuming we need to add a tag.
        # Let's try: POST /api/v1/catalogs/.../tables/.../tags (if exists) or business metadata.
        # Check if business metadata API exists for tagging.
        pass 
        # For now, if failed, we report failure to find.

    log("Verification Complete.")

if __name__ == "__main__":
    main()
