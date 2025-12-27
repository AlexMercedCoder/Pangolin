import sys
import os
import time
import json
import logging

# Add pypangolin to path
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "../pypangolin/src")))

from pypangolin.client import PangolinClient
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, IntegerType, StringType
import s3fs

# Setup Config
PANGOLIN_URL = "http://localhost:8080"
MINIO_ENDPOINT = "http://localhost:9000"
AWS_ACCESS_KEY_ID = "minioadmin"
AWS_SECRET_ACCESS_KEY = "minioadmin"

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("verify_pypangolin")

def verify_step(name):
    logger.info(f"--- [STEP] {name} ---")

def main():
    try:
        # 1. Admin Login & System Check
        verify_step("Initializing PyPangolin Client (Admin)")
        # Start with known root credentials
        client = PangolinClient(PANGOLIN_URL, username="tenant_admin", password="password123", tenant_id="00000000-0000-0000-0000-000000000000")
        
        # Verify System Settings (Parity Check)
        settings = client.system.get_settings()
        logger.info(f"System Settings retrieved: {settings.keys()}")
        assert "allow_public_signup" in settings
        
        # 2. Service User Creation (Parity Check)
        verify_step("Creating Service User via PyPangolin")
        su_name = f"su-verify-{int(time.time())}"
        service_user = client.service_users.create(name=su_name, role="tenant-admin")
        logger.info(f"Created Service User: {service_user.name} ({service_user.id})")
        
        # Rotate Key to get credentials
        su_with_key = client.service_users.rotate_key(service_user.id)
        api_key = su_with_key.api_key
        logger.info(f"Rotated Key. Got API Key: {api_key[:5]}...")
        
        # 3. PyIceberg setup with OAuth
        verify_step("Initializing PyIceberg with OAuth")
        catalog_name = "cli_catalog" # Assuming this exists from previous test, or we create it
        # Ensure catalog exists
        try:
            client.catalogs.get(catalog_name)
        except:
             # Create backend warehouse if needed? Let's assume environment is running from previous test
             # or create it quickly
             try:
                client.warehouses.create_s3("minio_wh", "test-bucket", endpoint=MINIO_ENDPOINT, 
                                          access_key=AWS_ACCESS_KEY_ID, secret_key=AWS_SECRET_ACCESS_KEY)
                client.catalogs.create(catalog_name, "minio_wh")
             except Exception as e:
                logger.warning(f"Catalog setup warning (might exist): {e}")

        catalog = load_catalog(
            "pangolin",
            **{
                "uri": f"{PANGOLIN_URL}/v1/{catalog_name}/",
                "type": "rest",
                "credential": f"{service_user.id}:{api_key}",
                "oauth2-server-uri": f"{PANGOLIN_URL}/v1/{catalog_name}/v1/oauth/tokens",
                "scope": "catalog",
                "s3.endpoint": MINIO_ENDPOINT,
                "s3.access-key-id": AWS_ACCESS_KEY_ID,
                "s3.secret-access-key": AWS_SECRET_ACCESS_KEY,
            }
        )
        logger.info("PyIceberg Catalog initialized")

        # 4. Iceberg Operations
        verify_step("Performing Iceberg Operations")
        ns = "pypangolin_verify"
        tbl = "metadata_check"
        
        try:
            catalog.create_namespace(ns)
        except:
            pass # Exists
            
        schema = Schema(NestedField(1, "id", IntegerType(), required=True), NestedField(2, "data", StringType()))
        try:
            catalog.drop_table(f"{ns}.{tbl}")
        except:
            pass

        table = catalog.create_table(f"{ns}.{tbl}", schema=schema)
        logger.info(f"Table created: {table}")
        
        # 5. MinIO Metadata Verification
        verify_step("Verifying MinIO Metadata Persistence")
        fs = s3fs.S3FileSystem(
            client_kwargs={'endpoint_url': MINIO_ENDPOINT},
            key=AWS_ACCESS_KEY_ID,
            secret=AWS_SECRET_ACCESS_KEY
        )
        
        # Location structure: test-bucket/{catalog}/{namespace}/{table}/metadata/....metadata.json
        # Note: 'cli_catalog' might be part of prefix depending on warehouse config.
        # Let's inspect the table location from PyIceberg
        location = table.location() # It IS a method
        logger.info(f"Table Location: {location}")
        
        # Strip s3://
        path = location.replace("s3://", "")
        # Look for metadata folder
        metadata_path = f"{path}/metadata"
        files = fs.ls(metadata_path)
        metadata_files = [f for f in files if f.endswith(".metadata.json")]
        
        logger.info(f"Found Metadata Files: {metadata_files}")
        assert len(metadata_files) > 0, "No metadata.json found in MinIO!"
        
        # 6. Audit Log Verification (Parity Check)
        verify_step("Verifying Audit Logs via PyPangolin")
        # Wait a moment for logs to flush if async
        time.sleep(1)
        
        # List events filtered by our service user
        # List events filtered by our service user
        events = client.audit.list_events(user_id=service_user.id, limit=5)
        logger.info(f"Found {len(events)} audit events for Service User")
        
        for event in events:
            logger.info(f"Event: {event.action} on {event.resource_type} ({event.result})")
            
        # We expect at least 'create_table'
        actions = [e.action for e in events]
        assert "create_table" in actions or "create_namespace" in actions, "Expected Iceberg actions in audit log"
        
        logger.info("✅ VERIFICATION SUCCESSFUL")

    except Exception as e:
        logger.error(f"❌ VERIFICATION FAILED: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

if __name__ == "__main__":
    main()
