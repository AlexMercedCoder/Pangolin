
import requests
import json
import base64
import os
import sys

# Configuration
BASE_URL = "http://localhost:8080"
ROOT_USER = "admin"
ROOT_PASSWORD = "admin123"

def get_basic_auth_header(username, password):
    credentials = f"{username}:{password}"
    encoded = base64.b64encode(credentials.encode()).decode()
    return {"Authorization": f"Basic {encoded}"}

def check_response(response, expected_status, task_name):
    if response.status_code == expected_status:
        print(f"[PASS] {task_name}")
        return True
    else:
        print(f"[FAIL] {task_name} - Expected {expected_status}, got {response.status_code}")
        print(f"Response: {response.text}")
        return False

def main():
    print("Starting Permissions Verification Test...")

    # 1. Bootstrap Tenant using Root Basic Auth
    print("\n--- Bootstrapping ---")
    headers_root = get_basic_auth_header(ROOT_USER, ROOT_PASSWORD)
    
    tenant_payload = {
        "name": "Acme Corp",
        "properties": {}
    }
    resp = requests.post(f"{BASE_URL}/api/v1/tenants", json=tenant_payload, headers=headers_root)
    if not check_response(resp, 201, "Create Tenant"):
        sys.exit(1)
    tenant_id = resp.json()["id"]
    print(f"Tenant ID: {tenant_id}")

    # 2. Create Users
    print("\n--- Creating Users ---")
    # Admin
    admin_payload = {
        "username": "admin_user",
        "email": "admin@acme.com",
        "password": "password123",
        "tenant-id": tenant_id,
        "role": "tenant-admin"
    }
    resp = requests.post(f"{BASE_URL}/api/v1/users", json=admin_payload, headers=headers_root)
    check_response(resp, 201, "Create Admin User")
    admin_id = resp.json()["id"]

    # Developer (Eng)
    dev_payload = {
        "username": "dev_user",
        "email": "dev@acme.com",
        "password": "password123",
        "tenant-id": tenant_id,
        "role": "tenant-user"
    }
    resp = requests.post(f"{BASE_URL}/api/v1/users", json=dev_payload, headers=headers_root)
    check_response(resp, 201, "Create Developer User")
    dev_id = resp.json()["id"]

    # Analyst
    analyst_payload = {
        "username": "analyst_user",
        "email": "analyst@acme.com",
        "password": "password123",
        "tenant-id": tenant_id,
        "role": "tenant-user"
    }
    resp = requests.post(f"{BASE_URL}/api/v1/users", json=analyst_payload, headers=headers_root)
    check_response(resp, 201, "Create Analyst User")
    analyst_id = resp.json()["id"]

    # 3. Login as Admin to configure system
    print("\n--- Admin Configuration ---")
    login_payload = {"username": "admin_user", "password": "password123"}
    resp = requests.post(f"{BASE_URL}/api/v1/users/login", json=login_payload)
    check_response(resp, 200, "Login Admin")
    admin_token = resp.json()["token"]
    headers_admin = {"Authorization": f"Bearer {admin_token}"}

    # Create Catalog
    catalog_payload = {
        "name": "analytics",
        "warehouse": "default",
        "storage-location": "s3://analytics",
        "properties": {}
    }
    resp = requests.post(f"{BASE_URL}/api/v1/catalogs", json=catalog_payload, headers=headers_admin)
    check_response(resp, 201, "Create Catalog 'analytics'")
    catalog_id = resp.json()["id"]
    print(f"Catalog ID: {catalog_id}")

    # Create Roles
    # Role: DataEngineer (Full Access)
    role_eng_payload = {
        "name": "DataEngineer",
        "description": "Full access to analytics catalog",
        "tenant-id": tenant_id
    }
    resp = requests.post(f"{BASE_URL}/api/v1/roles", json=role_eng_payload, headers=headers_admin)
    check_response(resp, 201, "Create Role DataEngineer")
    role_eng_id = resp.json()["id"]
    role_eng_data = resp.json()

    # Role: DataAnalyst (Read Only)
    role_analyst_payload = {
        "name": "DataAnalyst",
        "description": "Read access to analytics catalog",
        "tenant-id": tenant_id
    }
    resp = requests.post(f"{BASE_URL}/api/v1/roles", json=role_analyst_payload, headers=headers_admin)
    check_response(resp, 201, "Create Role DataAnalyst")
    role_analyst_id = resp.json()["id"]
    role_analyst_data = resp.json()

    # Add Permissions to Roles
    # DataEngineer: All actions on Catalog
    role_eng_data["permissions"].append({
        "scope": {
            "type": "catalog",
            "catalog_id": catalog_id
        },
        "actions": ["all"]
    })
    resp = requests.put(f"{BASE_URL}/api/v1/roles/{role_eng_id}", json=role_eng_data, headers=headers_admin)
    check_response(resp, 200, "Grant Permissions to DataEngineer")

    # DataAnalyst: Read, List on Catalog
    role_analyst_data["permissions"].append({
        "scope": {
            "type": "catalog",
            "catalog_id": catalog_id
        },
        "actions": ["read", "list"]
    })
    resp = requests.put(f"{BASE_URL}/api/v1/roles/{role_analyst_id}", json=role_analyst_data, headers=headers_admin)
    check_response(resp, 200, "Grant Permissions to DataAnalyst")

    # Assign Roles
    resp = requests.post(f"{BASE_URL}/api/v1/users/{dev_id}/roles", json={"role-id": role_eng_id}, headers=headers_admin)
    check_response(resp, 201, "Assign DataEngineer to Dev")

    resp = requests.post(f"{BASE_URL}/api/v1/users/{analyst_id}/roles", json={"role-id": role_analyst_id}, headers=headers_admin)
    check_response(resp, 201, "Assign DataAnalyst to Analyst")


    # 4. Verify Enforcement
    print("\n--- Verifying Permissions ---")

    # Login as Dev
    resp = requests.post(f"{BASE_URL}/api/v1/users/login", json={"username": "dev_user", "password": "password123"})
    dev_token = resp.json()["token"]
    headers_dev = {"Authorization": f"Bearer {dev_token}"}

    # Login as Analyst
    resp = requests.post(f"{BASE_URL}/api/v1/users/login", json={"username": "analyst_user", "password": "password123"})
    analyst_token = resp.json()["token"]
    headers_analyst = {"Authorization": f"Bearer {analyst_token}"}


    # Test 1: List Namespaces (Both Should Pass)
    print("\n[Test] List Namespaces")
    resp = requests.get(f"{BASE_URL}/v1/analytics/namespaces", headers=headers_dev)
    check_response(resp, 200, "Dev List Namespaces")
    
    resp = requests.get(f"{BASE_URL}/v1/analytics/namespaces", headers=headers_analyst)
    check_response(resp, 200, "Analyst List Namespaces")


    # Test 2: Create Namespace (Dev Pass, Analyst Fail)
    print("\n[Test] Create Namespace")
    ns_payload = {"namespace": ["logs"], "properties": {}}
    resp = requests.post(f"{BASE_URL}/v1/analytics/namespaces", json=ns_payload, headers=headers_dev)
    check_response(resp, 200, "Dev Create Namespace 'logs'")

    ns_payload_fail = {"namespace": ["hacks"], "properties": {}}
    resp = requests.post(f"{BASE_URL}/v1/analytics/namespaces", json=ns_payload_fail, headers=headers_analyst)
    check_response(resp, 403, "Analyst Create Namespace (Expect Fail)")


    # Test 3: Create Table (Dev Pass, Analyst Fail)
    print("\n[Test] Create Table")
    tbl_payload = {
        "name": "access",
        "location": "s3://analytics/logs/access",
        "schema": {
            "type": "struct",
            "fields": [{"id": 1, "name": "id", "type": "int", "required": True}]
        }
    }
    resp = requests.post(f"{BASE_URL}/v1/analytics/namespaces/logs/tables", json=tbl_payload, headers=headers_dev)
    check_response(resp, 200, "Dev Create Table 'access'")

    tbl_payload_fail = {
        "name": "secrets",
        "location": "s3://analytics/logs/secrets",
         "schema": {
            "type": "struct",
            "fields": [{"id": 1, "name": "id", "type": "int", "required": True}]
        }
    }
    resp = requests.post(f"{BASE_URL}/v1/analytics/namespaces/logs/tables", json=tbl_payload_fail, headers=headers_analyst)
    check_response(resp, 403, "Analyst Create Table (Expect Fail)")


    # Test 4: List Tables (Both Pass)
    print("\n[Test] List Tables")
    resp = requests.get(f"{BASE_URL}/v1/analytics/namespaces/logs/tables", headers=headers_dev)
    check_response(resp, 200, "Dev List Tables")

    resp = requests.get(f"{BASE_URL}/v1/analytics/namespaces/logs/tables", headers=headers_analyst)
    check_response(resp, 200, "Analyst List Tables")


    # Test 5: Insert Data / Commit (Dev Pass, Analyst Fail)
    # Actually 'update_table'
    print("\n[Test] Commit Transaction")
    commit_payload = {
        "identifier": {"namespace": ["logs"], "name": "access"},
        "requirements": [],
        "updates": []
    }
    resp = requests.post(f"{BASE_URL}/v1/analytics/namespaces/logs/tables/access", json=commit_payload, headers=headers_dev)
    check_response(resp, 200, "Dev Commit Table")
    
    # Needs to match 'access' table which analyst can see, but not update
    resp = requests.post(f"{BASE_URL}/v1/analytics/namespaces/logs/tables/access", json=commit_payload, headers=headers_analyst)
    check_response(resp, 403, "Analyst Commit Table (Expect Fail)")


    # Test 6: Delete Table (Dev Pass, Analyst Fail)
    print("\n[Test] Delete Table")
    # Analyst tries delete
    resp = requests.delete(f"{BASE_URL}/v1/analytics/namespaces/logs/tables/access", headers=headers_analyst)
    check_response(resp, 403, "Analyst Delete Table (Expect Fail)")

    # Dev deletes
    resp = requests.delete(f"{BASE_URL}/v1/analytics/namespaces/logs/tables/access", headers=headers_dev)
    check_response(resp, 204, "Dev Delete Table")

    print("\n--- Permissions Verification Completed ---")

if __name__ == "__main__":
    main()
