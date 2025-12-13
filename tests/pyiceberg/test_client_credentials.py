#!/usr/bin/env python3
"""
Test Scenario B: Client-Provided Credentials
PyIceberg provides S3 credentials directly in config
"""

from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, LongType, StringType, DoubleType
import pyarrow as pa

print("=" * 70)
print("SCENARIO B: CLIENT-PROVIDED CREDENTIALS")
print("=" * 70)

# Connect with client-provided S3 credentials
print("\n=== 1. Connecting with Client Credentials ===")
catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080",
        "prefix": "analytics",
        "s3.endpoint": "http://localhost:9000",
        "s3.access-key-id": "minioadmin",
        "s3.secret-access-key": "minioadmin",
        "s3.region": "us-east-1",
    }
)
print(f"✓ Connected with client credentials")

# Create namespace
print("\n=== 2. Creating Namespace ===")
try:
    catalog.create_namespace("client_creds")
    print("✓ Created namespace: client_creds")
except:
    print("⚠ Namespace already exists")

# Create table
print("\n=== 3. Creating Table ===")
schema = Schema(
    NestedField(1, "id", LongType()),
    NestedField(2, "name", StringType()),
    NestedField(3, "value", DoubleType()),
)

try:
    table = catalog.create_table("client_creds.test_table", schema=schema)
    print(f"✓ Created table: test_table")
except Exception as e:
    print(f"⚠ Table exists: {e}")
    table = catalog.load_table("client_creds.test_table")

# Write data
print("\n=== 4. Writing Data with Client Credentials ===")
data = pa.Table.from_pydict({
    "id": [1, 2, 3],
    "name": ["alpha", "beta", "gamma"],
    "value": [100.5, 200.75, 300.25],
})

try:
    print(f"  Writing {len(data)} rows...")
    table.append(data)
    print("✓✓✓ DATA WRITTEN SUCCESSFULLY! ✓✓✓")
    print("  Client-provided credentials working!")
except Exception as e:
    print(f"✗ Write failed: {e}")

# Read data
print("\n=== 5. Reading Data ===")
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
print("SCENARIO B SUMMARY")
print("=" * 70)
try:
    if len(result) > 0:
        print("✓ Client-Provided Credentials: WORKING")
        print("🎉 FULL INTEGRATION SUCCESS!")
    else:
        print("✗ Client-Provided Credentials: FAILED")
except:
    print("✗ Test incomplete")
print("=" * 70)
