import requests
import uuid
import time
import base64

# Configuration
BASE_URL = "http://localhost:8080"
ROOT_USER = "admin"
ROOT_PASSWORD = "admin123"

def get_basic_auth_header(username, password):
    credentials = f"{username}:{password}"
    encoded_credentials = base64.b64encode(credentials.encode()).decode()
    return {"Authorization": f"Basic {encoded_credentials}"}

headers_root = get_basic_auth_header(ROOT_USER, ROOT_PASSWORD)

def print_pass(message):
    print(f"\033[92m[PASS] {message}\033[0m")

def print_fail(message, response=None):
    print(f"\033[91m[FAIL] {message}\033[0m")
    if response:
        print(f"Response: {response.text}")

def check_response(response, expected_code, message):
    if response.status_code == expected_code:
        print_pass(message)
    else:
        print_fail(f"{message} - Expected {expected_code}, got {response.status_code}", response)

print("Starting Tag-Based Permission Test...")

# 1. Setup Tenant and Users
# Create Tenant
tenant_response = requests.post(f"{BASE_URL}/api/v1/tenants", json={"name": "tag_test_tenant"}, headers=headers_root)
if tenant_response.status_code == 201:
    print_pass("Create Tenant")
    tenant_id = tenant_response.json()["id"]
    print(f"Tenant ID: {tenant_id}")
else:
    # Try to clean up? Or just fail.
    # Assuming fresh backend restart before this (optional but good practice)
    check_response(tenant_response, 201, "Create Tenant")
    exit(1)

# Create Admin User
admin_user = {
    "username": "admin_tag",
    "email": "admin_tag@test.com",
    "password": "password123",
    "password": "password123",
    "tenant-id": tenant_id,
    "role": "tenant-admin"
}
r = requests.post(f"{BASE_URL}/api/v1/users", json=admin_user, headers=headers_root)
check_response(r, 201, "Create Admin User")

# Create Data Engineer (Will have Tag Permission)
engineer_user = {
    "username": "engineer",
    "email": "engineer@test.com",
    "password": "password123",
    "password": "password123",
    "tenant-id": tenant_id,
    "role": "tenant-user"
}
r = requests.post(f"{BASE_URL}/api/v1/users", json=engineer_user, headers=headers_root)
check_response(r, 201, "Create Engineer User")
engineer_id = r.json()["id"]

# Create Analyst (Will NOT have Tag Permission)
analyst_user = {
    "username": "analyst",
    "email": "analyst@test.com",
    "password": "password123",
    "password": "password123",
    "tenant-id": tenant_id,
    "role": "tenant-user"
}
r = requests.post(f"{BASE_URL}/api/v1/users", json=analyst_user, headers=headers_root)
check_response(r, 201, "Create Analyst User")
analyst_id = r.json()["id"]

# Login as Admin
auth_admin = requests.post(f"{BASE_URL}/api/v1/users/login", json={"username": "admin_tag", "password": "password123"})
check_response(auth_admin, 200, "Login Admin")
token_admin = auth_admin.json()["token"]
headers_admin = {"Authorization": f"Bearer {token_admin}"}

# 2. Create Catalog
catalog_payload = {
    "name": "sensitive_data",
    "type": "rest",
    "warehouse_id": str(uuid.uuid4()) # Placeholder
}
r = requests.post(f"{BASE_URL}/api/v1/catalogs?tenant_id={tenant_id}", json=catalog_payload, headers=headers_admin)
check_response(r, 201, "Create Catalog 'sensitive_data'")
catalog_id = r.json()["id"]

# 3. Create Namespace
ns_payload = {"namespace": ["finance"]}
# Iceberg REST API uses /v1/{prefix}/namespaces
# Prefix = catalog name = sensitive_data
r = requests.post(f"{BASE_URL}/v1/sensitive_data/namespaces", json=ns_payload, headers=headers_admin)
check_response(r, 200, "Create Namespace 'finance'")

# 4. Create Table 'salaries'
table_payload = {
    "name": "salaries",
    "schema": {
        "type": "struct",
        "fields": [{"id": 1, "name": "amount", "type": "int", "required": True}],
        "schema-id": 0,
        "identifier-field-ids": []
    },
    "location": "s3://warehouse/salaries",
    "partition-spec": [],
    "write-order": []
}
r = requests.post(f"{BASE_URL}/v1/sensitive_data/namespaces/finance/tables", json=table_payload, headers=headers_admin)
check_response(r, 200, "Create Table 'salaries'") 

# 5. Get Asset ID for tagging
# Extract from Create Table response
# TableResponse -> metadata -> table-uuid
create_table_data = r.json()
asset_id = create_table_data["metadata"]["table-uuid"]
print_pass(f"Found Asset ID (Table UUID): {asset_id}")

# 6. Apply 'PII' tag to Asset (Business Metadata)
metadata_payload = {
    "tags": ["PII"],
    "properties": {},
    "discoverable": True
}
# Using correct endpoint for creating/updating metadata
# business_metadata_handlers.rs: update_business_metadata -> POST /api/v1/assets/{asset_id}/metadata
# (Wait, route is POST /api/v1/assets/:id/metadata in lib.rs)
r = requests.post(f"{BASE_URL}/api/v1/assets/{asset_id}/metadata", json=metadata_payload, headers=headers_admin)
if r.status_code == 404:
    # try POST if PUT is not upsert?
    # Actually, let's check if we have a create route?
    # handler is `update_business_metadata`.
    print_fail("Metadata endpoint not found or asset not found", r)
    exit(1)
check_response(r, 200, "Tag Asset with 'PII'")

# 7. Create Role 'PIIAccess'
role_pii = {
    "name": "PIIAccess",
    "description": "Access to PII tagged data",
    "tenant-id": tenant_id
}
r = requests.post(f"{BASE_URL}/api/v1/roles", json=role_pii, headers=headers_admin)
check_response(r, 201, "Create Role 'PIIAccess'")
role_pii_data = r.json()
role_pii_id = role_pii_data["id"]

# 8. Grant Permission to Role (Tag Based)
# Action: Read (or Select?)
# Scope: Tag("PII")
perm_tag = {
    "id": str(uuid.uuid4()),
    "scope": { 
        "type": "tag",
        "tag_name": "PII"
    },
    "actions": ["all"], # Grant ALL on PII data
    "effect": "allow",
    "granted-by": str(uuid.UUID(int=0)),
    "granted-at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
}
role_pii_data["permissions"].append(perm_tag)
r = requests.put(f"{BASE_URL}/api/v1/roles/{role_pii_id}", json=role_pii_data, headers=headers_admin)
check_response(r, 200, "Grant 'PII' Tag Permission to Role")

# 9. Assign Role to Engineer
# Use role-id (kebab-case) as learned from debugging!
r = requests.post(f"{BASE_URL}/api/v1/users/{engineer_id}/roles", json={"role-id": role_pii_id}, headers=headers_admin)
check_response(r, 201, "Assign PII Role to Engineer")

# 10. Verify Access

# Engineer (With PII Role) should be able to Read Table
auth_eng = requests.post(f"{BASE_URL}/api/v1/users/login", json={"username": "engineer", "password": "password123"})
check_response(auth_eng, 200, "Login Engineer")
token_eng = auth_eng.json()["token"]
headers_eng = {"Authorization": f"Bearer {token_eng}"}

# We use the Iceberg Load Table endpoint to test read access
# GET /v1/{prefix}/namespaces/{namespace}/tables/{table}
# Prefix is sensitive_data
r = requests.get(f"{BASE_URL}/v1/sensitive_data/namespaces/finance/tables/salaries", headers=headers_eng)
check_response(r, 200, "Engineer Read PII Table (Allowed)")

# Analyst (Without Role) should be Denied
auth_analyst = requests.post(f"{BASE_URL}/api/v1/users/login", json={"username": "analyst", "password": "password123"})
check_response(auth_analyst, 200, "Login Analyst")
token_analyst = auth_analyst.json()["token"]
headers_analyst = {"Authorization": f"Bearer {token_analyst}"}

r = requests.get(f"{BASE_URL}/v1/sensitive_data/namespaces/finance/tables/salaries", headers=headers_analyst)
# Should be 403 Forbidden
check_response(r, 403, "Analyst Read PII Table (Denied)")

print("\n--- Tag Verification Completed ---")
