import os
import sys
import time
import requests
import json
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, IntegerType, StringType
import pyarrow as pa

# Configuration
API_URL = os.environ.get("PANGOLIN_API_URL", "http://pangolin-api:8080")
MINIO_URL = os.environ.get("MINIO_URL", "http://minio:9000")
TEST_MODE = os.environ.get("TEST_MODE", "no-auth")

def log(msg):
    print(f"[TEST] {msg}", flush=True)

log(f"Configuration: API_URL={API_URL}, MINIO_URL={MINIO_URL}")

def wait_for_api():
    log(f"Waiting for API at {API_URL}...")
    for _ in range(30):
        try:
            resp = requests.get(f"{API_URL}/health")
            if resp.status_code == 200:
                log("API is up!")
                return
        except:
            pass
        time.sleep(2)
    raise Exception("API failed to start")

def test_no_auth():
    log("=== Starting NO AUTH Tests ===")
    
    # 1. Create Warehouse (MinIO)
    log("Creating Warehouse...")
    wh_payload = {
        "name": "minio-warehouse",
        "storage_config": {
            "type": "s3",
            "bucket": "warehouse",
            "endpoint": MINIO_URL,
            "region": "us-east-1",
            "access_key_id": "minioadmin",
            "secret_access_key": "minioadmin",
            "allow_http": "true"
        },
        "vending_strategy": "None"
    }
    # No Auth should accept this without token or with arbitrary token
    resp = requests.post(f"{API_URL}/api/v1/warehouses", json=wh_payload)
    if resp.status_code not in [201, 409]:
        raise Exception(f"Failed to create warehouse: {resp.text}")
    log("Warehouse created.")

    # 2. Create Catalog
    log("Creating Catalog...")
    cat_payload = {
        "name": "sales",
        "type": "rest",
        "warehouse": "minio-warehouse"
    }
    resp = requests.post(f"{API_URL}/api/v1/catalogs", json=cat_payload)
    if resp.status_code not in [201, 409]:
        raise Exception(f"Failed to create catalog: {resp.text}")
    log("Catalog created.")

    # 3. Create Namespace
    # Note: No Auth defaults to TenantAdmin, so we can do this.
    # We need to use the Iceberg REST API directly or PyIceberg. Let's use requests for setup.
    log("Creating Namespace 'marketing'...")
    ns_payload = {"namespace": ["marketing"]}
    resp = requests.post(f"{API_URL}/v1/sales/namespaces", json=ns_payload)
    if resp.status_code not in [200, 409]: # Iceberg returns 200 for create namespace usually, or 409 if exists
         # If 409, verify it exists. But pyiceberg might wrap this.
         # Actually Iceberg REST Create Namespace is POST /v1/{prefix}/namespaces
         pass 

    # 4. PyIceberg Operations
    log("Testing PyIceberg...")
    catalog = load_catalog(
        "pangolin",
        **{
            "uri": f"{API_URL}/v1/sales",
            "s3.endpoint": MINIO_URL,
            "s3.access-key-id": "minioadmin",
            "s3.secret-access-key": "minioadmin",
            "header.X-Pangolin-Tenant": "default-tenant", # Just in case, though No Auth might ignore
            "s3.path-style-access": "true"
        }
    )

    try:
        catalog.create_namespace("marketing")
    except Exception as e:
        if "AlreadyExists" not in str(e):
            log(f"Namespace creation note: {e}")

    schema = Schema(
        NestedField(1, "id", IntegerType(), required=True),
        NestedField(2, "data", StringType(), required=False),
    )

    table_name = "marketing.campaigns"
    try:
        catalog.drop_table(table_name)
    except:
        pass

    table = catalog.create_table(table_name, schema=schema)
    log(f"Table {table_name} created.")

    # Append
    df = pa.Table.from_pylist([{"id": 1, "data": "test"}, {"id": 2, "data": "run"}])
    table.append(df)
    log("Data inserted.")

    # Read
    read_df = table.scan().to_arrow()
    log(f"Read {len(read_df)} rows.")
    assert len(read_df) == 2

    log("=== NO AUTH Tests Passed ===")

def test_auth():
    log("=== Starting AUTH Tests ===")
    
    # 1. Login as Root
    log("Logging in as Root...")
    auth_resp = requests.post(f"{API_URL}/api/v1/login", json={"username": "admin", "password": "password"})
    if auth_resp.status_code != 200:
        raise Exception(f"Root login failed: {auth_resp.text}")
    root_token = auth_resp.json()["token"]
    root_headers = {"Authorization": f"Bearer {root_token}"}

    # 2. Create Tenant
    log("Creating Tenant 'AcmeCorp'...")
    tenant_payload = {"name": "AcmeCorp", "slug": "acme"} # Slug might be generated or required depending on impl
    # Assuming create_tenant handler
    resp = requests.post(f"{API_URL}/api/v1/tenants", json={"name":"AcmeCorp", "organization_name": "Acme"}, headers=root_headers)
    if resp.status_code == 201:
        tenant_id = resp.json()["id"]
    elif resp.status_code == 409:
        # Listing tenants to find it is annoying without filter, assuming clean state ideally
        # But let's try to proceed.
        tenants = requests.get(f"{API_URL}/api/v1/tenants", headers=root_headers).json()
        tenant_id = next(t["id"] for t in tenants if t["name"] == "AcmeCorp")
    else:
        raise Exception(f"Failed to create tenant: {resp.text}")
    
    log(f"Tenant ID: {tenant_id}")

    # 3. Create Tenant Admin
    log("Creating Tenant Admin...")
    admin_user = "admin_acme"
    admin_pass = "password123"
    user_payload = {
        "username": admin_user,
        "password": admin_pass, 
        "role": "TenantAdmin",
        "tenant_id": tenant_id
    }
    requests.post(f"{API_URL}/api/v1/users", json=user_payload, headers=root_headers)

    # 4. Login as Tenant Admin
    log("Logging in as Tenant Admin...")
    # NOTE: New tenant-scoped login requires tenant_id or distinct path?
    # Based on Implementation Plan, existing login is /api/v1/login.
    # Auth tests should verify if `tenant_id` param is needed in login payload.
    login_payload = {"username": admin_user, "password": admin_pass, "tenant_id": tenant_id}
    ta_resp = requests.post(f"{API_URL}/api/v1/login", json=login_payload)
    if ta_resp.status_code != 200:
        raise Exception(f"Tenant Admin login failed: {ta_resp.text}")
    ta_token = ta_resp.json()["token"]
    ta_headers = {"Authorization": f"Bearer {ta_token}", "X-Pangolin-Tenant": tenant_id}

    # 5. Create Warehouse & Catalog (as Tenant Admin)
    wh_payload = {
        "name": "acme-warehouse",
        "storage_config": {
            "type": "s3",
            "bucket": "warehouse-acme",
            "endpoint": MINIO_URL,
            "region": "us-east-1",
            "access_key_id": "minioadmin",
            "secret_access_key": "minioadmin",
            "allow_http": "true"
        },
         "vending_strategy": {
             "AwsStatic": {
                 "access_key_id": "minioadmin", 
                 "secret_access_key": "minioadmin"
             }
         }
    }
    requests.post(f"{API_URL}/api/v1/warehouses", json=wh_payload, headers=ta_headers)

    cat_payload = {
        "name": "analytics",
        "type": "rest",
        "warehouse": "acme-warehouse"
    }
    requests.post(f"{API_URL}/api/v1/catalogs", json=cat_payload, headers=ta_headers)

    # 6. PyIceberg with Credential Vending
    log("Testing PyIceberg Vended Credentials...")
    # We pass NO credentials, but we pass the token.
    # We need to manually construct the catalog to pass headers properly if load_catalog doesn't support custom headers easily in all versions, 
    # but PyIceberg supports 'header.X-Pangolin-Tenant'.
    
    catalog = load_catalog(
        "pangolin_auth",
        **{
            "uri": f"{API_URL}/v1/analytics",
            "token": ta_token,
            "header.X-Pangolin-Tenant": tenant_id,
           "s3.path-style-access": "true"
            # No S3 keys!
        }
    )

    try:
        catalog.create_namespace("reports")
    except Exception as e:
         if "AlreadyExists" not in str(e):
            log(f"Namespace creation note: {e}")

    schema = Schema(NestedField(1, "x", IntegerType(), True))
    table = catalog.create_table("reports.daily", schema=schema)
    table.append(pa.Table.from_pylist([{"x": 100}]))
    
    log("=== AUTH Tests Passed ===")

if __name__ == "__main__":
    wait_for_api()
    if TEST_MODE == "no-auth":
        test_no_auth()
    else:
        test_auth()
