# Apache Iceberg Best Practices

## Overview

This guide covers best practices for using Apache Iceberg tables with Pangolin, optimizing for performance, reliability, and maintainability.

## Table Design

### Partitioning Strategy

**Time-Based Partitioning**
```python
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, TimestampType, LongType
from pyiceberg.partitioning import PartitionSpec, PartitionField
from pyiceberg.transforms import DayTransform

# Schema
schema = Schema(
    NestedField(1, "event_id", LongType(), required=True),
    NestedField(2, "event_time", TimestampType(), required=True),
    NestedField(3, "user_id", StringType(), required=True),
    NestedField(4, "event_type", StringType(), required=True)
)

# Partition by day
partition_spec = PartitionSpec(
    PartitionField(source_id=2, field_id=1000, transform=DayTransform(), name="event_day")
)
```

**Best Practices**
- Partition by query patterns (e.g., date for time-series data)
- Avoid over-partitioning (target 100MB-1GB per partition)
- Use transforms (day, month, hour) instead of raw values
- Consider cardinality (avoid high-cardinality partitions)

**Multi-Dimensional Partitioning**
```python
# Partition by date and region
partition_spec = PartitionSpec(
    PartitionField(source_id=2, field_id=1000, transform=DayTransform(), name="event_day"),
    PartitionField(source_id=5, field_id=1001, transform=IdentityTransform(), name="region")
)
```

### Schema Evolution

**Adding Columns**
```python
# Add new column (safe operation)
table.update_schema().add_column("new_field", StringType()).commit()

# Add with default value
table.update_schema().add_column(
    "status",
    StringType(),
    doc="Order status"
).commit()
```

**Renaming Columns**
```python
# Rename column (metadata-only operation)
table.update_schema().rename_column("old_name", "new_name").commit()
```

**Type Promotion**
```python
# Safe type promotions
table.update_schema().update_column("age", IntegerType()).commit()  # int -> long
table.update_schema().update_column("price", DoubleType()).commit()  # float -> double
```

**Best Practices**
- Use schema evolution instead of recreating tables
- Test schema changes in development first
- Document schema changes in metadata
- Avoid dropping columns (mark as deprecated instead)

### Sorting

**Sort Order**
```python
from pyiceberg.table.sorting import SortOrder, SortField
from pyiceberg.transforms import IdentityTransform

# Sort by frequently filtered columns
sort_order = SortOrder(
    SortField(source_id=3, transform=IdentityTransform()),  # user_id
    SortField(source_id=2, transform=IdentityTransform())   # event_time
)

table.update_properties().set("write.distribution-mode", "hash").commit()
```

**Benefits**
- Faster queries on sorted columns
- Better compression
- Improved data skipping

## Write Operations

### Batch Writes

**Optimize Batch Size**
```python
# Target 100MB-1GB per file
table.update_properties().set("write.target-file-size-bytes", "536870912").commit()  # 512MB

# Append data in batches
batch_size = 100000
for i in range(0, len(data), batch_size):
    batch = data[i:i+batch_size]
    table.append(batch)
```

### Upserts/Merges

**Merge-on-Read**
```python
# Efficient for frequent updates
table.update_properties().set("write.merge.mode", "merge-on-read").commit()

# Update rows
table.update().where("user_id = 'user123'").set("status", "active").commit()
```

**Copy-on-Write**
```python
# Better for read-heavy workloads
table.update_properties().set("write.merge.mode", "copy-on-write").commit()
```

### Compaction

**Auto-Compaction**
```python
# Enable automatic compaction
table.update_properties().set("write.metadata.compression-codec", "gzip").commit()
table.update_properties().set("write.metadata.metrics.default", "truncate(16)").commit()
```

**Manual Compaction**
```bash
# Compact small files
pangolin-admin optimize-catalog \
  --catalog production \
  --namespace events \
  --table user_events \
  --compact-files
```

## Read Operations

### Partition Pruning

**Leverage Partitions**
```python
# Query with partition filter (fast)
df = table.scan(
    row_filter="event_day >= '2025-12-01' AND event_day < '2025-12-31'"
).to_pandas()

# Without partition filter (slow - full scan)
df = table.scan(
    row_filter="event_type = 'click'"
).to_pandas()
```

### Column Projection

**Select Only Needed Columns**
```python
# Good - only read necessary columns
df = table.scan(
    selected_fields=("event_id", "event_time", "user_id")
).to_pandas()

# Bad - reads all columns
df = table.scan().to_pandas()
```

### Predicate Pushdown

**Filter Early**
```python
# Pushdown to file level
df = table.scan(
    row_filter="user_id = 'user123' AND event_time > '2025-12-01'"
).to_pandas()
```

## Snapshot Management

### Retention Policy

**Configure Retention**
```python
# Keep snapshots for 7 days
table.update_properties().set("history.expire.max-snapshot-age-ms", "604800000").commit()

# Keep minimum 10 snapshots
table.update_properties().set("history.expire.min-snapshots-to-keep", "10").commit()
```

**Manual Cleanup**
```bash
# Remove old snapshots
pangolin-admin optimize-catalog \
  --catalog production \
  --remove-old-snapshots \
  --retention-days 30
```

### Time Travel

**Query Historical Data**
```python
# Query as of timestamp
df = table.scan(
    snapshot_id=table.history()[0].snapshot_id
).to_pandas()

# Query as of specific time
import datetime
timestamp = datetime.datetime(2025, 12, 1).timestamp() * 1000
df = table.scan(as_of_timestamp=timestamp).to_pandas()
```

## Performance Optimization

### File Size

**Target File Size**
```python
# Optimal: 100MB-1GB per file
table.update_properties().set("write.target-file-size-bytes", "536870912").commit()  # 512MB
```

**Avoid Small Files**
- Combine small writes into batches
- Run regular compaction
- Use appropriate batch sizes

### Metadata Caching

**Enable Caching**
```python
# Cache metadata for faster queries
catalog_properties = {
    "cache-enabled": "true",
    "cache.expiration-interval-ms": "300000"  # 5 minutes
}
```

### Statistics

**Collect Statistics**
```python
# Enable column statistics
table.update_properties().set("write.metadata.metrics.default", "full").commit()

# Specific columns
table.update_properties().set("write.metadata.metrics.column.user_id", "counts").commit()
```

## Data Quality

### Schema Validation

**Enforce Schema**
```python
# Require schema compatibility
table.update_properties().set("commit.manifest.min-count-to-merge", "5").commit()
table.update_properties().set("commit.manifest-merge.enabled", "true").commit()
```

### Constraints

**Not Null Constraints**
```python
# Define required fields in schema
schema = Schema(
    NestedField(1, "id", LongType(), required=True),  # NOT NULL
    NestedField(2, "name", StringType(), required=False)  # NULLABLE
)
```

## Maintenance

### Regular Tasks

**Daily**
- Monitor table growth
- Check query performance
- Review error logs

**Weekly**
- Compact small files
- Review partition distribution
- Check snapshot count

**Monthly**
- Expire old snapshots
- Remove orphaned files
- Optimize table layout

### Monitoring

**Key Metrics**
```python
# Table size
metadata = table.metadata
total_size = sum(f.file_size_in_bytes for f in table.scan().plan_files())

# File count
file_count = len(list(table.scan().plan_files()))

# Snapshot count
snapshot_count = len(table.history())

# Partition count
partition_count = len(table.scan().plan_files())
```

## Migration

### From Hive/Parquet

**In-Place Migration**
```python
# Register existing Parquet as Iceberg
from pyiceberg.catalog import load_catalog

catalog = load_catalog("pangolin")
catalog.register_table(
    identifier="analytics.legacy_table",
    metadata_location="s3://bucket/path/to/metadata/v1.metadata.json"
)
```

**Copy Migration**
```python
# Copy data to new Iceberg table
source_df = spark.read.parquet("s3://bucket/legacy/data")
source_df.writeTo("analytics.new_iceberg_table").create()
```

## Best Practices Checklist

### Table Design
- [ ] Appropriate partitioning strategy
- [ ] Optimal file size (100MB-1GB)
- [ ] Sort order defined for common queries
- [ ] Schema documented

### Operations
- [ ] Batch writes configured
- [ ] Compaction scheduled
- [ ] Snapshot retention policy set
- [ ] Statistics collection enabled

### Performance
- [ ] Partition pruning utilized
- [ ] Column projection used
- [ ] Metadata caching enabled
- [ ] Regular maintenance scheduled

### Governance
- [ ] Schema evolution documented
- [ ] Data quality rules defined
- [ ] Retention policies configured
- [ ] Access patterns monitored

## Additional Resources

- [PyIceberg Integration](../features/pyiceberg_testing.md)
- [Table Formats Guide](../features/table_formats.md)
- [Maintenance Operations](../features/maintenance.md)
- [PyPangolin Iceberg Guide](../../pypangolin/docs/iceberg.md)
