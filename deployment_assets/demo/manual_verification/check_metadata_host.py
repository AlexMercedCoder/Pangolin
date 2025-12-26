from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, IntegerType
import time
import sys

def main():
    print("HOST: Checking for metadata.json persistence with HOST API...")
    
    # Connecting to localhost:8080 (which is now the cargo run process)
    catalog = load_catalog(
        "pangolin",
        **{
            "type": "rest",
            "uri": "http://localhost:8080/v1/demo",
            "header.X-Pangolin-Tenant": "00000000-0000-0000-0000-000000000000",
            # Explicit S3 config for Host
            "s3.endpoint": "http://localhost:9000",
            "s3.access-key-id": "minioadmin",
            "s3.secret-access-key": "minioadmin",
            "s3.region": "us-east-1",
            "s3.path-style-access": "true",
        }
    )

    # We need to recreate the catalog/warehouse because MemoryStore restarted
    print("Recreating Warehouse/Catalog in MemoryStore...")
    import requests
    try:
        requests.post("http://localhost:8080/api/v1/warehouses", json={
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
              "endpoint": "http://127.0.0.1:9000",
              "access_key_id": "minioadmin",
              "secret_access_key": "minioadmin",
              "s3.path-style-access": "true"
            }
        }, headers={"Content-Type": "application/json"})
        
        requests.post("http://localhost:8080/api/v1/catalogs", json={
            "name": "demo",
            "warehouse_name": "demo_warehouse",
            "storage_location": "s3://warehouse/demo_catalog"
        }, headers={"Content-Type": "application/json"})
    except Exception as e:
        print(f"Setup warning: {e}")

    ns_name = f"meta_check_host"
    try:
        catalog.create_namespace(ns_name)
    except:
        pass

    schema = Schema(
        NestedField(1, "id", IntegerType(), required=True),
    )

    table_name = f"{ns_name}.meta_table"
    print(f"Creating table '{table_name}'...")
    try:
        table = catalog.create_table(table_name, schema=schema)
        print(f"Table Location: {table.location()}")
        print(f"Metadata Location: {table.metadata_location}")
    except Exception as e:
        print(f"Failed to create table: {e}")

if __name__ == "__main__":
    main()
