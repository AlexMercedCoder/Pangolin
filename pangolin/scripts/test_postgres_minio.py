import os
import time
import requests
import boto3
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, IntegerType, StringType
import pyarrow as pa
import logging

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Configuration
API_URL = "http://localhost:8080"
MINIO_ENDPOINT = "http://localhost:9000"
MINIO_ACCESS_KEY = "minioadmin"
MINIO_SECRET_KEY = "minioadmin"
BUCKET_NAME = "test-bucket"
CATALOG_NAME = "test_catalog"

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

def create_tenant_and_warehouse():
    # 1. Create Tenant (if not exists - usually seeded or we just act as admin)
    # For simplicity, we'll assume a default tenant or use the API to create one.
    # Pangolin usually needs a tenant context.
    
    # 2. Create Warehouse
    warehouse_payload = {
        "name": "test_warehouse",
        "storage_config": {
            "s3.endpoint": MINIO_ENDPOINT,
            "s3.bucket": BUCKET_NAME,
            "s3.access-key-id": MINIO_ACCESS_KEY,
            "s3.secret-access-key": MINIO_SECRET_KEY,
            "s3.region": "us-east-1"
        }
    }
    # Using 'admin' auth typically needed. Assuming no-auth or basic default for now?
    # Checking docker-compose, PANGOLIN_NO_AUTH is not set, so AUTH is enabled.
    # We need a token.
    return warehouse_payload

def get_auth_token():
    # Login as default seeded admin
    login_payload = {
        "username": "tenant_admin", 
        "password": "password123",
        "tenant-id": "00000000-0000-0000-0000-000000000000"
    }
    resp = requests.post(f"{API_URL}/api/v1/users/login", json=login_payload)
    if resp.status_code == 200:
        return resp.json()["token"]
    
    print(f"Login failed: {resp.status_code} {resp.text}")
    raise Exception("Failed to get auth token")

def run_test():
    setup_minio()
    token = get_auth_token()
    headers = {"Authorization": f"Bearer {token}"}

    # --- Cleanup Step ---
    logger.info("Cleaning up existing catalog and warehouse...")
    requests.delete(f"{API_URL}/api/v1/catalogs/{CATALOG_NAME}", headers=headers)
    requests.delete(f"{API_URL}/api/v1/warehouses/test_warehouse", headers=headers)

    # 1. Create Warehouse
    logger.info("Creating warehouse...")
    requests.post(f"{API_URL}/api/v1/warehouses", json={
        "name": "test_warehouse",
        "storage_config": {
            "s3.endpoint": MINIO_ENDPOINT,
            "s3.bucket": BUCKET_NAME,
            "s3.access-key-id": MINIO_ACCESS_KEY,
            "s3.secret-access-key": MINIO_SECRET_KEY,
            "s3.region": "us-east-1",
            "s3.path-style-access": "true",
            "bucket": BUCKET_NAME # Legacy compat
        }
    }, headers=headers)

    # 2. Create Catalog
    logger.info("Creating catalog...")
    requests.post(f"{API_URL}/api/v1/catalogs", json={
        "name": CATALOG_NAME,
        "warehouse_name": "test_warehouse",
        "storage_location": f"s3://{BUCKET_NAME}/{CATALOG_NAME}"
    }, headers=headers)

    logger.info("Catalog created. Configuring PyIceberg...")

    # PyIceberg Config
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

    # 2. Create Namespace (Touch namespaces module)
    ns = "test_ns"
    try:
        catalog.create_namespace(ns)
        logger.info(f"Namespace {ns} created")
    except Exception as e:
        logger.warning(f"Namespace creation: {e}")

    # 3. Create Table (Touch assets, commits module)
    table_name = f"{ns}.test_table"
    schema = Schema(
        NestedField(1, "id", IntegerType(), required=True),
        NestedField(2, "data", StringType(), required=False),
    )
    
    try:
        table = catalog.create_table(table_name, schema)
        logger.info(f"Table {table_name} created")
    except Exception as e:
        table = catalog.load_table(table_name)
        logger.info(f"Table {table_name} loaded")

    # 4. Append Data (Touch commits, snapshots)
    df = pa.Table.from_pylist([
        {"id": 1, "data": "foo"},
        {"id": 2, "data": "bar"},
    ])
    table.append(df)
    logger.info("Data appended")

    # 5. Create Branch (Touch branches module)
    branch_name = "dev"
    requests.post(f"{API_URL}/api/v1/catalogs/{CATALOG_NAME}/branches", json={
        "name": branch_name,
        "ref": "main" 
    }, headers=headers)
    logger.info(f"Branch {branch_name} created")
    
    # 6. Create Tag (Touch tags module)
    tag_name = "v1"
    requests.post(f"{API_URL}/api/v1/catalogs/{CATALOG_NAME}/tags", json={
        "name": tag_name,
        "ref": "main"
    }, headers=headers)
    logger.info(f"Tag {tag_name} created")

    # 7. Service Users (Touch service_users module)
    svc_user_name = "ci_bot"
    requests.post(f"{API_URL}/api/v1/service-users", json={
        "name": svc_user_name,
        "role": "TenantUser"
    }, headers=headers)
    logger.info("Service User created")

    # 8. Audit (Touch audit module)
    # Check if we can list audit events
    audit_resp = requests.get(f"{API_URL}/api/v1/audit", headers=headers)
    if audit_resp.status_code == 200:
        events = audit_resp.json()
        logger.info(f"Audit events fetched: {len(events)}")
    else:
        logger.error(f"Audit fetch failed: {audit_resp.text}")

    # 9. Verify Metadata in MinIO
    logger.info("Verifying Metadata in MinIO...")
    # Expected path: bucket/warehouse/catalog/namespace/table/metadata/metadata.json
    # Or roughly around there. We'll list objects.
    objects = s3.list_objects_v2(Bucket=BUCKET_NAME)
    found_metadata = False
    if 'Contents' in objects:
        for obj in objects['Contents']:
            key = obj['Key']
            logger.info(f"Found object: {key}")
            if key.endswith("metadata.json"):
                found_metadata = True
    
    if found_metadata:
        logger.info("SUCCESS: `metadata.json` found in MinIO!")
    else:
        logger.error("FAILURE: `metadata.json` NOT found in MinIO!")
        exit(1)

    # 10. Access Requests (Touch access_requests module)
    # Just create one to test the endpoint
    requests.post(f"{API_URL}/api/v1/access-requests", json={
        "asset_id": "00000000-0000-0000-0000-000000000000", # Dummy ID
        "reason": "Test request"
    }, headers=headers)
    logger.info("Access Request created (dummy)")

if __name__ == "__main__":
    run_test()
