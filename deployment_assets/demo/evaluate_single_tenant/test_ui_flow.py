import sys
import os
import time
import uuid
import boto3
from botocore.client import Config
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, IntegerType

# Add pypangolin to path
sys.path.append(os.path.abspath("../../../pypangolin/src"))

from pypangolin import PangolinClient

# MinIO Config
S3_ENDPOINT = "http://127.0.0.1:9000"
S3_ACCESS_KEY = "minioadmin"
S3_SECRET_KEY = "minioadmin"
BUCKET = "warehouse"

# This logic needs to be dynamic or handle both names
def get_warehouse_name(auth_mode):
    return "ui_wh_auth" if auth_mode else "ui_wh"

CATALOG_NAME = "ui_cat"

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

def test_flow(auth_mode=False):
    print(f"--- Testing UI Flow (Auth: {auth_mode}) ---")
    
    client = PangolinClient(uri="http://localhost:8080")
    
    if auth_mode:
        # In Auth mode, we need to login to interact with the API
        # We assume the Browser Agent has already created the warehouse as Tenant Admin
        # But for this script to run PyIceberg, IT needs a token too.
        # We'll login as the same user the Browser Agent supposedly used (e.g. admin or created user)
        # OR just login as root to inspect things if possible, but PyIceberg needs valid token.
        # Let's assume the Browser used 'admin' (Root) to create tenant/user and then logged in as that user?
        # Simpler: Browser Agent logs in as 'admin' (Root) -> Creates Tenant -> Creates User -> Logs in as User -> Creates WH.
        # Wait, Root can't create WH.
        # So Browser Agent MUST: Login Root -> Create Tenant -> Create User -> Logout -> Login User -> Create WH.
        # This is complex for the browser agent.
        # 
        # ALTERNATIVE: Use the `test_pypangolin_live.py` (Auth) logic to provision the user FIRST via API,
        # THEN have the Browser Agent login as that user and create the WH.
        # YES, let's do that. The script `setup_auth_user.py` can provision the user.
        print("Assuming User 'ui_user' exists (provisioned by setup script). Logging in...")
        try:
             # We need to know the tenant ID. We'll find the tenant 'ui_tenant'.
             # Admin login to find tenant
             client.login("admin", "password")
             tenants = client.tenants.list()
             tenant = next((t for t in tenants if t.name == "ui_tenant"), None)
             if not tenant:
                 print("FAILURE: ui_tenant not found. Did setup run?")
                 sys.exit(1)
             
             client.login("ui_user", "password123", tenant_id=tenant.id)
             print(f"Logged in as ui_user. Token: {client.token[:10]}...")
        except Exception as e:
             print(f"Login failed: {e}")
             sys.exit(1)
        
        tenant_id = tenant.id
    else:
        tenant_id = "00000000-0000-0000-0000-000000000000"

    warehouse_name = get_warehouse_name(auth_mode)

    # 1. Verify Warehouse Exists (Created by UI)
    print(f"Verifying warehouse '{warehouse_name}' exists...")
    try:
        warehouses = client.warehouses.list()
        wh = next((w for w in warehouses if w.name == warehouse_name), None)
        
        if not wh:
             print(f"FAILURE: Warehouse '{warehouse_name}' not found in list.")
             sys.exit(1)
             
        print("Warehouse found.")
        # Check config
        print(f"Config: {wh.storage_config}")
        if wh.storage_config.get("s3.path-style-access") == "true":
             print("SUCCESS: s3.path-style-access is 'true'")
        else:
             print("FAILURE: s3.path-style-access is missing or false")
             sys.exit(1)
             
    except Exception as e:
        print(f"Warehouse check failed: {e}")
        sys.exit(1)

    # 2. Create Catalog (using the UI-created WH)
    try:
        print(f"Creating catalog '{CATALOG_NAME}'...")
        # Clean up if exists
        try: client.catalogs.delete(CATALOG_NAME)
        except: pass
        
        client.catalogs.create(name=CATALOG_NAME, warehouse=warehouse_name)
    except Exception as e:
        print(f"Catalog create failed: {e}")
        sys.exit(1)

    # 3. PyIceberg Table Create
    print("Using PyIceberg to create table...")
    catalog_uri = f"http://localhost:8080/v1/{CATALOG_NAME}"
    
    rest_config = {
        "type": "rest",
        "uri": catalog_uri,
    }
    
    if auth_mode:
        rest_config["token"] = client.token
    else:
        rest_config["header.X-Pangolin-Tenant"] = tenant_id

    try:
        iceberg_catalog = load_catalog("local", **rest_config)
        
        ns = "ui_ns"
        try: iceberg_catalog.create_namespace(ns)
        except: pass
        
        unique = str(uuid.uuid4())[:6]
        table_name = f"{ns}.ui_table_{unique}"
        schema = Schema(NestedField(1, "val", StringType(), required=True))
        
        print(f"Creating table {table_name}...")
        iceberg_catalog.create_table(table_name, schema=schema)
        
        # 4. Verify MinIO Persistence
        expected_prefix = f"{CATALOG_NAME}/{ns}/"
        if check_minio_persistence(expected_prefix):
            print("UI Flow Verification PASSED")
        else:
            print("UI Flow Verification FAILED (Persistence)")
            sys.exit(1)
            
    except Exception as e:
        print(f"PyIceberg/Persistence failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

if __name__ == "__main__":
    auth = len(sys.argv) > 1 and sys.argv[1] == "auth"
    test_flow(auth)
