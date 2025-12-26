import requests
import time
import os
import sys
import boto3
import sqlite3
import json
import uuid
import datetime

# Configuration
API_URL = "http://localhost:8080"
DB_PATH = "pangolin_test.db"
MINIO_ENDPOINT = "http://localhost:9000"
MINIO_ACCESS_KEY = "minioadmin"
MINIO_SECRET_KEY = "minioadmin"
BUCKET_NAME = "pangolin-test"

# Setup S3 Client
s3 = boto3.client('s3',
    endpoint_url=MINIO_ENDPOINT,
    aws_access_key_id=MINIO_ACCESS_KEY,
    aws_secret_access_key=MINIO_SECRET_KEY,
    region_name='us-east-1'
)

def check_response(resp, context):
    if resp.status_code >= 400:
        print(f"❌ {context} Failed: {resp.status_code} - {resp.text}")
        sys.exit(1)
    print(f"✅ {context} Success: {resp.status_code}")
    return resp.json() if resp.content else None

def wait_for_api():
    print("Waiting for API to be ready...")
    for _ in range(30):
        try:
            requests.get(f"{API_URL}/api/v1/health")
            print("✅ API is ready")
            return
        except requests.exceptions.ConnectionError:
            time.sleep(1)
    print("❌ API failed to start")
    sys.exit(1)

def main():
    # Ensure S3 Bucket Exists
    try:
        s3.create_bucket(Bucket=BUCKET_NAME)
        print(f"✅ Bucket '{BUCKET_NAME}' created or exists")
    except Exception as e:
        print(f"⚠️  Bucket creation warning: {e}")

    wait_for_api()

    # 1. Create Tenant
    tenant_id = str(uuid.uuid4())
    print(f"\n--- Creating Tenant {tenant_id} ---")
    requests.post(f"{API_URL}/api/v1/tenants", json={
        "id": tenant_id,
        "name": "sqlite_test_tenant",
        "properties": {"test": "true"}
    })
    
    # 2. Create Warehouse
    warehouse_name = "test_warehouse"
    print(f"\n--- Creating Warehouse {warehouse_name} ---")
    wh_payload = {
        "name": warehouse_name,
        "use_sts": False,
        "storage_config": {
            "type": "s3",
            "bucket": BUCKET_NAME,
            "region": "us-east-1",
            "endpoint": MINIO_ENDPOINT,
            "access_key_id": MINIO_ACCESS_KEY,
            "secret_access_key": MINIO_SECRET_KEY
        },
        "vending_strategy": None
    }
    check_response(requests.post(f"{API_URL}/api/v1/warehouses", json=wh_payload, headers={"X-Tenant-ID": tenant_id}), "Create Warehouse")

    # 3. Create Catalog
    catalog_name = "test_catalog"
    print(f"\n--- Creating Catalog {catalog_name} ---")
    cat_payload = {
        "name": catalog_name,
        "catalog_type": "Local",
        "warehouse_name": warehouse_name,
        "storage_location": f"s3://{BUCKET_NAME}/warehouse"
    }
    check_response(requests.post(f"{API_URL}/api/v1/catalogs", json=cat_payload, headers={"X-Tenant-ID": tenant_id}), "Create Catalog")

    # 4. Create Namespace
    namespace = "db"
    print(f"\n--- Creating Namespace {namespace} ---")
    ns_payload = {"namespace": [namespace]}
    # Note: Using standard Iceberg REST endpoint /v1/{catalog_name}/namespaces
    check_response(requests.post(f"{API_URL}/v1/{catalog_name}/namespaces", json=ns_payload, headers={"X-Tenant-ID": tenant_id}), "Create Namespace")

    # 5. Create Iceberg Table (Asset) using Iceberg REST API simulation or internal API
    table_name = "test_table"
    print(f"\n--- Creating Iceberg Table {table_name} ---")
    
    # Minimal Iceberg Table Schema
    schema = {
        "type": "struct",
        "schema-id": 0,
        "fields": [{"id": 1, "name": "id", "required": True, "type": "int"}]
    }
    
    iceberg_payload = {
        "name": table_name,
        "schema": schema,
        "location": f"s3://{BUCKET_NAME}/warehouse/db/{table_name}",
        "properties": {}
    }
    
    # Using the Iceberg REST endpoint
    url = f"{API_URL}/v1/{catalog_name}/namespaces/{namespace}/tables"
    resp = requests.post(url, json=iceberg_payload, headers={"X-Tenant-ID": tenant_id})
    check_response(resp, "Create Iceberg Table")

    # 6. Verify Minio Persistence
    print("\n--- Verifying Minio Persistence ---")
    # Metadata location pattern: warehouse/db/test_table/metadata/00000-....metadata.json
    try:
        paginator = s3.get_paginator('list_objects_v2')
        pages = paginator.paginate(Bucket=BUCKET_NAME, Prefix=f"warehouse/{namespace}/{table_name}/metadata")
        found = False
        for page in pages:
            if 'Contents' in page:
                for obj in page['Contents']:
                    if obj['Key'].endswith('.metadata.json'):
                        print(f"✅ Found metadata file: {obj['Key']}")
                        found = True
                        break
        if not found:
            print("❌ No metadata.json found in Minio!")
            sys.exit(1)
    except Exception as e:
        print(f"❌ Minio verification failed: {e}")
        sys.exit(1)

    # 7. Verify SQLite Persistence
    print("\n--- Verifying SQLite Persistence ---")
    try:
        conn = sqlite3.connect(DB_PATH)
        cursor = conn.cursor()
        
        # Check Asset
        cursor.execute("SELECT name, asset_type, metadata_location FROM assets WHERE name=?", (table_name,))
        row = cursor.fetchone()
        if row:
            print(f"✅ Found asset in SQLite: Name={row[0]}, Type={row[1]}, Loc={row[2]}")
        else:
            print("❌ Asset not found in SQLite!")
            sys.exit(1)
            
        # Check Tenant
        cursor.execute("SELECT name FROM tenants WHERE id=?", (tenant_id,))
        row = cursor.fetchone()
        if row:
            print(f"✅ Found tenant in SQLite: Name={row[0]}")
        else:
             print("❌ Tenant not found in SQLite!")
             sys.exit(1)
             
        conn.close()
    except Exception as e:
        print(f"❌ SQLite verification failed: {e}")
        sys.exit(1)

    print("\n🎉 Live Test Completed Successfully!")

if __name__ == "__main__":
    main()
