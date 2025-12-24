import requests
import json
import uuid

BASE_URL = "http://localhost:8080/api/v1"

def print_result(step, response):
    if response.status_code >= 200 and response.status_code < 300:
        print(f"[PASS] {step}")
        return response.json() if response.content else {}
    else:
        print(f"[FAIL] {step}: {response.status_code} {response.text}")
        return None

# 1. Login Root
print("--- Logging in as Root ---")
resp = requests.post(f"{BASE_URL}/users/login", json={"username": "admin", "password": "password"})
token = print_result("Login Root", resp).get("token")
headers = {"Authorization": f"Bearer {token}"}

if not token:
    exit(1)

# 2. Create Tenant Admin
print("--- Creating Tenant Admin ---")
# Root creates Tenant Admin in default tenant
user_payload = {
    "username": "verify_admin",
    "email": "verify_admin@test.com",
    "password": "password",
    "tenant_id": "00000000-0000-0000-0000-000000000000",
    "role": "tenant-admin" 
}
print_result("Create Tenant Admin", requests.post(f"{BASE_URL}/users", json=user_payload, headers=headers))

# 3. Login as Tenant Admin
print("--- Logging in as Tenant Admin ---")
admin_resp = requests.post(f"{BASE_URL}/users/login", json={"username": "verify_admin", "password": "password", "tenant-id": "00000000-0000-0000-0000-000000000000"})
admin_token = print_result("Login Tenant Admin", admin_resp).get("token")
admin_headers = {"Authorization": f"Bearer {admin_token}"}

if not admin_token:
    exit(1)

# 4. Create Warehouse (as Tenant Admin)
print("--- Creating Warehouse ---")
wh_payload = {
    "name": "test-verify-wh",
    "warehouse_type": "s3",
    "storage_config": {
      "bucket": "test-bucket",
      "region": "us-east-1",
      "endpoint": "http://localhost:9000"
    }
}
print_result("Create Warehouse", requests.post(f"{BASE_URL}/warehouses", json=wh_payload, headers=admin_headers))

# 5. Create Catalog (as Tenant Admin)
print("--- Creating Catalog ---")
cat_payload = {
    "name": "test-verify-cat",
    "warehouse_name": "test-verify-wh",
    "tenant_id": "00000000-0000-0000-0000-000000000000",
    "type": "managed"
}
print_result("Create Catalog", requests.post(f"{BASE_URL}/catalogs", json=cat_payload, headers=admin_headers))

# 6. Check Audit Logs (Issue 3) -> As Tenant Admin
print("--- Verifying Audit Logs ---")
# Use correct route /api/v1/audit
audit_resp = requests.get(f"{BASE_URL}/audit", headers=admin_headers)
audit_logs = print_result("Get Audit Logs", audit_resp)

if audit_logs:
    # Need to verify if paginated or list. Assuming list or {results: ...}
    # If paginated, might look different.
    # Looking at audit_handlers.rs `list_audit_events`, it usually returns Vec<AuditLogEntry> or pagination wrapper.
    # Let's assume list for now, or check structure if printing works.
    items = audit_logs if isinstance(audit_logs, list) else audit_logs.get("results", [])
    
    found_create_catalog = any(l.get("action") == "create_catalog" and l.get("resource_name") == "test-verify-cat" for l in items)
    # create_user was done by Root. Tenant Admin might NOT see Root's actions on the tenant unless scope allows.
    # But `create_catalog` was done by Tenant Admin.
    
    if found_create_catalog:
        print("[PASS] Audit Log contains create_catalog")
    else:
        print("[FAIL] Audit Log MISSING create_catalog")
        print("Logs found:", json.dumps(items, indent=2))
else:
    print("[FAIL] Failed to retrieve audit logs")


# 7. Create Asset (Table)
# Create Namespace first
print("--- Creating Namespace ---")
ns_payload = {"namespace": ["default"]}
print_result("Create Namespace", requests.post("http://localhost:8080/v1/test-verify-cat/namespaces", json=ns_payload, headers=admin_headers))

# Create Table
print("--- Creating Table ---")
tbl_payload = {
    "name": "verify_tbl",
    "schema": {
        "type": "struct",
        "fields": [
            {"id": 1, "name": "id", "type": "int", "required": True},
            {"id": 2, "name": "data", "type": "string", "required": False}
        ]
    }
}
# Iceberg route for create table
tbl_resp = requests.post("http://localhost:8080/v1/test-verify-cat/namespaces/default/tables", json=tbl_payload, headers=admin_headers)
tbl_data = print_result("Create Table", tbl_resp)
asset_id = tbl_data.get("metadata", {}).get("uuid") if tbl_data else None
# Wait, Iceberg create table response usually returns metadata logic.
# I need to check response structure. Usually standard Iceberg LoadTableResponse.
# If `tbl_data` doesn't have ID, I might need to fetch it via `load_table` or `get_asset`.
if not asset_id:
    # Try fetching asset details from business metadata API to get ID
    # Use search to find ID?
    pass

# 8. Make Discoverable (Issue 2)
# Need asset ID.
# Let's search for it as Admin to get ID.
print("--- Searching as Admin to get ID ---")
s_resp = requests.get(f"{BASE_URL}/search/assets?q=verify_tbl", headers=admin_headers)
s_res = print_result("Search Admin", s_resp)
if s_res and s_res.get("results"):
    asset = s_res["results"][0]
    asset_id = asset["id"]
    print(f"Asset ID: {asset_id}")

    print(f"--- Making Asset {asset_id} Discoverable ---")
    meta_payload = {
        "description": "Discoverable Asset",
        "tags": ["public"],
        "discoverable": True,
        "properties": {}
    }
    # Business Metadata endpoint: POST /api/v1/assets/:id/metadata
    meta_resp = requests.post(f"{BASE_URL}/assets/{asset_id}/metadata", json=meta_payload, headers=admin_headers)
    print_result("Update Metadata", meta_resp)


# 9. Create Tenant User (Fresh User)
print("--- Creating Fresh Tenant User ---")
user_payload = {
    "username": "verify_user",
    "email": "verify@test.com",
    "password": "password",
    "tenant_id": "00000000-0000-0000-0000-000000000000",
    "role": "tenant-user"
}
print_result("Create Tenant User", requests.post(f"{BASE_URL}/users", json=user_payload, headers=admin_headers))

# 10. Login as Fresh User
print("--- Logging in as Fresh User ---")
u_resp = requests.post(f"{BASE_URL}/users/login", json={"username": "verify_user", "password": "password", "tenant-id": "00000000-0000-0000-0000-000000000000"})
u_token = print_result("Login User", u_resp).get("token")
u_headers = {"Authorization": f"Bearer {u_token}"}

# 11. Search (Issue 1 & 2)
print("--- Searching as Fresh User ---")
search_resp = requests.get(f"{BASE_URL}/search/assets?q=verify_tbl", headers=u_headers)
search_results = print_result("Search", search_resp)

if search_results and search_results.get("results"):
    result = search_results["results"][0]
    print(f"Found Asset: {result.get('name')}")
    print(f"Asset Type: {result.get('asset_type')} (Should be 'table')")
    print(f"Has Access: {result.get('has_access')} (Should be False)")
    
    if result.get("asset_type") == "table":
        print("[PASS] Asset Type is 'table'")
    else:
        print(f"[FAIL] Asset Type is '{result.get('asset_type')}' (Expected 'table')")
        
    if result.get("has_access") is False:
        print("[PASS] Has Access is False")
    else:
        print(f"[FAIL] Has Access is {result.get('has_access')} (Expected False)")
else:
    print("[FAIL] No results found")
