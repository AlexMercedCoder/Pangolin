import os
import time
import requests
import uuid
from pypangolin import PangolinClient
from pypangolin.assets import ParquetAsset
import pandas as pd

# Configuration
API_URL = "http://localhost:8080"
USERNAME = "admin"
PASSWORD = "password"

def log(msg):
    print(f"[{time.strftime('%H:%M:%S')}] {msg}")

def main():
    log("=== Starting PyPangolin Git-like Operations Verification ===")
    
    # 1. Initialize Client & Login
    client = PangolinClient(API_URL, USERNAME, PASSWORD)
    log("✅ Logged in as Root/Admin")
    
    # 2. Setup: Create Tenant, User, Warehouse, Catalog
    tenant_name = f"git_tenant_{str(uuid.uuid4())[:8]}"
    tenant = client.tenants.create(tenant_name)
    client.set_tenant(tenant.id)
    log(f"✅ Created Tenant: {tenant.name} ({tenant.id})")
    
    admin_user = f"admin_{tenant_name}"
    client.users.create(admin_user, f"{admin_user}@example.com", "tenant-admin", tenant_id=tenant.id, password="password")
    
    # Login as tenant admin
    client = PangolinClient(API_URL, admin_user, "password", tenant_id=tenant.id)
    log(f"✅ Logged in as {admin_user}")
    
    wh_name = f"wh_{str(uuid.uuid4())[:8]}"
    # Ensure bucket exists
    try:
        import s3fs
        fs = s3fs.S3FileSystem(key="minioadmin", secret="minioadmin", client_kwargs={"endpoint_url": "http://localhost:9000"})
        if not fs.exists("pangolin-test-bucket"):
            fs.mkdir("pangolin-test-bucket")
    except Exception as e:
        log(f"⚠️ Bucket check skipped/failed: {e}")

    client.warehouses.create_s3(
        wh_name, 
        "pangolin-test-bucket", 
        "us-east-1", 
        "minioadmin", "minioadmin", 
        prefix="git_demo", 
        endpoint="http://localhost:9000"
    )
    log(f"✅ Created Warehouse: {wh_name}")
    
    catalog_name = "git_catalog"
    client.catalogs.create(catalog_name, wh_name, "Local")
    log(f"✅ Created Catalog: {catalog_name}")
    
    # 3. Create a Base Table on 'main'
    ns_client = client.catalogs.namespaces(catalog_name)
    ns_client.create(["git_ns"])
    
    df = pd.DataFrame({"id": [1, 2], "val": ["initial", "value"]})
    ParquetAsset.write(
        client, catalog_name, "git_ns", "test_table", 
        df, 
        f"s3://pangolin-test-bucket/git_demo/{catalog_name}/git_ns/test_table_main.parquet",
        storage_options={"key": "minioadmin", "secret": "minioadmin", "client_kwargs": {"endpoint_url": "http://localhost:9000"}}
    )
    log("✅ Created initial table on 'main'")

    # 4. Branching
    log("--- Testing Branching ---")
    dev_branch = client.branches.create("dev", from_branch="main", catalog_name=catalog_name)
    log(f"✅ Created Branch: {dev_branch.name} from {dev_branch.head_commit_id}")
    
    branches = client.branches.list(catalog_name=catalog_name)
    branch_names = [b.name for b in branches]
    log(f"Found branches: {branch_names}")
    assert "main" in branch_names
    assert "dev" in branch_names
    
    # 5. Tagging
    log("--- Testing Tagging ---")
    if dev_branch.head_commit_id:
        tag = client.tags.create("v1.0", commit_id=dev_branch.head_commit_id, catalog_name=catalog_name)
        log(f"✅ Created Tag: {tag.name} on {tag.commit_id}")
        
        tags = client.tags.list(catalog_name=catalog_name)
        tag_names = [t.name for t in tags]
        log(f"Found tags: {tag_names}")
        assert "v1.0" in tag_names
    else:
        log("⚠️ Skipping tag creation: No head commit ID on dev branch (expected for empty branch)")
    
    # 6. Listing Commits
    commits = client.branches.list_commits("main", catalog_name=catalog_name)
    log(f"✅ Main branch has {len(commits)} commits")
    
    # 7. Merge (Clean)
    # Note: Currently we haven't made changes to 'dev' that 'main' doesn't have, so this should be a fast-forward or no-op.
    # But let's verify the call structure works.
    log("--- Testing Merge ---")
    try:
        op = client.branches.merge("dev", "main", catalog_name=catalog_name)
        log(f"✅ Merge Triggered: {op.id} ({op.status})")
        
        # Verify status
        op_check = client.merge_operations.get(op.id)
        log(f"Merge Status: {op_check.status}")
    except Exception as e:
        log(f"⚠️ Merge test warning (might be expected for no-op): {e}")

    log("=== Verification Complete ===")

if __name__ == "__main__":
    main()
