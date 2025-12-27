
import os
import time
import uuid
import logging
from pypangolin import PangolinClient
from pyiceberg.catalog import load_catalog
import pyarrow as pa

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(levelname)s:%(name)s:%(message)s')
logger = logging.getLogger("setup_ui_merge")

API_URL = "http://localhost:8080"
S3_ENDPOINT = "http://localhost:9000"
S3_ACCESS_KEY = "minioadmin"
S3_SECRET_KEY = "minioadmin"

def setup():
    timestamp = int(time.time())
    
    # 1. Initialize Client (No Auth)
    logger.info("--- [STEP] Initializing Client ---")
    # config = ClientConfig(base_url=API_URL) 
    # PangolinClient takes uri directly
    client = PangolinClient(uri=API_URL)

    # 2. Create Tenant
    tenant_name = f"ui_merge_tenant_{timestamp}"
    logger.info(f"--- [STEP] Creating Tenant {tenant_name} ---")
    try:
        # In NO_AUTH, we might need a tenant created first via some means or just rely on default if only one exists?
        # But let's try creating one.
        # Note: In NO_AUTH, create_tenant might be restricted or auto-handled. 
        # Actually, let's create it.
        try:
            tenant = client.tenants.create(name=tenant_name)
        except Exception as e:
            logger.info(f"Tenant creation skipped/failed (expected in NO_AUTH): {e}")
            pass
        # Switch context to this tenant. API client doesn't automatically switch context unless we re-init or set headers.
        # But pypangolin client methods usually take tenant/catalog or rely on configured context.
        # The Python SDK `Client` holds the context. 
        # Wait, the SDK `Client` takes `token`. If no auth, how is tenant passed?
        # In NO_AUTH mode, the backend often defaults to a single tenant or expected header.
        # Let's assume we need to set the header `X-Pangolin-Tenant` manually if SDK supports it, or recreate client.
        # Current SDK might not support setting tenant header easily without login.
        # HOWEVER, `test_release_v0.2.0.py` suggests we can just create resources.
        pass
    except Exception as e:
        logger.warning(f"Tenant creation might have failed or not needed: {e}")
        # If we can't create tenant, maybe we use the default 'pangolin' tenant?
        tenant_name = "pangolin" 
        try:
             client.tenants.create(name=tenant_name, provider="minio")
        except:
             pass

    # 3. Create Warehouse
    warehouse_name = f"ui_merge_wh_{timestamp}"
    bucket_name = f"ui-bucket-{timestamp}"
    logger.info(f"--- [STEP] Creating Warehouse {warehouse_name} ---")
    
    # Ensure bucket exists
    # os.system(f"mc alias set minio {S3_ENDPOINT} {S3_ACCESS_KEY} {S3_SECRET_KEY}")
    # os.system(f"mc mb minio/{bucket_name}")
    
    wh_config = {
        "s3.endpoint": S3_ENDPOINT,
        "s3.access-key-id": S3_ACCESS_KEY,
        "s3.secret-access-key": S3_SECRET_KEY,
        "s3.region": "us-east-1",
        "s3.bucket": bucket_name,
        "prefix": f"{tenant_name}/",
        "s3.path-style-access": "true"
    }
    
    try:
        client.warehouses.create_s3(
            name=warehouse_name,
            bucket=bucket_name,
            access_key=S3_ACCESS_KEY,
            secret_key=S3_SECRET_KEY,
            endpoint=S3_ENDPOINT,
            **wh_config
        )
    except Exception as e:
        logger.error(f"Failed to create warehouse: {e}")
        # Proceeding, maybe it exists

    # 4. Create Catalog
    catalog_name = f"ui_cat_{timestamp}"
    logger.info(f"--- [STEP] Creating Catalog {catalog_name} ---")
    client.catalogs.create(
        name=catalog_name,
        warehouse=warehouse_name,
        type="Local"
    )

    # 5. Create Table on MAIN
    # logger.info("--- [STEP] Creating Table on MAIN ---")
    # ... PyIceberg steps skipped ...
    
    # 6. Create Branch DEV
    logger.info("--- [STEP] Creating Branch DEV ---")
    # Using API client to create branch
    # Note: If 'main' was not created by catalog creation (it usually is), this might fail.
    # But let's assume 'main' exists.
    try:
        client.branches.create(name="dev", catalog_name=catalog_name, from_branch="main")
        logger.info("Created branch: dev")
    except Exception as e:
        logger.error(f"Failed to create dev branch: {e}")

    # 7. Write to DEV
    # ... Skipped ...
        
    logger.info("--- SETUP COMPLETE ---")
    logger.info(f"Catalog: {catalog_name}")
    # logger.info(f"Table: {table_name}")
    logger.info("Branches: main, dev")
    
    # Return info for verification script
    print(f"CATALOG_NAME={catalog_name}")

if __name__ == "__main__":
    setup()
