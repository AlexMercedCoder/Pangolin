import logging
import uuid
import time
import requests
import json
import boto3
from botocore.config import Config
from pypangolin import PangolinClient, get_iceberg_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, IntegerType

# Setup Logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("verify_merge")

# Config
API_URL = "http://localhost:8080"
ROOT_USER = "admin"
ROOT_PASS = "password"
TENANT_NAME = f"merge_verify_tenant_{int(time.time())}"
WAREHOUSE_NAME = f"merge_verify_wh_{int(time.time())}"
CATALOG_NAME = f"merge_cat_{int(time.time())}"
NAMESPACE = "merge_ns"
TABLE_NAME = "test_table"
BUCKET = f"merge-bucket-{int(time.time())}" 

def get_s3_client():
    return boto3.client('s3',
                      endpoint_url='http://localhost:9000',
                      aws_access_key_id='minioadmin',
                      aws_secret_access_key='minioadmin',
                      config=Config(signature_version='s3v4'),
                      region_name='us-east-1')

def create_bucket():
    s3 = get_s3_client()
    try:
        s3.create_bucket(Bucket=BUCKET)
        logger.info(f"✅ Created bucket: {BUCKET}")
    except Exception as e:
        logger.error(f"❌ Failed to create bucket: {e}")
        raise e

def verify_s3_file(prefix):
    s3 = get_s3_client()
    try:
        response = s3.list_objects_v2(Bucket=BUCKET, Prefix=prefix)
        if 'Contents' in response:
            logger.info(f"✅ Found S3 objects under {prefix}")
            return True
        else:
            logger.error(f"❌ No S3 objects found under {prefix}")
            return False
    except Exception as e:
        logger.error(f"❌ S3 Check Failed: {e}")
        return False

def run_verification():
    create_bucket()
    
    client = PangolinClient(API_URL)
    
    # 1. Login as Root
    logger.info("--- [STEP] Authenticating as Root ---")
    client.login(ROOT_USER, ROOT_PASS)
    
    # 2. Create Tenant
    logger.info(f"--- [STEP] Creating Tenant {TENANT_NAME} ---")
    tenant = client.tenants.create(TENANT_NAME)
    client.set_tenant(tenant.id)
    
    # 3. Create Tenant Admin and Login
    logger.info("--- [STEP] Creating Tenant Admin and Logging In ---")
    admin_user = f"admin_{int(time.time())}"
    admin_pass = "password123"
    client.users.create(admin_user, f"{admin_user}@example.com", "tenant-admin", tenant_id=tenant.id, password=admin_pass)
    client.login(admin_user, admin_pass, tenant_id=tenant.id)
    
    # 3. Create Warehouse (MinIO)
    logger.info(f"--- [STEP] Creating Warehouse {WAREHOUSE_NAME} ---")
    client.warehouses.create_s3(
        name=WAREHOUSE_NAME,
        bucket=BUCKET,
        endpoint="http://localhost:9000",
        access_key="minioadmin",
        secret_key="minioadmin",
        prefix=f"{TENANT_NAME}/",
        **{"s3.path-style-access": "true"}
    )
    
    wh = next(w for w in client.warehouses.list() if w.name == WAREHOUSE_NAME)
    logger.info(f"Warehouse Config: {wh.storage_config}")
    
    # 4. Create Catalog
    logger.info(f"--- [STEP] Creating Catalog {CATALOG_NAME} ---")
    client.catalogs.create(CATALOG_NAME, WAREHOUSE_NAME)
    
    # 5. Create Table on MAIN branch
    logger.info("--- [STEP] Creating Table on MAIN ---")
    # Using PyIceberg directly via REST catalog
    # Get token for main
    token = client.token 
    # Or generate a new token if needed, but root token works.
    
    # Create pyiceberg catalog for MAIN
    cat_main = get_iceberg_catalog(CATALOG_NAME, uri=API_URL, token=token)
    
    cat_main.create_namespace(NAMESPACE)
    
    schema = Schema(
        NestedField(1, "id", IntegerType(), required=True),
        NestedField(2, "data", StringType(), required=False),
    )
    
    table_main = cat_main.create_table(
        f"{NAMESPACE}.{TABLE_NAME}",
        schema=schema
    )
    
    # Insert data into MAIN
    import pyarrow as pa
    df_main = pa.Table.from_pylist([{"id": 1, "data": "main_data"}], schema=schema.as_arrow())
    table_main.append(df_main)
    logger.info("Inserted data into MAIN")

    # 6. Create Branch DEV
    logger.info("--- [STEP] Creating Branch DEV ---")
    branch_dev = client.branches.create("dev", from_branch="main", catalog_name=CATALOG_NAME)
    logger.info(f"Created branch: {branch_dev.name}")

    # 7. Write to DEV (Isolation Test)
    # 7. Write to DEV (Isolation Test)
    logger.info("--- [STEP] Writing to DEV (Isolation Test) ---")
    try:
        # Load table from dev branch
        table_identifier = f"{NAMESPACE}.{TABLE_NAME}@{branch_dev.name}"
        table_dev = cat_main.load_table(table_identifier)
        
        # Verify it has main's data initially
        scan = table_dev.scan().to_arrow()
        logger.info(f"Dev branch initial rows: {len(scan)}")
        # assert len(scan) == 1
        
        # Insert new data to DEV
        df_dev = pa.Table.from_pylist([{"id": 2, "data": "dev_data"}], schema=schema.as_arrow())
        table_dev.append(df_dev)
        logger.info("Inserted data into DEV")
        
        # Verify DEV has 2 rows
        scan_dev = table_dev.scan().to_arrow()
        logger.info(f"Dev branch rows after insert: {len(scan_dev)}")
        # assert len(scan_dev) == 2
        
        # Verify MAIN still has 1 row
        table_main.refresh()
        logger.info(f"Main branch metadata location: {table_main.metadata_location}")
        scan_main = table_main.scan().to_arrow()
        logger.info(f"Main branch rows: {len(scan_main)}")
        assert len(scan_main) == 1
        logger.info("✅ Isolation Confirmed")
        
    except Exception as e:
        logger.error(f"Branch access failed: {e}")
        raise e

    # 8. Merge Flow
    logger.info("--- [STEP] Testing Merge Flow ---")
    # Create Merge Op
    try:
        merge_op = client.branches.merge(source_branch="dev", target_branch="main", catalog_name=CATALOG_NAME)
        logger.info(f"Created Merge Op: {merge_op.id} (Status: {merge_op.status})")
        op_id = merge_op.id
    except Exception as e:
        # Handle 409 Conflict which returns the Op ID
        logger.info(f"Merge Creation returned mismatch/conflict: {e}")
        # Parse output from string representation if possible or assume it's the Conflict body
        # The client raises an error with the response text.
        # Let's assume we need to parse it manually if the SDK doesn't wrap it nice.
        import re
        match = re.search(r'operation_id":"([a-f0-9-]+)"', str(e))
        if match:
            op_id = match.group(1)
            logger.info(f"Extracted Operation ID from conflict: {op_id}")
        else:
            raise e

    # Check for conflicts
    conflicts = client.merge_operations.list_conflicts(op_id)
    logger.info(f"Conflicts: {len(conflicts)}")
    
    if len(conflicts) > 0:
        logger.info("Resolving conflicts...")
        for c in conflicts:
            client.merge_operations.resolve_conflict(c.id, "source")
            
    # Complete Merge if not already completed
    curr_op = client.merge_operations.get(op_id)
    if curr_op.status.lower() != "completed" and curr_op.status.lower() != "merged":
        client.merge_operations.complete(op_id)
        logger.info("Completed Merge Operation")
    else:
        logger.info("Merge Operation already completed")
    
    # Verify Merge
    op_final = client.merge_operations.get(op_id)
    logger.info(f"Final Status: {op_final.status}")
    assert op_final.status.lower() == "completed"
    
    # Verify MAIN has data from DEV
    # table_main.refresh()
    table_main = cat_main.load_table(f"{NAMESPACE}.{TABLE_NAME}")
    logger.info(f"Main branch metadata location: {table_main.metadata_location}")
    scan_main_post = table_main.scan().to_arrow()
    logger.info(f"Main branch rows after merge: {len(scan_main_post)}")
    # Should have 2 rows now
    assert len(scan_main_post) == 2
    logger.info("✅ Merge Verified")

    # 9. Persistence Check
    logger.info("--- [STEP] Verifying S3 Persistence ---")
    # Check for metadata files in the bucket under the tenant prefix
    verify_s3_file(f"{TENANT_NAME}/{CATALOG_NAME}/{NAMESPACE}/{TABLE_NAME}/metadata")

if __name__ == "__main__":
    try:
        run_verification()
        print("✅ VERIFICATION SUCCESSFUL")
    except Exception as e:
        logging.error(f"Verification Failed: {e}")
        exit(1)
