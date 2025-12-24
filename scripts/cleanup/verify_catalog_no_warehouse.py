import os
import pyarrow as pa
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, IntegerType

# Configuration
CATALOG_NAME = "catalog2"
TENANT_ID = "00000000-0000-0000-0000-000000000000"
ADMIN_TOKEN = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIwMDAwMDAwMC0wMDAwLTAwMDAtMDAwMC0wMDAwMDAwMDAwMDAiLCJqdGkiOiI4OTRiY2I4ZC0wZWZhLTRmM2EtYWJjOS1mNWE2NWQ5ZDJlNjQiLCJ1c2VybmFtZSI6ImFkbWluIiwidGVuYW50X2lkIjoiMDAwMDAwMDAtMDAwMC0wMDAwLTAwMDAtMDAwMDAwMDAwMDAwIiwicm9sZSI6InRlbmFudC1hZG1pbiIsImV4cCI6MTc2NjU4MjMyMywiaWF0IjoxNzY2NDk1OTIzfQ.gxJi5OSFjXqe2W7dACCTOODf8hYSvEMhX7tTpleGj8Y"
API_URL = f"http://localhost:8080/v1/{CATALOG_NAME}"

print(f"--- Verifying Catalog: {CATALOG_NAME} (No Warehouse) ---")

# Load catalog with client-side storage settings
# Since the catalog has no warehouse, we MUST provide 'warehouse' and S3 credentials
catalog = load_catalog(
    CATALOG_NAME,
    **{
        "uri": API_URL,
        "header.X-Pangolin-Tenant": TENANT_ID,
        "token": ADMIN_TOKEN,
        # Client-side storage configuration
        "s3.endpoint": "http://localhost:9000",
        "s3.access-key-id": "minioadmin",
        "s3.secret-access-key": "minioadmin",
        "warehouse": f"s3://warehouse/{CATALOG_NAME}", # Path manual override
        "py-io-impl": "pyiceberg.io.pyarrow.PyArrowFileIO"
    }
)

# 1. Create a Namespace
namespace = "test_ns"
print(f"Creating namespace: {namespace}...")
try:
    catalog.create_namespace(namespace)
except Exception as e:
    print(f"Namespace might already exist: {e}")

# 2. Define Schema
schema = Schema(
    NestedField(1, "id", IntegerType(), required=True),
    NestedField(2, "data", StringType(), required=True),
)

# 3. Create Tables
tables = ["table_a", "table_b"]
for table_name in tables:
    identifier = f"{namespace}.{table_name}"
    print(f"Creating table: {identifier}...")
    
    # Delete if exists
    try:
        catalog.drop_table(identifier)
    except:
        pass
        
    # Explicitly define location client-side
    location = f"s3://warehouse/{CATALOG_NAME}/{namespace}/{table_name}"
    table = catalog.create_table(identifier, schema=schema, location=location)
    print(f"Successfully created: {identifier}")

    # 4. Write some data
    df = pa.Table.from_pydict({
        "id": [1, 2, 3],
        "data": [f"val1_{table_name}", f"val2_{table_name}", f"val3_{table_name}"]
    })
    
    print(f"Writing data to {identifier}...")
    table.append(df)

    # 5. Read back data
    print(f"Reading data from {identifier}...")
    read_df = table.scan().to_arrow()
    print(f"Data read from {identifier}:")
    print(read_df)
    print("-" * 20)

print("\nVerification Complete!")
