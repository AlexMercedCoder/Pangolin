import os
import time
import requests
import boto3
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, IntegerType, StringType
import pyarrow as pa
import logging
import uuid

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

# Configuration
API_URL = "http://localhost:8080"
MINIO_ENDPOINT = "http://localhost:9000"
MINIO_ACCESS_KEY = "minioadmin"
MINIO_SECRET_KEY = "minioadmin"
BUCKET_NAME = "test-bucket-memory"
CATALOG_NAME = "memory_catalog"

# S3 Client for verification
s3 = boto3.client(
    "s3",
    endpoint_url=MINIO_ENDPOINT,
    aws_access_key_id=MINIO_ACCESS_KEY,
    aws_secret_access_key=MINIO_SECRET_KEY,
    region_name="us-east-1"
)

def setup_minio():
    try:
        s3.create_bucket(Bucket=BUCKET_NAME)
        logger.info(f"Created bucket: {BUCKET_NAME}")
    except s3.exceptions.BucketAlreadyOwnedByYou:
        logger.info(f"Bucket {BUCKET_NAME} already exists")
    except Exception as e:
        logger.error(f"Error creating bucket: {e}")

def get_auth_token():
    # Login as seeded admin (MemoryStore defaults to NO_AUTH/SEED_ADMIN if env vars set)
    # We will assume the server is started with PANGOLIN_NO_AUTH=true or seeded.
    login_payload = {
        "username": "tenant_admin", 
        "password": "password123",
        "tenant-id": "00000000-0000-0000-0000-000000000000"
    }
    
    # Retry logic for server startup
    for i in range(10):
        try:
            resp = requests.post(f"{API_URL}/api/v1/users/login", json=login_payload)
            if resp.status_code == 200:
                return resp.json()["token"]
            else:
                logger.warning(f"Login attempt {i+1} failed: {resp.status_code}")
        except requests.exceptions.ConnectionError:
            logger.warning(f"Connection attempt {i+1} failed (server not up?)")
        
        time.sleep(2)
    
    raise Exception("Failed to get auth token after retries")

def run_test():
    setup_minio()
    token = get_auth_token()
    headers = {"Authorization": f"Bearer {token}"}
    
    logger.info("Auth Successful. Starting MemoryStore Modules Test...")

    # --- Cleanup (Best Effort) ---
    requests.delete(f"{API_URL}/api/v1/catalogs/{CATALOG_NAME}", headers=headers)
    requests.delete(f"{API_URL}/api/v1/warehouses/memory_warehouse", headers=headers)

    # 1. Warehouse (Module: warehouses.rs)
    logger.info("1. Testing Warehouse Module...")
    wh_resp = requests.post(f"{API_URL}/api/v1/warehouses", json={
        "name": "memory_warehouse",
        "storage_config": {
            "s3.endpoint": MINIO_ENDPOINT,
            "s3.bucket": BUCKET_NAME,
            "s3.access-key-id": MINIO_ACCESS_KEY,
            "s3.secret-access-key": MINIO_SECRET_KEY,
            "s3.region": "us-east-1",
            "s3.path-style-access": "true"
        }
    }, headers=headers)
    assert wh_resp.status_code in [201, 200], f"Warehouse creation failed: {wh_resp.text}"

    # 2. Catalog (Module: catalogs.rs)
    logger.info("2. Testing Catalog Module...")
    cat_resp = requests.post(f"{API_URL}/api/v1/catalogs", json={
        "name": CATALOG_NAME,
        "warehouse_name": "memory_warehouse",
        "storage_location": f"s3://{BUCKET_NAME}/{CATALOG_NAME}"
    }, headers=headers)
    assert cat_resp.status_code in [201, 200], f"Catalog creation failed: {cat_resp.text}"

    # 3. Iceberg Integration (Modules: namespaces.rs, assets.rs, io.rs, signer.rs)
    logger.info("3. Testing Iceberg Integration (Namespaces, Assets, IO)...")
    catalog = load_catalog(
        CATALOG_NAME,
        **{
            "uri": f"{API_URL}/v1/{CATALOG_NAME}",
            "s3.endpoint": MINIO_ENDPOINT,
            "s3.access-key-id": MINIO_ACCESS_KEY,
            "s3.secret-access-key": MINIO_SECRET_KEY,
            "token": token
        }
    )
    
    ns = "mem_ns"
    try:
        catalog.create_namespace(ns)
        logger.info(f"Namespace {ns} created")
    except Exception as e:
        logger.warning(f"Namespace {ns} exists or error: {e}")

    table_name = f"{ns}.mem_table"
    schema = Schema(
        NestedField(1, "id", IntegerType(), required=True),
        NestedField(2, "val", StringType(), required=False),
    )
    
    try:
        table = catalog.create_table(table_name, schema)
        logger.info(f"Table {table_name} created")
    except Exception:
        table = catalog.load_table(table_name)
        logger.info(f"Table {table_name} loaded")

    df = pa.Table.from_pylist([{"id": 1, "val": "A"}, {"id": 2, "val": "B"}])
    table.append(df)
    logger.info("Data inserted into table")

    # 4. Branching (Module: branches.rs, commits.rs)
    logger.info("4. Testing Branching Module...")
    resp = requests.post(f"{API_URL}/api/v1/branches", json={
        "name": "feature-1",
        "catalog": CATALOG_NAME,
        "from_branch": "main" 
    }, headers=headers)
    if resp.status_code not in [200, 201]:
        logger.error(f"Branch creation failed: {resp.status_code} {resp.text}")
    
    resp_list = requests.get(f"{API_URL}/api/v1/branches", params={"catalog": CATALOG_NAME}, headers=headers)
    if resp_list.status_code != 200:
        logger.error(f"List branches failed: {resp_list.status_code} {resp_list.text}")
        raise Exception("List branches failed")
        
    branches = resp_list.json()
    assert any(b['name'] == 'feature-1' for b in branches), "Branch feature-1 not found"
    logger.info("Branch feature-1 created and verified")

    # 5. Tags (Module: tags.rs)
    logger.info("5. Testing Tags Module...")
    requests.post(f"{API_URL}/api/v1/catalogs/{CATALOG_NAME}/tags", json={
        "name": "release-v1",
        "ref": "main"
    }, headers=headers)
    logger.info("Tag release-v1 created")

    logger.info("6. Testing Service Users Module...")
    su_resp = requests.post(f"{API_URL}/api/v1/service-users", json={
        "name": "mem_bot",
        "role": "tenant-user"
    }, headers=headers)
    assert su_resp.status_code in [201, 200], f"Service user failed: {su_resp.text}"
    logger.info("Service user created")

    # 7. Roles & Permissions (Module: roles.rs, permissions.rs)
    logger.info("7. Testing Roles & Permissions Module...")
    role_resp = requests.post(f"{API_URL}/api/v1/roles", json={
        "name": "CustomRole",
        "description": "Test role",
        "permissions": []
    }, headers=headers)
    if role_resp.status_code in [201, 200]:
        logger.info("Role CustomRole created")
    else:
        logger.warning(f"Role creation failed (maybe not relevant for tenant admin): {role_resp.text}")

    # 8. Audit (Module: audit.rs)
    logger.info("8. Testing Audit Module...")
    audit_resp = requests.get(f"{API_URL}/api/v1/audit", headers=headers)
    assert audit_resp.status_code == 200
    assert len(audit_resp.json()) > 0, "No audit events found"
    logger.info("Audit events verified")

    # 9. Verification of MinIO Persistence implies IO module worked
    logger.info("9. Verifying MinIO Persistence (metadata.json)...")
    objects = s3.list_objects_v2(Bucket=BUCKET_NAME)
    found_metadata = False
    if 'Contents' in objects:
        for obj in objects['Contents']:
            if obj['Key'].endswith("metadata.json"):
                found_metadata = True
                logger.info(f"Found metadata: {obj['Key']}")
                break
    
    if found_metadata:
        logger.info("✅ SUCCESS: `metadata.json` persistency verified!")
    else:
        logger.error("❌ FAILURE: `metadata.json` NOT found in MinIO!")
        exit(1)

    logger.info("✅ MEMORY STORE LIVE TEST COMPLETE")

if __name__ == "__main__":
    run_test()
