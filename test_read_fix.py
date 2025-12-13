#!/usr/bin/env python3
"""
Quick test to verify reads are working after snapshot fix
"""

from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, LongType, StringType
import pyarrow as pa

print("Testing Data Read After Snapshot Fix")
print("=" * 50)

catalog = load_catalog('pangolin', **{
    'uri': 'http://localhost:8080',
    'prefix': 'analytics',
    's3.endpoint': 'http://localhost:9000',
    's3.access-key-id': 'minioadmin',
    's3.secret-access-key': 'minioadmin',
    's3.region': 'us-east-1',
})

# Create fresh table
try:
    catalog.drop_table('client_creds.read_test')
except:
    pass

schema = Schema(
    NestedField(1, 'id', LongType()),
    NestedField(2, 'name', StringType()),
)

table = catalog.create_table('client_creds.read_test', schema=schema)
print("✓ Created table")

# Write data
data = pa.Table.from_pydict({
    'id': [1, 2, 3],
    'name': ['Alice', 'Bob', 'Charlie'],
})

table.append(data)
print("✓ Wrote 3 rows")

# Read data
scan = table.scan()
result = scan.to_arrow()
print(f"✓ Read {len(result)} rows")

if len(result) > 0:
    print("\n🎉 SUCCESS! Data reads working!")
    print("\nData:")
    print(result.to_pandas())
else:
    print("\n✗ Still reading 0 rows")
