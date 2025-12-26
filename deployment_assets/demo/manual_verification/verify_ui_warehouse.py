
import sys
import os
import boto3
import uuid
from botocore.client import Config
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, IntegerType

# Add pypangolin to path
sys.path.append(os.path.abspath("../../../pypangolin/src"))

from pypangolin import PangolinClient

# Configuration
BASE_URL = "http://localhost:8080"
USERNAME = "ui_user_9e213df9"
PASSWORD = "password123"
TENANT_ID = "68c436bf-df33-4305-849e-3f9826523a3c"
WAREHOUSE_NAME = "ui_wh_auth"
CATALOG_NAME = f"ui_verify_cat_{uuid.uuid4().hex[:6]}"

# MinIO Config
S3_ENDPOINT = "http://127.0.0.1:9000"
S3_ACCESS_KEY = "minioadmin"
S3_SECRET_KEY = "minioadmin"
BUCKET = "warehouse"

def check_minio_persistence(prefix):
    print(f"Checking MinIO for objects with prefix: {prefix}")
    s3 = boto3.client('s3',
                      endpoint_url=S3_ENDPOINT,
                      aws_access_key_id=S3_ACCESS_KEY,
                      aws_secret_access_key=S3_SECRET_KEY,
                      config=Config(signature_version='s3v4'),
                      region_name='us-east-1')
    
    try:
        response = s3.list_objects_v2(Bucket=BUCKET, Prefix=prefix)
        if 'Contents' in response:
            for obj in response['Contents']:
                if obj['Key'].endswith('metadata.json'):
                    print(f"SUCCESS: Found metadata.json: {obj['Key']}")
                    return True
        print("FAILURE: No metadata.json found.")
        return False
    except Exception as e:
        print(f"MinIO Check Failed: {e}")
        return False

def verify_warehouse():
    print(f"--- Verifying Warehouse: {WAREHOUSE_NAME} ---")
    
    try:
        client = PangolinClient(uri=BASE_URL)
        client.login(USERNAME, PASSWORD, tenant_id=TENANT_ID)
        print("Logged in successfully.")
        
        # 1. Inspect Warehouse Configuration
        print(f"Retrieving warehouse '{WAREHOUSE_NAME}'...")
        # WarehouseClient.get wasn't implemented, so we find it in list()
        warehouses = client.warehouses.list()
        wh = next((w for w in warehouses if w.name == WAREHOUSE_NAME), None)
        
        if not wh:
            print(f"FAILURE: Warehouse '{WAREHOUSE_NAME}' not found.")
            sys.exit(1)
            
        print(f"Warehouse Storage Config: {wh.storage_config}")
        
        if wh.storage_config.get("s3.path-style-access") != "true":
            print("FAILURE: 's3.path-style-access' is NOT set to 'true'.")
            # We continue anyway to see if it fails
        else:
            print("SUCCESS: 's3.path-style-access' IS set to 'true'.")

        # 2. Create Catalog
        print(f"Creating catalog '{CATALOG_NAME}'...")
        client.catalogs.create(name=CATALOG_NAME, warehouse=WAREHOUSE_NAME)
        
        # 3. Create Table (PyIceberg)
        catalog_uri = f"{BASE_URL}/v1/{CATALOG_NAME}"
        print(f"Loading Iceberg catalog from {catalog_uri}...")
        
        iceberg_catalog = load_catalog(
            "local",
            **{
                "type": "rest",
                "uri": catalog_uri,
                "token": client.token, 
            }
        )
        
        ns = "ui_verify_ns"
        try: iceberg_catalog.create_namespace(ns)
        except: pass
        
        table_name = f"{ns}.table_verify"
        schema = Schema(NestedField(1, "id", IntegerType(), required=True))
        
        print(f"Creating table {table_name}...")
        iceberg_catalog.create_table(table_name, schema=schema)
        
        # 4. Verify Persistence
        expected_prefix = f"{CATALOG_NAME}/{ns}/"
        if check_minio_persistence(expected_prefix):
            print("VERIFICATION PASSED")
        else:
            print("VERIFICATION FAILED: Metadata not persisted.")
            sys.exit(1)

    except Exception as e:
        print(f"Verification Failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

if __name__ == "__main__":
    verify_warehouse()
