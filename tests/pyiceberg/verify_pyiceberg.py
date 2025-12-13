
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, IntegerType, StringType
import time
import os

# Configuration
WAREHOUSE_NAME = "main_warehouse"
CATALOG_URI = "http://localhost:8080"
WAREHOUSE_PATH = "s3://pangolin/data"
TENANT_ID = "00000000-0000-0000-0000-000000000001"

print(f"Connecting to Pangolin Catalog at {CATALOG_URI}...")
print(f"Using tenant ID: {TENANT_ID}")
print(f"Using warehouse: {WAREHOUSE_NAME}")

# 1. Initialize Catalog
# According to PyIceberg docs, headers are added via header.* prefix
# and the warehouse parameter tells PyIceberg which prefix to use in URLs
catalog = load_catalog(
    "pangolin",
    **{
        "type": "rest",
        "uri": CATALOG_URI,
        "warehouse": WAREHOUSE_NAME,
        "header.X-Pangolin-Tenant": TENANT_ID,
        "s3.endpoint": "http://localhost:9000",
        "s3.access-key-id": "minioadmin",
        "s3.secret-access-key": "minioadmin",
        "py-io-impl": "pyiceberg.io.pyarrow.PyArrowFileIO",
        "s3.region": "us-east-1",
    }
)

print(f"Catalog initialized: {catalog}")
print(f"Catalog properties: {catalog.properties}")

# 2. Create Namespace
print("\\nCreating namespace 'test_ns'...")
try:
    catalog.create_namespace("test_ns")
    print("Namespace 'test_ns' created.")
except Exception as e:
    print(f"Namespace creation failed (or exists): {e}")

# 3. Create Table
schema = Schema(
    NestedField(1, "id", IntegerType(), required=True),
    NestedField(2, "data", StringType(), required=False),
)

table_name = "test_ns.my_table"
print(f"\\nCreating table '{table_name}'...")
try:
    table = catalog.create_table(
        identifier=table_name,
        schema=schema,
        location=f"{WAREHOUSE_PATH}/test_ns/my_table"
    )
    print(f"Table '{table_name}' created: {table}")
except Exception as e:
    print(f"Table creation failed: {e}")
    # Try loading if it exists
    try:
        table = catalog.load_table(table_name)
        print(f"Loaded existing table '{table_name}'.")
    except Exception as load_e:
        print(f"Could not load table: {load_e}")
        exit(1)

print("\\nVerification Complete!")
