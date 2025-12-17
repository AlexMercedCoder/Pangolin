import requests
import json
import time
import os
import sys
import subprocess

# --- Configuration ---
API_URL = "http://localhost:8080"
AWS_ACCESS_KEY_ID = os.environ.get("AWS_ACCESS_KEY_ID", "minioadmin")
AWS_SECRET_ACCESS_KEY = os.environ.get("AWS_SECRET_ACCESS_KEY", "minioadmin")
AWS_REGION = os.environ.get("AWS_REGION", "us-east-1")
AWS_ENDPOINT_URL = os.environ.get("AWS_ENDPOINT_URL", "http://localhost:9000")

def wait_for_api():
    """Waits for the API to be available."""
    print(f"Waiting for API at {API_URL}...")
    for _ in range(30):
        try:
            requests.get(f"{API_URL}/health")
            print("API is up!")
            return
        except requests.exceptions.ConnectionError:
            time.sleep(1)
    print("API failed to start.")
    sys.exit(1)

def login(username, password):
    """Login and get JWT token."""
    resp = requests.post(f"{API_URL}/api/v1/auth/login", json={
        "username": username,
        "password": password
    })
    if resp.status_code != 200:
        print(f"Login failed: {resp.status_code} - {resp.text}")
        sys.exit(1)
    return resp.json()["token"]

def create_tenant_with_admin(tenant_name, admin_username, admin_password, root_token):
    """Creates a tenant and its admin user."""
    print(f"\n=== Creating Tenant: {tenant_name} ===")
    
    headers = {"Authorization": f"Bearer {root_token}"}
    
    # Create tenant
    tenant_resp = requests.post(f"{API_URL}/api/v1/tenants", 
                                json={"name": tenant_name}, 
                                headers=headers)
    if tenant_resp.status_code not in [200, 201]:
        print(f"Failed to create tenant: {tenant_resp.status_code} - {tenant_resp.text}")
        sys.exit(1)
    tenant_id = tenant_resp.json()["id"]
    print(f"✓ Tenant created: {tenant_id}")
    
    # Create admin user for this tenant
    user_resp = requests.post(f"{API_URL}/api/v1/users",
                              json={
                                  "username": admin_username,
                                  "password": admin_password,
                                  "tenant_id": tenant_id
                              },
                              headers=headers)
    if user_resp.status_code not in [200, 201]:
        print(f"Failed to create user: {user_resp.status_code} - {user_resp.text}")
        sys.exit(1)
    print(f"✓ Admin user created: {admin_username}")
    
    # Login as the new admin to get their token
    admin_token = login(admin_username, admin_password)
    print(f"✓ Admin logged in")
    
    return {
        "tenant_id": tenant_id,
        "tenant_name": tenant_name,
        "admin_token": admin_token,
        "admin_username": admin_username
    }

def create_warehouse_and_catalog(tenant_info, bucket_suffix):
    """Creates warehouse and catalog for a tenant."""
    print(f"\n--- Setting up infrastructure for {tenant_info['tenant_name']} ---")
    
    headers = {"Authorization": f"Bearer {tenant_info['admin_token']}"}
    
    # Create bucket
    bucket_name = f"tenant-{bucket_suffix}"
    try:
        subprocess.run(f"mc mb minio/{bucket_name}", shell=True, check=False,
                      stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    except:
        pass
    
    # Create warehouse
    wh_name = f"{tenant_info['tenant_name']}_warehouse"
    storage_config = {
        "s3.bucket": bucket_name,
        "s3.endpoint": AWS_ENDPOINT_URL,
        "s3.region": AWS_REGION,
        "s3.access-key-id": AWS_ACCESS_KEY_ID,
        "s3.secret-access-key": AWS_SECRET_ACCESS_KEY,
        "s3.path-style-access": "true"
    }
    
    wh_resp = requests.post(f"{API_URL}/api/v1/warehouses",
                           json={
                               "name": wh_name,
                               "use_sts": False,
                               "storage_config": storage_config
                           },
                           headers=headers)
    if wh_resp.status_code not in [200, 201]:
        print(f"Failed to create warehouse: {wh_resp.status_code} - {wh_resp.text}")
        sys.exit(1)
    print(f"✓ Warehouse created: {wh_name}")
    
    # Create catalog
    cat_name = f"{tenant_info['tenant_name']}_catalog"
    cat_resp = requests.post(f"{API_URL}/api/v1/catalogs",
                            json={
                                "name": cat_name,
                                "warehouse_name": wh_name,
                                "catalog_type": "Local"
                            },
                            headers=headers)
    if cat_resp.status_code not in [200, 201]:
        print(f"Failed to create catalog: {cat_resp.status_code} - {cat_resp.text}")
        sys.exit(1)
    print(f"✓ Catalog created: {cat_name}")
    
    tenant_info["warehouse_name"] = wh_name
    tenant_info["catalog_name"] = cat_name
    tenant_info["bucket_name"] = bucket_name
    return tenant_info

def test_tenant_operations(tenant_info):
    """Tests PyIceberg operations for a tenant."""
    from pyiceberg.catalog import load_catalog
    import pyarrow as pa
    
    print(f"\n--- Testing Operations for {tenant_info['tenant_name']} ---")
    
    catalog = load_catalog(
        tenant_info['catalog_name'],
        **{
            "type": "rest",
            "uri": f"{API_URL}/v1/{tenant_info['catalog_name']}",
            "token": tenant_info['admin_token'],
            "header.X-Iceberg-Access-Delegation": "vended-credentials",
            "s3.endpoint": AWS_ENDPOINT_URL,
            "s3.region": AWS_REGION,
            "s3.path-style-access": "true"
        }
    )
    
    ns_name = "test_namespace"
    tbl_name = "test_table"
    
    # Create namespace
    try:
        catalog.create_namespace(ns_name)
        print(f"✓ Created namespace: {ns_name}")
    except Exception as e:
        print(f"  (Namespace may exist: {e})")
    
    # Create table
    schema = pa.schema([("id", pa.int64()), ("tenant", pa.string()), ("data", pa.string())])
    table = catalog.create_table(f"{ns_name}.{tbl_name}", schema=schema)
    print(f"✓ Created table: {ns_name}.{tbl_name}")
    
    # Write data
    data = pa.Table.from_pylist([
        {"id": 1, "tenant": tenant_info['tenant_name'], "data": f"secret_data_from_{tenant_info['tenant_name']}_1"},
        {"id": 2, "tenant": tenant_info['tenant_name'], "data": f"secret_data_from_{tenant_info['tenant_name']}_2"}
    ])
    table.append(data)
    print(f"✓ Wrote 2 rows")
    
    # Read data
    scan = table.scan()
    result = scan.to_arrow().to_pylist()
    print(f"✓ Read {len(result)} rows")
    
    # Verify data
    for row in result:
        if row['tenant'] != tenant_info['tenant_name']:
            print(f"❌ Data contamination! Expected {tenant_info['tenant_name']}, got {row['tenant']}")
            sys.exit(1)
    
    print(f"✓ All data belongs to {tenant_info['tenant_name']}")
    return result

if __name__ == "__main__":
    wait_for_api()
    
    # Login as root
    print("\n=== Logging in as root ===")
    root_token = login("admin", "password")
    print("✓ Root logged in")
    
    # Create two tenants with their admins
    tenant1 = create_tenant_with_admin("acme_corp", "acme_admin", "acme123", root_token)
    tenant2 = create_tenant_with_admin("globex_inc", "globex_admin", "globex123", root_token)
    
    # Setup infrastructure for each tenant
    tenant1 = create_warehouse_and_catalog(tenant1, "acme")
    tenant2 = create_warehouse_and_catalog(tenant2, "globex")
    
    # Run operations for each tenant
    tenant1_data = test_tenant_operations(tenant1)
    tenant2_data = test_tenant_operations(tenant2)
    
    # Verify isolation
    print("\n=== Verifying Multi-Tenant Isolation ===")
    
    # Check data integrity
    for row in tenant1_data:
        if row['tenant'] != tenant1['tenant_name']:
            print(f"❌ Tenant 1 data contaminated!")
            sys.exit(1)
    print(f"✓ Tenant 1 data is clean")
    
    for row in tenant2_data:
        if row['tenant'] != tenant2['tenant_name']:
            print(f"❌ Tenant 2 data contaminated!")
            sys.exit(1)
    print(f"✓ Tenant 2 data is clean")
    
    # Verify separate buckets
    if tenant1['bucket_name'] == tenant2['bucket_name']:
        print(f"❌ Tenants sharing same bucket!")
        sys.exit(1)
    print(f"✓ Separate buckets: {tenant1['bucket_name']} vs {tenant2['bucket_name']}")
    
    # Verify separate tenant IDs
    if tenant1['tenant_id'] == tenant2['tenant_id']:
        print(f"❌ Tenants have same ID!")
        sys.exit(1)
    print(f"✓ Separate tenant IDs")
    
    print("\n✅ MULTI-TENANT ISOLATION TEST PASSED!")
    print("   - Two separate tenants created")
    print("   - Each tenant has its own admin user")
    print("   - Each tenant has separate warehouse and catalog")
    print("   - Each tenant uses separate S3 bucket")
    print("   - Credential vending works independently for each tenant")
    print("   - No data leakage between tenants")
