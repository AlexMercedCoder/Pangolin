# PostgreSQL Backend Storage

PostgreSQL is a powerful, open-source relational database that provides excellent performance, reliability, and feature set for Pangolin's metadata storage.

## Overview

PostgreSQL stores all Pangolin catalog metadata in a structured SQL schema with:
- Strong ACID guarantees
- Foreign key constraints for data integrity
- Efficient indexing for fast queries
- Built-in replication and backup tools
- Mature ecosystem and tooling

## Pros and Cons

### ✅ Advantages
- **Battle-tested**: Decades of production use
- **ACID Compliance**: Strong consistency guarantees
- **Rich Query Capabilities**: Complex joins, aggregations, window functions
- **Foreign Keys**: Automatic referential integrity
- **Managed Options**: AWS RDS, Azure Database, Google Cloud SQL
- **Excellent Tooling**: pgAdmin, DBeaver, psql, and many others
- **JSON Support**: JSONB for flexible property storage
- **Full-text Search**: Built-in search capabilities
- **Extensions**: PostGIS, pg_partman, and hundreds more

### ⚠️ Considerations
- **Setup Complexity**: Requires database server installation
- **Resource Usage**: Higher memory footprint than SQLite
- **Vertical Scaling**: Primarily scales up (bigger server)
- **Cost**: Managed services have ongoing costs

## Use Cases

**Best For**:
- Production deployments
- Multi-user environments
- High-concurrency workloads
- Enterprise deployments
- When you need strong consistency

**Not Ideal For**:
- Embedded applications
- Edge/IoT devices
- Serverless with cold starts
- When you need horizontal scaling (consider MongoDB)

## Prerequisites

- PostgreSQL 12+ installed
- Database created for Pangolin
- User with full permissions on the database

## Installation

### Local Development (Docker)

```bash
# Start PostgreSQL
docker run -d \
  --name pangolin-postgres \
  -e POSTGRES_DB=pangolin \
  -e POSTGRES_USER=pangolin \
  -e POSTGRES_PASSWORD=your_password \
  -p 5432:5432 \
  postgres:16

# Verify connection
psql -h localhost -U pangolin -d pangolin
```

### Production (AWS RDS)

```bash
# Create RDS instance via AWS Console or CLI
aws rds create-db-instance \
  --db-instance-identifier pangolin-prod \
  --db-instance-class db.t3.medium \
  --engine postgres \
  --engine-version 16.1 \
  --master-username pangolin \
  --master-user-password your_secure_password \
  --allocated-storage 100 \
  --storage-type gp3 \
  --vpc-security-group-ids sg-xxxxx \
  --db-subnet-group-name your-subnet-group \
  --backup-retention-period 7 \
  --preferred-backup-window "03:00-04:00" \
  --preferred-maintenance-window "mon:04:00-mon:05:00"
```

## Configuration

### Environment Variables

```bash
# Connection string format:
# postgresql://username:password@host:port/database
# postgres://username:password@host:port/database

# Local development
DATABASE_URL=postgres://pangolin:your_password@localhost:5432/pangolin

# Production (RDS)
DATABASE_URL=postgresql://pangolin:password@pangolin-prod.xxxxx.us-east-1.rds.amazonaws.com:5432/pangolin

# With SSL (recommended for production)
DATABASE_URL=postgresql://pangolin:password@host:5432/pangolin?sslmode=require

# Connection pool settings (optional)
DATABASE_MAX_CONNECTIONS=10
DATABASE_MIN_CONNECTIONS=2
DATABASE_CONNECT_TIMEOUT=30
```

### Schema Initialization

Pangolin automatically initializes and migrates the PostgreSQL schema on startup using `sqlx`. Manual schema application is generally not required unless you are performing a custom installation or troubleshooting.

The comprehensive schema includes:

**Core Infrastructure**:
- `tenants` - Multi-tenant isolation records
- `warehouses` - Storage configurations (S3, Azure, GCP)
- `catalogs` - Iceberg catalog definitions
- `namespaces` - Namespace hierarchies and properties
- `assets` - Table and view metadata pointers (Isolated by `branch_name`)
- `branches` - Git-like branch definitions
- `tags` - Immutable commit pointers (snapshots)
- `commits` - Detailed operation history
- `metadata_locations` - Physical Iceberg metadata file path tracking

**Governance & Security**:
- `users` - Root and Tenant user accounts
- `service_users` - Machine-to-machine API identities
- `roles` - RBAC role definitions
- `user_roles` - Role assignments to users
- `permissions` - Direct (TBAC/RBAC) permission grants
- `access_requests` - Data discovery access workflows
- `audit_logs` - Comprehensive tamper-evident trail
- `merge_operations` - Branch merge tracking
- `merge_conflicts` - Conflict details for merges
- `revoked_tokens` - Blacklisted JWT identifiers
- `active_tokens` - (Internal) Session tracking for active tokens

**System & Maintenance**:
- `system_settings` - Tenant-specific configuration overrides
- `federated_sync_stats` - Status tracking for cross-tenant federation

**Indexes**:
- Primary keys on all tables
- Foreign key indexes for joining tenant/catalog/namespace context
- Composite indexes for optimized listing (e.g., `idx_assets_tenant_catalog`)
- GIN indexes for efficient JSONB and Array searches (PostgreSQL specific)

### Manual Schema Application

> [!WARNING]
> Manual application of `postgres_schema.sql` may bypass migration tracking. It is recommended to let Pangolin handle initialization automatically.

If you must apply the schema manually:

```bash
# Download the current consolidated schema
curl -O https://raw.githubusercontent.com/AlexMercedCoder/Pangolin/main/pangolin/pangolin_store/sql/postgres_schema.sql

# Apply to your database
psql -h localhost -U pangolin -d pangolin -f postgres_schema.sql
```

## Performance Tuning

### Recommended PostgreSQL Settings

```sql
-- For production workloads
ALTER SYSTEM SET shared_buffers = '4GB';
ALTER SYSTEM SET effective_cache_size = '12GB';
ALTER SYSTEM SET maintenance_work_mem = '1GB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '16MB';
ALTER SYSTEM SET default_statistics_target = 100;
ALTER SYSTEM SET random_page_cost = 1.1;  -- For SSD storage
ALTER SYSTEM SET effective_io_concurrency = 200;  -- For SSD storage
ALTER SYSTEM SET work_mem = '64MB';
ALTER SYSTEM SET min_wal_size = '1GB';
ALTER SYSTEM SET max_wal_size = '4GB';

-- Reload configuration
SELECT pg_reload_conf();
```

### Monitoring

```sql
-- Check active connections
SELECT count(*) FROM pg_stat_activity WHERE datname = 'pangolin';

-- Check table sizes
SELECT 
  schemaname,
  tablename,
  pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

-- Check index usage
SELECT 
  schemaname,
  tablename,
  indexname,
  idx_scan,
  idx_tup_read,
  idx_tup_fetch
FROM pg_stat_user_indexes
ORDER BY idx_scan DESC;
```

## Backup and Restore

### Automated Backups (RDS)

AWS RDS provides automated backups:
- Daily snapshots
- Point-in-time recovery
- Cross-region replication

### Manual Backups

```bash
# Backup
pg_dump -h localhost -U pangolin -d pangolin -F c -f pangolin_backup.dump

# Restore
pg_restore -h localhost -U pangolin -d pangolin -c pangolin_backup.dump

# Backup to SQL (human-readable)
pg_dump -h localhost -U pangolin -d pangolin > pangolin_backup.sql

# Restore from SQL
psql -h localhost -U pangolin -d pangolin < pangolin_backup.sql
```

## High Availability

### Replication

```sql
-- On primary server
ALTER SYSTEM SET wal_level = 'replica';
ALTER SYSTEM SET max_wal_senders = 3;
ALTER SYSTEM SET wal_keep_size = '1GB';

-- Create replication user
CREATE ROLE replicator WITH REPLICATION LOGIN PASSWORD 'repl_password';

-- On replica server
# In recovery.conf or postgresql.auto.conf
primary_conninfo = 'host=primary_host port=5432 user=replicator password=repl_password'
```

### Connection Pooling

Use PgBouncer for connection pooling:

```ini
# pgbouncer.ini
[databases]
pangolin = host=localhost port=5432 dbname=pangolin

[pgbouncer]
listen_port = 6432
listen_addr = *
auth_type = md5
auth_file = /etc/pgbouncer/userlist.txt
pool_mode = transaction
max_client_conn = 1000
default_pool_size = 25
```

Then connect to PgBouncer instead:
```bash
DATABASE_URL=postgresql://pangolin:password@localhost:6432/pangolin
```

## Troubleshooting

### Connection Issues

```bash
# Test connection
psql -h localhost -U pangolin -d pangolin -c "SELECT version();"

# Check if PostgreSQL is running
sudo systemctl status postgresql

# Check logs
sudo tail -f /var/log/postgresql/postgresql-16-main.log
```

### Performance Issues

```sql
-- Find slow queries
SELECT 
  pid,
  now() - query_start AS duration,
  query,
  state
FROM pg_stat_activity
WHERE state != 'idle'
  AND now() - query_start > interval '5 seconds'
ORDER BY duration DESC;

-- Analyze table statistics
ANALYZE VERBOSE;

-- Rebuild indexes
REINDEX DATABASE pangolin;
```

## Security Best Practices

1. **Use SSL/TLS**: Always use `sslmode=require` in production
2. **Strong Passwords**: Use long, random passwords
3. **Least Privilege**: Grant only necessary permissions
4. **Network Security**: Use VPC/security groups to restrict access
5. **Regular Updates**: Keep PostgreSQL updated
6. **Audit Logging**: Enable PostgreSQL audit logging
7. **Encryption at Rest**: Use encrypted storage (EBS encryption for RDS)

## Migration from Other Backends

### From SQLite

```bash
# Export from SQLite
sqlite3 pangolin.db .dump > sqlite_dump.sql

# Convert to PostgreSQL format (manual editing required)
# Then import:
psql -h localhost -U pangolin -d pangolin -f converted_dump.sql
```

### From MongoDB

Custom migration script required. Contact support or see migration guide.

## Additional Resources

- [PostgreSQL Official Documentation](https://www.postgresql.org/docs/)
- [AWS RDS PostgreSQL](https://aws.amazon.com/rds/postgresql/)
- [Azure Database for PostgreSQL](https://azure.microsoft.com/en-us/services/postgresql/)
- [Google Cloud SQL for PostgreSQL](https://cloud.google.com/sql/docs/postgres)
- [PostgreSQL Performance Tuning](https://wiki.postgresql.org/wiki/Performance_Optimization)

## Next Steps

- [MongoDB Backend](mongodb.md)
- [SQLite Backend](sqlite.md)
- [Backend Comparison](comparison.md)
- [Warehouse Storage](../warehouse/README.md)
