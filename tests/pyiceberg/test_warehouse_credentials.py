#!/usr/bin/env python3
"""
Test Scenario A: Warehouse-Based Credential Vending
Server vends credentials from warehouse configuration
"""

import requests
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, LongType, StringType, DoubleType
import pyarrow as pa

print("=" * 70)
print("SCENARIO A: WAREHOUSE-BASED CREDENTIAL VENDING")
print("=" * 70)

# Step 1: Create warehouse with MinIO credentials
print("\n=== 1. Creating Warehouse with Credentials ===")
warehouse_data = {
    "name": "minio_warehouse",
    "use_sts": False,
    "storage_config": {
        "type": "s3",
        "bucket": "warehouse",
        "access_key_id": "minioadmin",
        "secret_access_key": "minioadmin",
        "endpoint": "http://localhost:9000",
        "region": "us-east-1"
    }
}

resp = requests.post(
    "http://localhost:8080/api/v1/warehouses",
    json=warehouse_data
)
if resp.status_code == 200:
    print("✓ Created warehouse: minio_warehouse")
elif "already exists" in resp.text.lower():
    print("⚠ Warehouse already exists")
else:
    print(f"✗ Failed to create warehouse: {resp.status_code} - {resp.text}")

# Step 2: Create catalog linked to warehouse
print("\n=== 2. Creating Catalog Linked to Warehouse ===")
catalog_data = {
    "name": "analytics",
    "warehouse_name": "minio_warehouse",
    "storage_location": "s3://warehouse/analytics/"
}

resp = requests.post(
    "http://localhost:8080/api/v1/catalogs",
    json=catalog_data
)
if resp.status_code == 200:
    print("✓ Created catalog: analytics")
elif "already exists" in resp.text.lower():
    print("⚠ Catalog already exists")
else:
    print(f"✗ Failed to create catalog: {resp.status_code} - {resp.text}")

# Step 3: Connect PyIceberg (no credentials needed - will be vended)
print("\n=== 3. Connecting PyIceberg (Credential Vending) ===")
catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080",
        "prefix": "analytics",
    }
)
print(f"✓ Connected - credentials will be vended by server")

# Step 4: Create namespace
print("\n=== 4. Creating Namespace ===")
try:
    catalog.create_namespace("vended_creds")
    print("✓ Created namespace: vended_creds")
except:
    print("⚠ Namespace already exists")

# Step 5: Create table
print("\n=== 5. Creating Table ===")
schema = Schema(
    NestedField(1, "id", LongType()),
    NestedField(2, "name", StringType()),
    NestedField(3, "amount", DoubleType()),
)

try:
    table = catalog.create_table("vended_creds.transactions", schema=schema)
    print(f"✓ Created table: transactions")
except Exception as e:
    print(f"⚠ Table exists: {e}")
    table = catalog.load_table("vended_creds.transactions")

# Step 6: Write data (server vends credentials)
print("\n=== 6. Writing Data (Server Vends Credentials) ===")
data = pa.Table.from_pydict({
    "id": [1, 2, 3, 4, 5],
    "name": ["tx1", "tx2", "tx3", "tx4", "tx5"],
    "amount": [100.0, 250.5, 175.25, 300.75, 425.0],
})

try:
    print(f"  Writing {len(data)} rows...")
    table.append(data)
    print("✓✓✓ DATA WRITTEN SUCCESSFULLY! ✓✓✓")
    print("  Credential vending is working!")
except Exception as e:
    print(f"✗ Write failed: {e}")
    import traceback
    traceback.print_exc()

# Step 7: Read data
print("\n=== 7. Reading Data ===")
try:
    scan = table.scan()
    result = scan.to_arrow()
    print(f"✓ Read {len(result)} rows")
    if len(result) > 0:
        print("\nData:")
        print(result.to_pandas())
except Exception as e:
    print(f"✗ Read failed: {e}")

print("\n" + "=" * 70)
print("SCENARIO A SUMMARY")
print("=" * 70)
try:
    if len(result) > 0:
        print("✓ Warehouse-Based Credential Vending: WORKING")
        print("🎉 FULL INTEGRATION SUCCESS!")
    else:
        print("✗ Credential Vending: FAILED")
except:
    print("✗ Test incomplete")
print("=" * 70)
