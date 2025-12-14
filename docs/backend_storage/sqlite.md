# SQLite Backend Storage

SQLite is a lightweight, embedded SQL database that provides zero-configuration storage for Pangolin's metadata.

## Overview

SQLite stores all Pangolin catalog metadata in a single file with:
- Zero configuration required
- ACID compliance
- Full SQL support
- Extremely low resource usage
- Perfect for development and embedded deployments

## Pros and Cons

### ✅ Advantages
- **Zero Configuration**: No separate database server needed
- **Single File**: Entire database in one file
- **Lightweight**: Minimal memory and CPU usage
- **Fast**: Excellent performance for read-heavy workloads
- **Portable**: Copy the file to move the database
- **Embedded**: Perfect for edge devices and embedded systems
- **ACID Compliant**: Full transaction support
- **Foreign Keys**: Referential integrity support
- **No Network**: No network latency

### ⚠️ Considerations
- **Single Writer**: Limited concurrent write performance
- **No Replication**: Manual backup/restore required
- **File-Based**: Not suitable for distributed systems
- **Scalability**: Limited to single machine
- **No User Management**: File-level permissions only

## Use Cases

**Best For**:
- Local development
- Testing and CI/CD
- Embedded applications
- Edge/IoT deployments
- Single-user applications
- Prototyping

**Not Ideal For**:
- High-concurrency write workloads
- Distributed systems
- Multi-server deployments
- When you need replication
- Large-scale production (use PostgreSQL/MongoDB)

## Prerequisites

- None! SQLite is embedded in Pangolin

## Installation

No installation required - SQLite is built into Pangolin.

## Configuration

### Environment Variables

```bash
# File-based database
DATABASE_URL=sqlite:///path/to/pangolin.db

# Relative path
DATABASE_URL=sqlite://./pangolin.db

# In-memory database (testing only, data lost on restart)
DATABASE_URL=sqlite::memory:

# Absolute path
DATABASE_URL=sqlite:////home/user/pangolin/data/pangolin.db
```

### Schema Initialization

Pangolin automatically creates the schema on first startup. The schema includes:

**Tables**:
- `tenants` - Tenant information
- `warehouses` - Storage warehouse configurations
- `catalogs` - Catalog definitions
- `namespaces` - Namespace hierarchies
- `assets` - Table and view metadata
- `branches` - Branch information
- `tags` - Tag information
- `commits` - Commit history
- `metadata_locations` - Iceberg metadata file locations
- `audit_logs` - Audit trail

**Indexes**:
- Primary keys on all tables
- Foreign key indexes
- Composite indexes for common queries

### Manual Schema Application

```bash
# Download schema
curl -O https://raw.githubusercontent.com/AlexMercedCoder/Pangolin/main/pangolin/pangolin_store/sql/sqlite_schema.sql

# Apply schema
sqlite3 pangolin.db < sqlite_schema.sql
```

## Performance Tuning

### Recommended PRAGMA Settings

```sql
-- Enable foreign keys (required)
PRAGMA foreign_keys = ON;

-- Performance optimizations
PRAGMA journal_mode = WAL;           -- Write-Ahead Logging for better concurrency
PRAGMA synchronous = NORMAL;         -- Balance between safety and speed
PRAGMA cache_size = -64000;          -- 64MB cache
PRAGMA temp_store = MEMORY;          -- Use memory for temp tables
PRAGMA mmap_size = 30000000000;      -- 30GB memory-mapped I/O
PRAGMA page_size = 4096;             -- 4KB pages (default)
```

### Monitoring

```sql
-- Database size
SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size();

-- Table sizes
SELECT 
  name,
  SUM(pgsize) as size
FROM dbstat
GROUP BY name
ORDER BY size DESC;

-- Index usage
EXPLAIN QUERY PLAN SELECT * FROM catalogs WHERE tenant_id = '...';
```

## Backup and Restore

### File-Based Backup

```bash
# Simple file copy (database must be idle)
cp pangolin.db pangolin_backup.db

# Using SQLite backup command (safe while database is in use)
sqlite3 pangolin.db ".backup pangolin_backup.db"

# Dump to SQL
sqlite3 pangolin.db .dump > pangolin_backup.sql

# Restore from SQL
sqlite3 pangolin_new.db < pangolin_backup.sql
```

### Automated Backups

```bash
#!/bin/bash
# backup_sqlite.sh

BACKUP_DIR="/backups/pangolin"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
DB_FILE="/path/to/pangolin.db"

mkdir -p $BACKUP_DIR
sqlite3 $DB_FILE ".backup $BACKUP_DIR/pangolin_$TIMESTAMP.db"

# Keep only last 7 days
find $BACKUP_DIR -name "pangolin_*.db" -mtime +7 -delete
```

## High Availability

SQLite does not support built-in replication. For high availability:

1. **Use PostgreSQL or MongoDB** for production
2. **File-level replication**: Use tools like Litestream
3. **Regular backups**: Automated backup scripts

### Litestream (Continuous Replication)

```bash
# Install Litestream
curl -s https://api.github.com/repos/benbjohnson/litestream/releases/latest \
  | grep "browser_download_url.*linux-amd64.tar.gz" \
  | cut -d : -f 2,3 \
  | tr -d \" \
  | wget -qi -

# Configure litestream.yml
replicas:
  - url: s3://my-bucket/pangolin.db
    
# Run Litestream
litestream replicate pangolin.db
```

## Security Best Practices

1. **File Permissions**: Restrict access to database file
   ```bash
   chmod 600 pangolin.db
   chown pangolin:pangolin pangolin.db
   ```

2. **Encryption at Rest**: Use filesystem encryption (LUKS, dm-crypt)

3. **Backup Encryption**: Encrypt backup files
   ```bash
   sqlite3 pangolin.db .dump | gpg -c > pangolin_backup.sql.gpg
   ```

4. **Network Security**: SQLite has no network access (inherently secure)

## Migration

### To PostgreSQL

```bash
# Export from SQLite
sqlite3 pangolin.db .dump > sqlite_dump.sql

# Convert to PostgreSQL (requires manual editing)
# Then import to PostgreSQL
psql -U pangolin -d pangolin -f converted_dump.sql
```

### To MongoDB

Custom migration script required.

## Troubleshooting

### Database Locked

```bash
# Check for locks
lsof | grep pangolin.db

# Force unlock (use with caution)
fuser -k pangolin.db
```

### Corruption

```bash
# Check integrity
sqlite3 pangolin.db "PRAGMA integrity_check;"

# Recover from corruption
sqlite3 pangolin.db ".recover" | sqlite3 pangolin_recovered.db
```

### Performance Issues

```sql
-- Analyze database
ANALYZE;

-- Vacuum to reclaim space
VACUUM;

-- Rebuild indexes
REINDEX;
```

## Test Results

✅ **All 6 SQLite tests passing**:
- `test_sqlite_tenant_crud`
- `test_sqlite_warehouse_crud`
- `test_sqlite_catalog_crud`
- `test_sqlite_namespace_operations`
- `test_sqlite_asset_operations`
- `test_sqlite_multi_tenant_isolation`

## Development Workflow

```bash
# Start Pangolin with SQLite
DATABASE_URL=sqlite://./dev.db cargo run --bin pangolin_api

# Inspect database
sqlite3 dev.db
sqlite> .tables
sqlite> .schema catalogs
sqlite> SELECT * FROM tenants;

# Reset database
rm dev.db
# Restart Pangolin to recreate schema
```

## Additional Resources

- [SQLite Official Documentation](https://www.sqlite.org/docs.html)
- [SQLite Performance Tuning](https://www.sqlite.org/speed.html)
- [Litestream (Replication)](https://litestream.io/)
- [SQLite Best Practices](https://www.sqlite.org/bestpractice.html)

## Next Steps

- [PostgreSQL Backend](postgresql.md)
- [MongoDB Backend](mongodb.md)
- [Backend Comparison](comparison.md)
- [Warehouse Storage](../warehouse/README.md)
