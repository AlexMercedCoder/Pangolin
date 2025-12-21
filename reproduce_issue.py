
import requests
import boto3
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, IntegerType
import logging
import time

# Logging
logging.basicConfig(level=logging.INFO)

API_URL = "http://localhost:8080"
S3_ENDPOINT = "http://localhost:9000"
AWS_ACCESS_KEY = "minioadmin"
AWS_SECRET_KEY = "minioadmin"
BUCKET_NAME = "test-bucket"


def check_response(resp, msg):
    if resp.status_code >= 400:
        print(f"FAILED {msg}: {resp.status_code} {resp.text}")
        # exit(1) # Don't exit yet, see what else fails
    else:
        print(f"SUCCESS {msg}")

# 1. Setup Resources
def setup_resources():
    print("Setting up resources...")
    
    # Check if API is up
    for i in range(10):
        try:
            requests.get(f"{API_URL}/health")
            break
        except:
            print(f"Waiting for API... {i}")
            time.sleep(1)
            
    # Create Tenant (Skipping creation, using default)
    t_id = "00000000-0000-0000-0000-000000000000"
    # resp = requests.post(f"{API_URL}/api/v1/tenants", json={"name": "demo_tenant", "id": t_id})
    # check_response(resp, "Create Tenant")
    
    # Create Warehouse
    warehouse_payload = {
        "name": "demo_warehouse",
        "tenant_id": t_id,
        "storage_config": {
            "s3.bucket": BUCKET_NAME,
            "s3.endpoint": S3_ENDPOINT,
            "s3.access-key-id": AWS_ACCESS_KEY,
            "s3.secret-access-key": AWS_SECRET_KEY,
            "s3.region": "us-east-1"
        },
        "use_sts": False
    }
    # Warehouse route /api/v1/warehouses (implied)
    resp = requests.post(f"{API_URL}/api/v1/warehouses", json=warehouse_payload)
    check_response(resp, "Create Warehouse")
    
    # Create Catalog
    catalog_payload = {
        "name": "demo",
        "tenant_id": t_id, # Need to pass tenant_id
        "catalog_type": "Local",
        "warehouse_name": "demo_warehouse",
        "storage_location": "s3://test-bucket/demo"
    }
    resp = requests.post(f"{API_URL}/api/v1/catalogs", json=catalog_payload)
    check_response(resp, "Create Catalog")
    
    # Ensure Bucket Exists
    s3 = boto3.client('s3', endpoint_url=S3_ENDPOINT, aws_access_key_id=AWS_ACCESS_KEY, aws_secret_access_key=AWS_SECRET_KEY)
    try:
        s3.create_bucket(Bucket=BUCKET_NAME)
    except:
        pass # Bucket might exist

    return t_id

# 2. Run Iceberg
def run_iceberg():
    print("Running Iceberg operations...")
    catalog = load_catalog(
        "pangolin",
        **{
            "type": "rest",
            "uri": f"{API_URL}/v1/demo", 
            "header.X-Iceberg-Access-Delegation": "vended-credentials",
            "header.X-Pangolin-Tenant": "00000000-0000-0000-0000-000000000000",
            "s3.endpoint": S3_ENDPOINT,
            "s3.access-key-id": AWS_ACCESS_KEY,
            "s3.secret-access-key": AWS_SECRET_KEY
        }
    )

    try:
        catalog.create_namespace("test_ns")
    except Exception as e:
        print(f"NS might exist: {e}")

    schema = Schema(
        NestedField(1, "id", IntegerType(), required=True),
        NestedField(2, "data", StringType(), required=False),
    )

    try:
        table = catalog.create_table("test_ns.test_table", schema=schema)
        print(f"Created table: {table}")
        
        # Verify Metadata File
        s3 = boto3.client('s3', endpoint_url=S3_ENDPOINT, aws_access_key_id=AWS_ACCESS_KEY, aws_secret_access_key=AWS_SECRET_KEY)
        
        # List objects in bucket
        objs = s3.list_objects_v2(Bucket=BUCKET_NAME, Prefix="demo/test_ns/test_table/metadata/")
        if 'Contents' in objs:
            print("SUCCESS: Metadata files found in S3:")
            for obj in objs['Contents']:
                print(f" - {obj['Key']}")
        else:
            print("FAILURE: No metadata files found in S3!")
            exit(1)
            
    except Exception as e:
        print(f"Error creating table: {e}")
        # Check if table exists
        try:
             t = catalog.load_table("test_ns.test_table")
             print("Table exists (loaded successfully). checking S3...")
             # Verify Metadata File
             s3 = boto3.client('s3', endpoint_url=S3_ENDPOINT, aws_access_key_id=AWS_ACCESS_KEY, aws_secret_access_key=AWS_SECRET_KEY)
             objs = s3.list_objects_v2(Bucket=BUCKET_NAME, Prefix="demo/test_ns/test_table/metadata/")
             if 'Contents' in objs:
                 print("SUCCESS: Metadata files found in S3 (on load):")
                 for obj in objs['Contents']:
                     print(f" - {obj['Key']}")
             else:
                 print("FAILURE: No metadata files found in S3 (on load)!")
                 exit(1)
        except:
             print("Check failed.")
             raise e

setup_resources()
run_iceberg()
