import requests
import json
import uuid
import time
import base64
import os
import sys

# Configuration
BASE_URL = "http://localhost:8080"
E2E_PASSWORD = "password123"
ROOT_USER = "admin"
ROOT_PASSWORD = "admin123"

def get_basic_auth_header(username, password):
    credentials = f"{username}:{password}"
    encoded = base64.b64encode(credentials.encode()).decode()
    return {"Authorization": f"Basic {encoded}"}

# Colors for output
class Colors:
    HEADER = '\033[95m'
    OKBLUE = '\033[94m'
    OKGREEN = '\033[92m'
    WARNING = '\033[93m'
    FAIL = '\033[91m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'
    UNDERLINE = '\033[4m'

def print_pass(msg):
    print(f"{Colors.OKGREEN}[PASS]{Colors.ENDC} {msg}")

def print_fail(msg):
    print(f"{Colors.FAIL}[FAIL]{Colors.ENDC} {msg}")

def check_response(resp, expected_code, msg):
    if resp.status_code == expected_code:
        print_pass(msg)
        return True
    else:
        print_fail(f"{msg} - Expected {expected_code}, got {resp.status_code}")
        print(f"Response: {resp.text}")
        return False

def main():
    print(f"{Colors.HEADER}Starting Branching Permissions Test...{Colors.ENDC}\n")

    # 1. Create Tenant
    print("--- Bootstrapping ---")
    headers_root = get_basic_auth_header(ROOT_USER, ROOT_PASSWORD)
    tenant_resp = requests.post(f"{BASE_URL}/api/v1/tenants", json={"name": "branching_test_tenant"}, headers=headers_root)
    if not check_response(tenant_resp, 201, "Create Tenant"):
        sys.exit(1)
    tenant_id = tenant_resp.json()["id"]
    print(f"Tenant ID: {tenant_id}")

    # 2. Create Users
    print("\n--- Creating Users ---")
    # Admin
    admin_user = {
        "username": "admin",
        "email": "admin@example.com",
        "password": E2E_PASSWORD,
        "tenant-id": tenant_id,
        "role": "tenant-admin"
    }
    requests.post(f"{BASE_URL}/api/v1/users", json=admin_user, headers=headers_root)
    print_pass("Create Admin User")

    # Developer (Experimental Branching only)
    dev_user = {
        "username": "dev",
        "email": "dev@example.com",
        "password": E2E_PASSWORD,
        "tenant-id": tenant_id,
        "role": "tenant-user"
    }
    dev_resp = requests.post(f"{BASE_URL}/api/v1/users", json=dev_user, headers=headers_root)
    print_pass("Create Dev User")
    dev_id = dev_resp.json()["id"]
    print(f"Dev ID: {dev_id}")

    # Ingester (Ingest Branching only)
    ingest_user = {
        "username": "ingester",
        "email": "ingester@example.com",
        "password": E2E_PASSWORD,
        "tenant-id": tenant_id,
        "role": "tenant-user"
    }
    ingest_resp = requests.post(f"{BASE_URL}/api/v1/users", json=ingest_user, headers=headers_root)
    print_pass("Create Ingest User")
    ingest_id = ingest_resp.json()["id"]

    # 3. Login Admin
    print("\n--- Admin Configuration ---")
    auth_resp = requests.post(f"{BASE_URL}/api/v1/users/login", json={"username": "admin", "password": E2E_PASSWORD})
    check_response(auth_resp, 200, "Login Admin")
    token_admin = auth_resp.json()["token"]
    headers_admin = {"Authorization": f"Bearer {token_admin}"}

    # 4. Create Catalog
    cat_resp = requests.post(f"{BASE_URL}/api/v1/catalogs", json={"name": "analytics"}, headers=headers_admin)
    check_response(cat_resp, 201, "Create Catalog 'analytics'")
    catalog_id = cat_resp.json()["id"]

    # 5. Create Roles
    # Role: ExperimentalBrancher
    role_exp = {
        "name": "ExperimentalBrancher",
        "tenant-id": tenant_id,
        "permissions": []
    }
    r1 = requests.post(f"{BASE_URL}/api/v1/roles", json=role_exp, headers=headers_admin)
    check_response(r1, 201, "Create Role ExperimentalBrancher")
    role_exp_data = r1.json()
    role_exp_id = role_exp_data["id"]
    print_pass("Create Role ExperimentalBrancher")

    # Role: IngestBrancher
    role_ing = {
        "name": "IngestBrancher",
        "tenant-id": tenant_id,
        "permissions": []
    }
    r2 = requests.post(f"{BASE_URL}/api/v1/roles", json=role_ing, headers=headers_admin)
    check_response(r2, 201, "Create Role IngestBrancher")
    role_ing_data = r2.json()
    role_ing_id = role_ing_data["id"]
    print_pass("Create Role IngestBrancher")

    # 6. Grant Permissions
    import datetime
    now_str = datetime.datetime.utcnow().isoformat() + "Z"
    
    # Exp -> experimental-branching
    perm_exp = {
        "id": str(uuid.uuid4()),
        "user-id": str(uuid.UUID(int=0)), # Nil UUID for Role permission
        "scope": {"type": "catalog", "catalog_id": catalog_id},
        "actions": ["experimental-branching"],
        "granted-by": str(uuid.UUID(int=0)), # Placeholder
        "granted-at": now_str
    }
    role_exp_data["permissions"].append(perm_exp)
    r_update = requests.put(f"{BASE_URL}/api/v1/roles/{role_exp_id}", json=role_exp_data, headers=headers_admin)
    check_response(r_update, 200, "Grant Experimental Permission to Role")

    # Ing -> ingest-branching
    perm_ing = {
        "id": str(uuid.uuid4()),
        "user-id": str(uuid.UUID(int=0)),
        "scope": {"type": "catalog", "catalog_id": catalog_id},
        "actions": ["ingest-branching"],
        "granted-by": str(uuid.UUID(int=0)),
        "granted-at": now_str
    }
    role_ing_data["permissions"].append(perm_ing)
    r_update2 = requests.put(f"{BASE_URL}/api/v1/roles/{role_ing_id}", json=role_ing_data, headers=headers_admin)
    check_response(r_update2, 200, "Grant Ingest Permission to Role")

    # 7. Assign Roles
    r_assign1 = requests.post(f"{BASE_URL}/api/v1/users/{dev_id}/roles", json={"role-id": role_exp_id}, headers=headers_admin)
    check_response(r_assign1, 201, "Assign Experimental Role to Dev")

    r_assign2 = requests.post(f"{BASE_URL}/api/v1/users/{ingest_id}/roles", json={"role-id": role_ing_id}, headers=headers_admin)
    check_response(r_assign2, 201, "Assign Ingest Role to Ingester")

    # 8. Testing
    print("\n--- Verifying Permissions ---")

    # Login Dev
    auth_dev = requests.post(f"{BASE_URL}/api/v1/users/login", json={"username": "dev", "password": E2E_PASSWORD})
    token_dev = auth_dev.json()["token"]
    headers_dev = {"Authorization": f"Bearer {token_dev}"}

    # Login Ingester
    auth_ing = requests.post(f"{BASE_URL}/api/v1/users/login", json={"username": "ingester", "password": E2E_PASSWORD})
    token_ing = auth_ing.json()["token"]
    headers_ing = {"Authorization": f"Bearer {token_ing}"}

    # Test 1: Dev creates Experimental Branch (Should Pass)
    req1 = {
        "name": "dev-experiment",
        "branch_type": "experimental",
        "catalog": "analytics"
    }
    resp1 = requests.post(f"{BASE_URL}/api/v1/branches", json=req1, headers=headers_dev)
    check_response(resp1, 201, "Dev Create Experimental Branch")

    # Test 2: Dev creates Ingest Branch (Should Fail)
    req2 = {
        "name": "dev-ingest",
        "branch_type": "ingest",
        "catalog": "analytics"
    }
    resp2 = requests.post(f"{BASE_URL}/api/v1/branches", json=req2, headers=headers_dev)
    check_response(resp2, 403, "Dev Create Ingest Branch (Expect Fail)")

    # Test 3: Ingester creates Ingest Branch (Should Pass)
    req3 = {
        "name": "etl-ingest",
        "branch_type": "ingest",
        "catalog": "analytics"
    }
    resp3 = requests.post(f"{BASE_URL}/api/v1/branches", json=req3, headers=headers_ing)
    check_response(resp3, 201, "Ingester Create Ingest Branch")

    # Test 4: Ingester creates Experimental Branch (Should Fail)
    req4 = {
        "name": "etl-experiment",
        "branch_type": "experimental",
        "catalog": "analytics"
    }
    resp4 = requests.post(f"{BASE_URL}/api/v1/branches", json=req4, headers=headers_ing)
    check_response(resp4, 403, "Ingester Create Experimental Branch (Expect Fail)")

    print("\n--- Branching Verification Completed ---")

if __name__ == "__main__":
    main()
