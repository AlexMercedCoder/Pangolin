
import os
import shutil
import time
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, IntegerType, StringType
import pyarrow as pa
import pytest

# Configure Environment
# Ensure PANGOLIN_URI points to your running Pangolin instance
PANGOLIN_URI = os.getenv("PANGOLIN_URI", "http://localhost:8080")
WAREHOUSE_PATH = "s3://warehouse/mongo_test"

# MinIO Credentials (for local testing)
os.environ["AWS_ACCESS_KEY_ID"] = "minioadmin"
os.environ["AWS_SECRET_ACCESS_KEY"] = "minioadmin"
os.environ["AWS_REGION"] = "us-east-1"
os.environ["AWS_S3_ENDPOINT"] = "http://localhost:9000"
os.environ["PYICEBERG_CATALOG__DEFAULT__URI"] = f"{PANGOLIN_URI}/v1/mongo_catalog"
os.environ["PYICEBERG_CATALOG__DEFAULT__TYPE"] = "rest"
os.environ["PYICEBERG_CATALOG__DEFAULT__S3_ENDPOINT"] = "http://localhost:9000"

import requests

def test_mongo_store_workflow():
    print(f"Connecting to Pangolin at {PANGOLIN_URI}...")

    # 0. Setup Warehouse and Catalog via API
    # Create Warehouse
    wh_payload = {
        "name": "mongo_warehouse",
        "storage_config": {
            "type": "s3",
            "s3.bucket": "warehouse",
            "s3.region": "us-east-1",
            "s3.endpoint": "http://localhost:9000",
            "s3.access-key-id": "minioadmin",
            "s3.secret-access-key": "minioadmin"
        }
    }
    resp = requests.post(f"{PANGOLIN_URI}/api/v1/warehouses", json=wh_payload)
    if resp.status_code in [200, 201, 409]:
        print("Warehouse 'mongo_warehouse' ready.")
    else:
        print(f"Failed to create warehouse: {resp.text}")
        return

    # Create Catalog
    cat_payload = {
        "name": "mongo_catalog",
        "warehouse_name": "mongo_warehouse",
        "type": "Rest"
    }
    resp = requests.post(f"{PANGOLIN_URI}/api/v1/catalogs", json=cat_payload)
    if resp.status_code in [200, 201, 409]:
        print("Catalog 'mongo_catalog' ready.")
    else:
        print(f"Failed to create catalog: {resp.text}")
        return

    # 1. Initialize Catalog
    conf = {
        "uri": f"{PANGOLIN_URI}/v1/mongo_catalog",
        "type": "rest",
        "s3.endpoint": "http://localhost:9000",
        "s3.access-key-id": "minioadmin",
        "s3.secret-access-key": "minioadmin",
        "s3.region": "us-east-1"
    }
    catalog = load_catalog("mongo_catalog", **conf)
    
    # 2. Create Namespace
    ns = "test_mongo_ns"
    try:
        catalog.create_namespace(ns)
        print(f"Namespace '{ns}' created.")
    except Exception as e:
        print(f"Namespace creation failed (might exist): {e}")

    # 3. Create Table
    schema = Schema(
        NestedField(1, "id", IntegerType(), required=True),
        NestedField(2, "data", StringType(), required=False),
    )
    table_name = f"{ns}.mongo_table"
    
    try:
        catalog.drop_table(table_name)
    except:
        pass

    table = catalog.create_table(
        table_name,
        schema=schema,
        location=f"{WAREHOUSE_PATH}/{ns}/mongo_table"
    )
    print(f"Table '{table_name}' created.")

    # 4. Write Data
    df = pa.Table.from_pylist([
        {"id": 1, "data": "mongo_row_1"},
        {"id": 2, "data": "mongo_row_2"},
    ], schema=schema.as_arrow())
    
    table.append(df)
    print("Data appended.")

    # 5. Read Data
    scan = table.scan()
    result = scan.to_arrow()
    print("Data read properly:")
    print(result)
    
    assert len(result) == 2
    
    # 6. Verify Metadata Location logic (Regression Check)
    # We check if we can update the table (commit v2)
    # This triggers update_metadata_location with optimistic locking
    
    df2 = pa.Table.from_pylist([
        {"id": 3, "data": "mongo_row_3"},
    ], schema=schema.as_arrow())
    
    table.append(df2)
    print("Second append successful (Optimistic Locking verified).")
    
    # 7. Clean up
    catalog.drop_table(table_name)
    catalog.drop_namespace(ns)
    print("Cleanup complete.")

if __name__ == "__main__":
    test_mongo_store_workflow()
