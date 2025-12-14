# Time Travel Queries

## Overview

Pangolin supports Iceberg's time-travel capabilities, allowing users to query tables as they existed at a specific point in time or snapshot. This is essential for:
- Auditing historical data states
- Reproducing past analysis results
- Recovering from accidental data modifications
- Comparing data across time periods

---

## Usage

Time-travel queries are performed using Iceberg's standard query parameters.

### By Snapshot ID

Query the table as it was at a specific snapshot:

```bash
GET /v1/analytics/namespaces/sales/tables/transactions?snapshot-id=123456789
Authorization: Bearer <token>
X-Pangolin-Tenant: <tenant-id>
```

### By Timestamp

Query the table as it was at a specific timestamp (milliseconds since epoch):

```bash
GET /v1/analytics/namespaces/sales/tables/transactions?as-of-timestamp=1678886400000
Authorization: Bearer <token>
X-Pangolin-Tenant: <tenant-id>
```

---

## Client Examples

### PyIceberg

```python
from pyiceberg.catalog import load_catalog
from datetime import datetime

catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080",
        "prefix": "analytics",
        "token": "your-jwt-token",
    }
)

table = catalog.load_table("sales.transactions")

# Query by snapshot ID
snapshot_id = 123456789
df = table.scan(snapshot_id=snapshot_id).to_pandas()

# Query by timestamp
timestamp = datetime(2024, 1, 15, 10, 30, 0)
df = table.scan(as_of_timestamp=timestamp).to_pandas()

# Query as of yesterday
from datetime import timedelta
yesterday = datetime.now() - timedelta(days=1)
df = table.scan(as_of_timestamp=yesterday).to_pandas()
```

### Spark SQL

```sql
-- Query by snapshot ID
SELECT * FROM analytics.sales.transactions VERSION AS OF 123456789;

-- Query by timestamp
SELECT * FROM analytics.sales.transactions TIMESTAMP AS OF '2024-01-15 10:30:00';

-- Query as of yesterday
SELECT * FROM analytics.sales.transactions TIMESTAMP AS OF current_timestamp() - INTERVAL 1 DAY;
```

### Trino

```sql
-- Query by snapshot ID
SELECT * FROM iceberg.analytics.sales.transactions FOR VERSION AS OF 123456789;

-- Query by timestamp
SELECT * FROM iceberg.analytics.sales.transactions FOR TIMESTAMP AS OF TIMESTAMP '2024-01-15 10:30:00';
```

---

## Snapshot Management

### List Snapshots

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog("pangolin", **config)
table = catalog.load_table("sales.transactions")

# Get table metadata
metadata = table.metadata

# List all snapshots
for snapshot in metadata.snapshots:
    print(f"Snapshot ID: {snapshot.snapshot_id}")
    print(f"Timestamp: {snapshot.timestamp_ms}")
    print(f"Operation: {snapshot.summary.get('operation')}")
    print("---")
```

### Get Snapshot Details

```python
# Get specific snapshot
snapshot_id = 123456789
snapshot = table.snapshot_by_id(snapshot_id)

print(f"Snapshot ID: {snapshot.snapshot_id}")
print(f"Parent ID: {snapshot.parent_snapshot_id}")
print(f"Timestamp: {snapshot.timestamp_ms}")
print(f"Manifest List: {snapshot.manifest_list}")
```

---

## Implementation Details

When a time-travel parameter is provided:

1. **Pangolin retrieves** the current metadata for the table
2. **Verifies** if the requested snapshot exists in the metadata history
3. **Returns** the metadata pointing to the requested snapshot
4. **Client reads** data files from the snapshot's manifest list

If the snapshot is not found (e.g., expired and cleaned up), a `404 Not Found` is returned.

---

## Snapshot Retention

### Default Retention

Iceberg tables retain snapshots based on configuration:
- **Default**: Last 5 snapshots
- **Time-based**: Snapshots from last 7 days

### Configure Retention

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog("pangolin", **config)
table = catalog.load_table("sales.transactions")

# Update table properties
table.update_properties(
    **{
        "history.expire.max-snapshot-age-ms": "604800000",  # 7 days
        "history.expire.min-snapshots-to-keep": "10"
    }
)
```

### Expire Snapshots

```python
# Expire snapshots older than 7 days
table.expire_snapshots(
    older_than=datetime.now() - timedelta(days=7),
    retain_last=5
)
```

---

## Use Cases

### Audit Historical Data

```python
# Compare data from last month to current
last_month = datetime.now() - timedelta(days=30)
df_last_month = table.scan(as_of_timestamp=last_month).to_pandas()
df_current = table.scan().to_pandas()

# Find differences
diff = df_current.merge(df_last_month, how='outer', indicator=True)
new_records = diff[diff['_merge'] == 'left_only']
deleted_records = diff[diff['_merge'] == 'right_only']
```

### Recover from Accidental Deletion

```python
# Find snapshot before deletion
snapshots = table.metadata.snapshots
for snapshot in reversed(snapshots):
    if snapshot.summary.get('operation') == 'delete':
        # Use snapshot before this one
        previous_snapshot = snapshot.parent_snapshot_id
        df = table.scan(snapshot_id=previous_snapshot).to_pandas()
        break
```

### Reproduce Past Analysis

```python
# Reproduce analysis from specific date
analysis_date = datetime(2024, 1, 15)
df = table.scan(as_of_timestamp=analysis_date).to_pandas()

# Run same analysis
result = df.groupby('category').agg({'amount': 'sum'})
```

---

## Best Practices

### 1. **Configure Appropriate Retention**
Balance between:
- **Storage costs**: More snapshots = more metadata storage
- **Audit requirements**: Compliance may require longer retention
- **Recovery needs**: Keep enough history for rollback scenarios

### 2. **Use Timestamp for Business Logic**
Prefer timestamp-based queries for business logic:
```python
# Good: Business-friendly
df = table.scan(as_of_timestamp=datetime(2024, 1, 1)).to_pandas()

# Less ideal: Requires knowing snapshot IDs
df = table.scan(snapshot_id=123456789).to_pandas()
```

### 3. **Document Snapshot IDs for Critical States**
For important milestones, document snapshot IDs:
```python
# Production release snapshot
PROD_RELEASE_V1 = 123456789
df = table.scan(snapshot_id=PROD_RELEASE_V1).to_pandas()
```

### 4. **Regular Snapshot Cleanup**
Automate snapshot expiration to manage storage:
```python
# Weekly cleanup job
table.expire_snapshots(
    older_than=datetime.now() - timedelta(days=30),
    retain_last=10
)
```

### 5. **Test Time Travel Queries**
Verify time travel works before relying on it:
```python
# Test query
snapshots = table.metadata.snapshots
if len(snapshots) > 1:
    df = table.scan(snapshot_id=snapshots[0].snapshot_id).to_pandas()
    assert len(df) >= 0  # Verify query works
```

---

## Troubleshooting

### "Snapshot not found"

**Cause**: Snapshot expired or never existed.

**Solution**:
1. List available snapshots: `table.metadata.snapshots`
2. Check retention policy
3. Use a more recent snapshot or timestamp

### "Timestamp too old"

**Cause**: No snapshots exist for the requested timestamp.

**Solution**:
1. Check oldest available snapshot: `min(s.timestamp_ms for s in table.metadata.snapshots)`
2. Use a more recent timestamp
3. Increase snapshot retention if needed

### Performance issues with old snapshots

**Cause**: Old snapshots may reference many small files.

**Solution**:
1. Consider table compaction
2. Use more recent snapshots when possible
3. Optimize query filters to reduce data scanned

---

## Related Documentation

- [Branch Management](./branch_management.md) - Git-like branching for catalogs
- [Merge Conflicts](../merge_conflicts.md) - Merging branches
- [PyIceberg Testing](./pyiceberg_testing.md) - PyIceberg integration
