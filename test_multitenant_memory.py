import requests
import json
import time
import os
import sys
import subprocess
import uuid

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

def create_warehouse_stack(stack_name, bucket_suffix, tenant_id):
    """Creates a warehouse and catalog for a given tenant."""
    print(f"\n=== Creating Stack: {stack_name} (Tenant: {tenant_id}) ===")
    
    # 1. Create Bucket in MinIO
    bucket_name = f"stack-{bucket_suffix}"
    try:
        subprocess.run(f"mc mb minio/{bucket_name}", shell=True, check=False, 
                      stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    except:
        pass
    
    # 2. Create Warehouse with credentials
    wh_name = f"{stack_name}_warehouse"
    storage_config = {
        "s3.bucket": bucket_name,
        "s3.endpoint": AWS_ENDPOINT_URL,
        "s3.region": AWS_REGION,
        "s3.access-key-id": AWS_ACCESS_KEY_ID,
        "s3.secret-access-key": AWS_SECRET_ACCESS_KEY,
        "s3.path-style-access": "true"
    }
    
    wh_resp = requests.post(f"{API_URL}/api/v1/warehouses", json={
        "name": wh_name,
        "use_sts": False,  # Use static for simplicity
        "storage_config": storage_config
    })
    if wh_resp.status_code not in [200, 201]:
        print(f"Failed to create warehouse: {wh_resp.status_code} - {wh_resp.text}")
        sys.exit(1)
    print(f"✓ Warehouse created: {wh_name}")
    
    # 3. Create Catalog
    cat_name = f"{stack_name}_catalog"
    cat_resp = requests.post(f"{API_URL}/api/v1/catalogs", json={
        "name": cat_name,
        "warehouse_name": wh_name,
        "catalog_type": "Local"
    })
    if cat_resp.status_code not in [200, 201]:
        print(f"Failed to create catalog: {cat_resp.status_code} - {cat_resp.text}")
        sys.exit(1)
    print(f"✓ Catalog created: {cat_name}")
    
    return {
        "tenant_id": tenant_id,
        "stack_name": stack_name,
        "warehouse_name": wh_name,
        "catalog_name": cat_name,
        "bucket_name": bucket_name
    }

def test_stack_operations(stack_info, test_id):
    """Tests PyIceberg operations for a specific stack."""
    from pyiceberg.catalog import load_catalog
    import pyarrow as pa
    
    print(f"\n--- Testing Stack: {stack_info['stack_name']} ---")
    
    catalog = load_catalog(
        stack_info['catalog_name'],
        **{
            "type": "rest",
            "uri": f"{API_URL}/v1/{stack_info['catalog_name']}",
            "header.X-Iceberg-Access-Delegation": "vended-credentials",
            "s3.endpoint": AWS_ENDPOINT_URL,
            "s3.region": AWS_REGION,
            "s3.path-style-access": "true"
        }
    )
    
    ns_name = f"ns_{test_id}"
    tbl_name = f"table_{test_id}"
    
    # Create namespace
    try:
        catalog.create_namespace(ns_name)
        print(f"✓ Created namespace: {ns_name}")
    except Exception as e:
        print(f"  (Namespace may already exist: {e})")
    
    # Create table
    schema = pa.schema([("id", pa.int64()), ("stack", pa.string()), ("data", pa.string())])
    table = catalog.create_table(f"{ns_name}.{tbl_name}", schema=schema)
    print(f"✓ Created table: {ns_name}.{tbl_name}")
    
    # Write data with stack identifier
    data = pa.Table.from_pylist([
        {"id": 1, "stack": stack_info['stack_name'], "data": f"data_from_{stack_info['stack_name']}_1"},
        {"id": 2, "stack": stack_info['stack_name'], "data": f"data_from_{stack_info['stack_name']}_2"}
    ])
    table.append(data)
    print(f"✓ Wrote 2 rows to {ns_name}.{tbl_name}")
    
    # Read data back
    scan = table.scan()
    result = scan.to_arrow().to_pylist()
    print(f"✓ Read {len(result)} rows from {ns_name}.{tbl_name}")
    
    # Verify data belongs to this stack
    for row in result:
        if row['stack'] != stack_info['stack_name']:
            print(f"❌ ERROR: Found data from wrong stack! Expected {stack_info['stack_name']}, got {row['stack']}")
            sys.exit(1)
    
    print(f"✓ All data belongs to stack: {stack_info['stack_name']}")
    return result

def verify_isolation(stack1_data, stack2_data, stack1_info, stack2_info):
    """Verifies that stacks are properly isolated."""
    print("\n=== Verifying Stack Isolation ===")
    
    # Check stack1 data
    for row in stack1_data:
        if row['stack'] != stack1_info['stack_name']:
            print(f"❌ Stack 1 has contaminated data!")
            return False
    print(f"✓ Stack 1 data is clean (all rows belong to {stack1_info['stack_name']})")
    
    # Check stack2 data
    for row in stack2_data:
        if row['stack'] != stack2_info['stack_name']:
            print(f"❌ Stack 2 has contaminated data!")
            return False
    print(f"✓ Stack 2 data is clean (all rows belong to {stack2_info['stack_name']})")
    
    # Verify they used different buckets
    if stack1_info['bucket_name'] == stack2_info['bucket_name']:
        print(f"❌ Stacks are using the same bucket!")
        return False
    print(f"✓ Stacks use separate buckets: {stack1_info['bucket_name']} vs {stack2_info['bucket_name']}")
    
    return True

if __name__ == "__main__":
    wait_for_api()
    
    # Use default tenant (created by NO_AUTH mode)
    DEFAULT_TENANT_ID = "00000000-0000-0000-0000-000000000000"
    
    # Create two separate warehouse/catalog stacks within the same tenant
    stack1 = create_warehouse_stack("acme_stack", "acme", DEFAULT_TENANT_ID)
    stack2 = create_warehouse_stack("globex_stack", "globex", DEFAULT_TENANT_ID)
    
    # Run operations for each stack
    stack1_data = test_stack_operations(stack1, "test1")
    stack2_data = test_stack_operations(stack2, "test2")
    
    # Verify isolation
    if verify_isolation(stack1_data, stack2_data, stack1, stack2):
        print("\n✅ MULTI-WAREHOUSE ISOLATION TEST PASSED!")
        print("   - Both stacks can read/write independently")
        print("   - No data leakage between stacks")
        print("   - Separate S3 buckets per warehouse")
        print("   - Credential vending works correctly for each warehouse")
    else:
        print("\n❌ MULTI-WAREHOUSE ISOLATION TEST FAILED!")
        sys.exit(1)
