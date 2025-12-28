import requests
import uuid
import time
import json

BASE_URL = "http://localhost:8080/api/v1"
HEADERS = {
    "Content-Type": "application/json", 
    "X-Pangolin-Tenant": "00000000-0000-0000-0000-000000000000",
    "X-Pangolin-Username": "tenant_admin"
}

def create_warehouse(name):
    url = f"{BASE_URL}/warehouses"
    payload = {
        "name": name,
        "storage_type": "local",
        "storage_config": {"root": "/tmp/pangolin_data"}
    }
    resp = requests.post(url, json=payload, headers=HEADERS)
    print(f"Create Warehouse {name}: {resp.status_code}")
    return resp.status_code == 201

def create_catalog(name, warehouse="default-warehouse"):
    create_warehouse(warehouse)
    
    url = f"{BASE_URL}/catalogs"
    payload = {
        "name": name,
        "catalog_type": "Local",
        "warehouse_name": warehouse,
        "storage_location": f"s3://warehouse/{name}"
    }
    resp = requests.post(url, json=payload, headers=HEADERS)
    print(f"Create Catalog {name}: {resp.status_code}")
    return resp.status_code == 201

def create_asset_via_iceberg(catalog, table_name):
    # Use generic register_asset endpoint to verify store logic (simplifies test)
    # Route: /api/v1/catalogs/{catalog_name}/namespaces/{namespace}/assets
    
    url = f"{BASE_URL}/catalogs/{catalog}/namespaces/default/assets"
    payload = {
        "name": table_name,
        "kind": "ICEBERG_TABLE",
        "location": f"s3://warehouse/{catalog}/default/{table_name}",
        "properties": {}
    }
    resp = requests.post(url, json=payload, headers=HEADERS)
    print(f"Create Asset {table_name}: {resp.status_code}")
    return resp.status_code == 201

def create_branch(catalog, name, from_branch="main"):
    url = f"{BASE_URL}/branches"
    payload = {
        "name": name,
        "catalog": catalog,
        "from_branch": from_branch
    }
    resp = requests.post(url, json=payload, headers=HEADERS)
    print(f"Create Branch {name}: {resp.status_code}")
    return resp.status_code == 201

def list_branch_assets(catalog, branch):
    # We can list assets via iceberg interface or list_assets (if available).
    # Easier to check branch definition which contains asset list
    url = f"{BASE_URL}/branches/{branch}?catalog={catalog}"
    resp = requests.get(url, headers=HEADERS)
    if resp.status_code == 200:
        assets = resp.json().get("assets", [])
        print(f"Branch {branch} has {len(assets)} assets")
        return assets
    return []

def list_commits(catalog, branch):
    url = f"{BASE_URL}/branches/{branch}/commits?catalog={catalog}"
    resp = requests.get(url, headers=HEADERS)
    if resp.status_code == 200:
        commits = resp.json()
        print(f"Branch {branch} has {len(commits)} commits")
        return commits
    return []

def run_test():
    print("Starting Bulk Ops Verification...")
    catalog = f"bulk_test_{uuid.uuid4().hex[:8]}"
    
    # 1. Create Catalog
    if not create_catalog(catalog):
        print("Failed to create catalog")
        return

    # 2. Create 5 Tables in main
    for i in range(5):
        create_asset_via_iceberg(catalog, f"table_{i}")

    # 3. List Commits (Should have 5 commits from table creations)
    commits = list_commits(catalog, "main")
    if len(commits) < 5:
        print(f"Expected at least 5 commits, got {len(commits)}")
    else:
        print("Commit ancestry check passed")

    # 4. Create Branch (Bulk Copy)
    # This triggers the new copy_assets_bulk optimization
    create_branch(catalog, "dev", "main")
    
    # 5. Verify Assets in Dev
    assets = list_branch_assets(catalog, "dev")
    if len(assets) == 5:
        print("Bulk copy verification passed: 5 assets found in new branch")
    else:
        print(f"Bulk copy failed: Expected 5 assets, found {len(assets)}")

if __name__ == "__main__":
    run_test()
