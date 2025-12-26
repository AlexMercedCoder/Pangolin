import requests
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, IntegerType
import time
import os

def main():
    print("Setting up Pangolin in Docker...")
    time.sleep(2) # Give services a moment
    
    # 1. Create Warehouse
    # We deliberately include s3.path-style-access to check if API honors it (it likely doesn't)
    wh_payload = {
        "name": "demo_warehouse",
        "vending_strategy": {
            "AwsStatic": {
                "access_key_id": "minioadmin",
                "secret_access_key": "minioadmin"
            }
        },
        "storage_config": {
            "type": "s3",
            "bucket": "warehouse",
            "region": "us-east-1",
            "endpoint": "http://minio:9000", # Internal Docker URL
            "access_key_id": "minioadmin",
            "secret_access_key": "minioadmin",
            "s3.path-style-access": "true" 
        }
    }
    
    try:
        print("Creating Warehouse 'demo_warehouse'...")
        r = requests.post("http://pangolin-api:8080/api/v1/warehouses", json=wh_payload, headers={"Content-Type": "application/json"})
        if r.status_code in [200, 201]:
            print("Warehouse created.")
        else:
            print(f"Warehouse creation status: {r.status_code} {r.text}")
    except Exception as e:
        print(f"Warehouse creation error: {e}")

    # 2. Create Catalog
    cat_payload = {
        "name": "demo",
        "warehouse_name": "demo_warehouse",
        "storage_location": "s3://warehouse/demo_catalog"
    }
    try:
        print("Creating Catalog 'demo'...")
        r = requests.post("http://pangolin-api:8080/api/v1/catalogs", json=cat_payload, headers={"Content-Type": "application/json"})
        if r.status_code in [200, 201]:
            print("Catalog created.")
        else:
            print(f"Catalog creation status: {r.status_code} {r.text}")
    except Exception as e:
        print(f"Catalog creation error: {e}")

    # 3. Use PyIceberg to create table
    print("Connecting to catalog...")
    catalog = load_catalog(
        "pangolin",
        **{
            "type": "rest",
            "uri": "http://pangolin-api:8080/v1/demo",
            "header.X-Pangolin-Tenant": "00000000-0000-0000-0000-000000000000",
            # We don't need credentials for create_table usually if server handles metadata
            # But we might need them if we wrote data. For now, just create table.
        }
    )

    ns = "setup_test_ns"
    try:
        catalog.create_namespace(ns)
        print(f"Created namespace '{ns}'")
    except Exception:
        print(f"Namespace '{ns}' exists")
    
    schema = Schema(NestedField(1, "id", IntegerType(), required=True))
    
    tbl = f"{ns}.test_table"
    print(f"Creating table '{tbl}'...")
    try:
        table = catalog.create_table(tbl, schema=schema)
        print(f"Table created: {table}")
        print(f"Metadata Location: {table.metadata_location}")
    except Exception as e:
        print(f"Failed to create table: {e}")

if __name__ == "__main__":
    main()
