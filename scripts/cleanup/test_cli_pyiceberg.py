#!/usr/bin/env python3
"""
Test PyIceberg integration with CLI-created warehouse
Uses Basic Auth instead of OAuth
"""
import os
import requests
from pyiceberg.catalog import load_catalog

# Configuration
CATALOG_NAME = "cli-test-catalog"
NAMESPACE = "test_ns"
TABLE_NAME = "test_table"

print("=== PyIceberg Integration Test with CLI-Created Warehouse ===\n")

# Create catalog using Basic Auth
print("1. Loading catalog...")
catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080",
        "credential": "cli-admin:testpass123",
        "warehouse": CATALOG_NAME,
        "header.X-Iceberg-Access-Delegation": "vended-credentials",
    }
)
print(f"✅ Catalog loaded: {CATALOG_NAME}\n")

# Create namespace
print(f"2. Creating namespace '{NAMESPACE}'...")
try:
    catalog.create_namespace(NAMESPACE)
    print(f"✅ Namespace created\n")
except Exception as e:
    if "already exists" in str(e).lower():
        print(f"⚠️  Namespace already exists\n")
    else:
        raise

# Create table
print(f"3. Creating table '{NAMESPACE}.{TABLE_NAME}'...")
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, IntegerType

schema = Schema(
    NestedField(1, "id", IntegerType(), required=True),
    NestedField(2, "name", StringType(), required=False),
)

try:
    table = catalog.create_table(f"{NAMESPACE}.{TABLE_NAME}", schema=schema)
    print(f"✅ Table created\n")
except Exception as e:
    if "already exists" in str(e).lower():
        print(f"⚠️  Table already exists, loading it...\n")
        table = catalog.load_table(f"{NAMESPACE}.{TABLE_NAME}")
    else:
        raise

# Write data
print("4. Writing data...")
import pyarrow as pa

data = pa.Table.from_pydict({
    "id": [1, 2, 3],
    "name": ["Alice", "Bob", "Charlie"]
})

table.append(data)
print(f"✅ Data written ({len(data)} rows)\n")

# Read data
print("5. Reading data...")
df = table.scan().to_arrow()
print(f"✅ Data read ({len(df)} rows)")
print(f"   Data: {df.to_pydict()}\n")

print("=" * 60)
print("✅ ALL TESTS PASSED!")
print("=" * 60)
print("\nCLI warehouse creation with correct property names works perfectly!")
print("PyIceberg can successfully:")
print("  - Connect to catalog")
print("  - Create namespaces")
print("  - Create tables")
print("  - Write data")
print("  - Read data")
