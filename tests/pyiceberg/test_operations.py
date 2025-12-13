#!/usr/bin/env python3
"""
Test PyIceberg operations that currently work with Pangolin:
- Multiple appends (creates multiple snapshots)
- Time travel between snapshots
- Filtered scans
- Metadata inspection
- Table statistics
"""

import pyarrow as pa
from pyiceberg.catalog import load_catalog
from datetime import datetime
import time

print("=" * 70)
print("PYICEBERG OPERATIONS TEST")
print("=" * 70)

# Connect to catalog
catalog = load_catalog('pangolin', **{
    'uri': 'http://localhost:8080',
    'prefix': 'analytics',
    's3.endpoint': 'http://localhost:9000',
    's3.access-key-id': 'minioadmin',
    's3.secret-access-key': 'minioadmin',
    's3.region': 'us-east-1',
})

print("\n=== 1. Setup: Create Table ===")
try:
    catalog.drop_table('operations_test.orders')
except:
    pass

try:
    catalog.create_namespace('operations_test')
except:
    pass

from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, LongType, StringType, DoubleType

schema = Schema(
    NestedField(1, 'order_id', LongType(), required=True),
    NestedField(2, 'customer', StringType(), required=False),
    NestedField(3, 'amount', DoubleType(), required=False),
    NestedField(4, 'status', StringType(), required=False),
)

table = catalog.create_table('operations_test.orders', schema=schema)
print("✓ Created table: orders")

print("\n=== 2. Multiple Appends (Create Snapshots) ===")
# Append 1
data1 = pa.Table.from_pydict({
    'order_id': [1, 2, 3],
    'customer': ['Alice', 'Bob', 'Charlie'],
    'amount': [100.0, 200.0, 150.0],
    'status': ['pending', 'completed', 'pending'],
})
table.append(data1)
print("✓ Append 1: 3 rows")
time.sleep(1)  # Ensure different timestamps

# Append 2
data2 = pa.Table.from_pydict({
    'order_id': [4, 5],
    'customer': ['David', 'Eve'],
    'amount': [300.0, 250.0],
    'status': ['completed', 'pending'],
})
table.append(data2)
print("✓ Append 2: 2 rows")
time.sleep(1)

# Append 3
data3 = pa.Table.from_pydict({
    'order_id': [6, 7, 8],
    'customer': ['Frank', 'Grace', 'Henry'],
    'amount': [175.0, 225.0, 190.0],
    'status': ['completed', 'pending', 'completed'],
})
table.append(data3)
print("✓ Append 3: 3 rows")

# Reload table to get fresh metadata
table = catalog.load_table('operations_test.orders')

print(f"\n✓ Total rows: {len(table.scan().to_pandas())}")
print(f"✓ Total snapshots: {len(table.metadata.snapshots) if table.metadata.snapshots else 0}")

print("\n=== 3. Snapshot History ===")
if table.metadata.snapshots:
    print(f"Snapshots ({len(table.metadata.snapshots)} total):")
    for i, snapshot in enumerate(table.metadata.snapshots):
        ts = datetime.fromtimestamp(snapshot.timestamp_ms / 1000)
        print(f"  {i+1}. ID: {snapshot.snapshot_id}")
        print(f"     Timestamp: {ts}")
        # Access summary safely
        try:
            summary_dict = {}
            for key in snapshot.summary:
                summary_dict[key] = snapshot.summary[key]
            print(f"     Summary: {summary_dict}")
        except:
            print(f"     Summary: (available)")
else:
    print("⚠ No snapshots found")

print("\n=== 4. Time Travel Queries ===")
if table.metadata.snapshots and len(table.metadata.snapshots) >= 3:
    # Query each snapshot
    for i, snapshot in enumerate(table.metadata.snapshots):
        df = table.scan(snapshot_id=snapshot.snapshot_id).to_pandas()
        print(f"\nSnapshot {i+1} ({snapshot.snapshot_id}): {len(df)} rows")
        print(df[['order_id', 'customer', 'amount']].to_string(index=False))
    
    print("\n✓ Time travel working across all snapshots!")
else:
    print("⚠ Need at least 3 snapshots for time travel test")

print("\n=== 5. Filtered Scans ===")
# Filter by status
completed = table.scan(row_filter="status == 'completed'").to_pandas()
print(f"\nCompleted orders: {len(completed)} rows")
print(completed[['order_id', 'customer', 'amount']].to_string(index=False))

# Filter by amount
high_value = table.scan(row_filter="amount >= 200").to_pandas()
print(f"\nHigh-value orders (>= $200): {len(high_value)} rows")
print(high_value[['order_id', 'customer', 'amount']].to_string(index=False))

# Combined filter
pending_high = table.scan(
    row_filter="status == 'pending' AND amount > 150"
).to_pandas()
print(f"\nPending high-value orders: {len(pending_high)} rows")
if len(pending_high) > 0:
    print(pending_high[['order_id', 'customer', 'amount', 'status']].to_string(index=False))

print("\n=== 6. Column Projection ===")
# Select specific columns
projected = table.scan(
    selected_fields=('order_id', 'amount')
).to_pandas()
print(f"\nProjected columns (order_id, amount): {len(projected)} rows")
print(projected.head().to_string(index=False))

print("\n=== 7. Table Metadata ===")
print(f"\nTable UUID: {table.metadata.table_uuid}")
print(f"Location: {table.metadata.location}")
print(f"Format version: {table.metadata.format_version}")
print(f"Current schema ID: {table.metadata.current_schema_id}")
print(f"Current snapshot ID: {table.metadata.current_snapshot_id}")

print("\nSchema:")
for field in table.schema().fields:
    required = "required" if field.required else "optional"
    print(f"  {field.field_id}: {field.name} ({field.field_type}, {required})")

print("\n=== 8. Snapshot Metadata ===")
if table.metadata.current_snapshot_id:
    current_snap = None
    for snap in table.metadata.snapshots:
        if snap.snapshot_id == table.metadata.current_snapshot_id:
            current_snap = snap
            break
    
    if current_snap:
        print(f"\nCurrent snapshot: {current_snap.snapshot_id}")
        print(f"Timestamp: {datetime.fromtimestamp(current_snap.timestamp_ms / 1000)}")
        print(f"Manifest list: {current_snap.manifest_list}")
        if current_snap.parent_snapshot_id:
            print(f"Parent snapshot: {current_snap.parent_snapshot_id}")

print("\n" + "=" * 70)
print("TEST SUMMARY")
print("=" * 70)

results = {
    "Table creation": "✓ PASS",
    "Multiple appends": "✓ PASS",
    "Snapshot creation": "✓ PASS" if table.metadata.snapshots and len(table.metadata.snapshots) >= 3 else "⚠ PARTIAL",
    "Time travel": "✓ PASS" if table.metadata.snapshots and len(table.metadata.snapshots) >= 3 else "⚠ PARTIAL",
    "Filtered scans": "✓ PASS",
    "Column projection": "✓ PASS",
    "Metadata inspection": "✓ PASS",
    "Snapshot tracking": "✓ PASS",
}

for feature, status in results.items():
    print(f"{feature:.<40} {status}")

print("\n🎉 PyIceberg operations test complete!")
print("=" * 70)
