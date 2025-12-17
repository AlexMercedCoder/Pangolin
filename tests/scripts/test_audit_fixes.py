import requests
import json
import time

BASE_URL = "http://localhost:8080"
TENANT_HEADER = "x-pangolin-tenant"
DEFAULT_TENANT_ID = "00000000-0000-0000-0000-000000000000" # Placeholder, will fetch real one

def get_tenant_id():
    # List tenants and pick first or create default
    try:
        resp = requests.get(f"{BASE_URL}/api/v1/tenants")
        if resp.status_code == 200:
            tenants = resp.json()
            if tenants:
                return tenants[0]['id']
    except Exception as e:
        print(f"Error fetching tenants: {e}")
    return DEFAULT_TENANT_ID

TENANT_ID = get_tenant_id()
HEADERS = {
    TENANT_HEADER: TENANT_ID,
    "Content-Type": "application/json"
}

def create_catalog(name):
    print(f"Creating catalog '{name}'...")
    resp = requests.post(f"{BASE_URL}/api/v1/catalogs", headers=HEADERS, json={
        "name": name,
        "storage_location": f"s3://warehouse/{name}"
    })
    if resp.status_code in [201, 200]:
        print("Success")
        return True
    else:
        print(f"Failed: {resp.text}")
        return False

def create_branch(catalog, branch_name):
    print(f"Creating branch '{branch_name}' in catalog '{catalog}'...")
    resp = requests.post(f"{BASE_URL}/api/v1/branches", headers=HEADERS, json={
        "name": branch_name,
        "catalog": catalog,
        "from_branch": "main" # Assuming main exists or empty initialization
    })
    
    # If 404 (e.g. main doesn't exist yet on new catalog), we might need to handle empty init. 
    # Current API might fail if source branch missing.
    # But for a new catalog, 'main' might not exist. 
    # Let's try to list branches first to see what exists.
    if resp.status_code == 404: 
        # Checking if it's because source branch not found.
        # Pangolin API currently might expect a source branch.
        print("404 received. This might be because 'main' doesn't exist in new catalog.")
    
    if resp.status_code in [201, 200]:
        print("Success")
        return True
    else:
        print(f"Failed: {resp.text}")
        return False

def test_list_commits(catalog, branch):
    print(f"Testing list_commits for branch '{branch}' in catalog '{catalog}'...")
    # 1. Correct catalog
    resp = requests.get(f"{BASE_URL}/api/v1/branches/{branch}/commits?catalog={catalog}", headers=HEADERS)
    if resp.status_code == 200:
        print("[PASS] List Commits with correct catalog returned 200")
    else:
        print(f"[FAIL] List Commits with correct catalog failed: {resp.status_code} {resp.text}")

    # 2. Incorrect catalog (default)
    # The branch shouldn't exist in default, so it should 404
    resp = requests.get(f"{BASE_URL}/api/v1/branches/{branch}/commits", headers=HEADERS)
    if resp.status_code == 404:
        print("[PASS] List Commits without catalog param returned 404 (Expected)")
    else:
        print(f"[FAIL] List Commits without catalog param returned {resp.status_code} (Expected 404)")

def test_tags(catalog, branch):
    # Need a commit ID first. Since we just created branch, maybe it has no commits?
    # If new catalog/branch is empty, we can't create a tag easily without a commit.
    # Pangolin stores HEAD commit. 
    
    # Get branch details to find HEAD commit
    resp = requests.get(f"{BASE_URL}/api/v1/branches/{branch}?catalog={catalog}", headers=HEADERS)
    if resp.status_code != 200:
        print(f"Failed to get branch: {resp.text}")
        return

    branch_data = resp.json()
    head_commit = branch_data.get('head_commit_id')
    
    if not head_commit:
        import uuid
        head_commit = str(uuid.uuid4())
        print(f"Branch has no HEAD commit. Using random UUID {head_commit} for Tag test.")

    TAG_NAME = "audit_tag_v1"
    
    # 1. Create Tag
    print(f"Creating tag '{TAG_NAME}' in catalog '{catalog}'...")
    resp = requests.post(f"{BASE_URL}/api/v1/tags", headers=HEADERS, json={
        "name": TAG_NAME,
        "catalog": catalog,
        "commit_id": head_commit
    })
    if resp.status_code == 200:
        print("[PASS] Create Tag success")
    else:
        print(f"[FAIL] Create Tag failed: {resp.text}")
        return

    # 2. List Tags (Positive)
    resp = requests.get(f"{BASE_URL}/api/v1/tags?catalog={catalog}", headers=HEADERS)
    tags = resp.json()
    if any(t['name'] == TAG_NAME for t in tags):
        print("[PASS] Tag found in correct catalog list")
    else:
        print("[FAIL] Tag NOT found in correct catalog list")

    # 3. List Tags (Negative)
    resp = requests.get(f"{BASE_URL}/api/v1/tags", headers=HEADERS) # defaults to default
    tags = resp.json()
    if not any(t['name'] == TAG_NAME for t in tags):
        print("[PASS] Tag NOT found in default catalog list (Expected)")
    else:
        print("[FAIL] Tag FOUND in default catalog list (Unexpected leak)")

    # 4. Delete Tag
    print(f"Deleting tag '{TAG_NAME}'...")
    resp = requests.delete(f"{BASE_URL}/api/v1/tags/{TAG_NAME}?catalog={catalog}", headers=HEADERS)
    if resp.status_code in [200, 204]:
        print("[PASS] Delete Tag success")
    else:
        print(f"[FAIL] Delete Tag failed: {resp.text}")

def main():
    CATALOG = "audit_catalog"
    BRANCH = "audit_branch"
    
    if not create_catalog(CATALOG):
        return
        
    # Create a table to ensure we have a commit (and 'main' branch exists implicitly or explicitly)
    # Actually, let's create a Namespace and Table via Iceberg API to perform a commit on 'main'.
    # This ensures 'main' has a HEAD.
    # Iceberg URL: /v1/{prefix}/namespaces...
    
    print("Creating table to generate commits...")
    ns_resp = requests.post(f"{BASE_URL}/v1/{CATALOG}/namespaces", headers=HEADERS, json={"namespace": ["audit_ns"]})
    
    tbl_resp = requests.post(f"{BASE_URL}/v1/{CATALOG}/namespaces/audit_ns/tables", headers=HEADERS, json={
        "name": "audit_table",
        "schema": {
            "type": "struct",
            "fields": [{"id": 1, "name": "id", "type": "int", "required": True}]
        }
    })
    
    if tbl_resp.status_code == 200:
        print("Table created (Commit generated).")
    else:
        print(f"Failed to create table: {tbl_resp.text}")

    # Now create branch from main
    if not create_branch(CATALOG, BRANCH):
        # If create branch fails, maybe main exists now?
        pass

    test_list_commits(CATALOG, BRANCH)
    test_tags(CATALOG, BRANCH)

if __name__ == "__main__":
    main()
