# Database Migrations for Enhanced Audit Logging

This directory contains database migration scripts for upgrading the audit logging system to support enhanced features including type-safe enums, comprehensive user tracking, request context, and structured metadata.

## Overview

The enhanced audit logging system introduces the following improvements:

- **Type-Safe Enums**: Actions and resource types are now enums instead of free-form strings
- **User Tracking**: Separate `user_id` and `username` fields for better user identification
- **Request Context**: IP address and user agent tracking
- **Result Tracking**: Explicit success/failure status
- **Error Messages**: Capture error details for failed operations
- **Structured Metadata**: JSON metadata for additional context

## Migration Files

### PostgreSQL
- **File**: `postgres/001_enhanced_audit_logging.sql`
- **Description**: Updates PostgreSQL audit_logs table schema
- **Features**: Full-text search on metadata, GIN indexes, foreign key constraints

### SQLite
- **File**: `sqlite/001_enhanced_audit_logging.sql`
- **Description**: Updates SQLite audit_logs table schema
- **Features**: Check constraints, indexes for common queries

### MongoDB
MongoDB migrations are handled programmatically through the application code since MongoDB is schema-less. The enhanced structure is automatically applied when logging new audit events.

## Running Migrations

### PostgreSQL

```bash
# Using psql
psql -U postgres -d pangolin < migrations/postgres/001_enhanced_audit_logging.sql

# Or using a migration tool like sqlx
sqlx migrate run --database-url "postgresql://user:pass@localhost/pangolin"
```

### SQLite

```bash
# Using sqlite3
sqlite3 pangolin.db < migrations/sqlite/001_enhanced_audit_logging.sql

# Or using sqlx
sqlx migrate run --database-url "sqlite://pangolin.db"
```

### MongoDB

No migration needed. The enhanced schema is automatically applied when new audit events are logged.

## Schema Changes

### Old Schema
```sql
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    actor VARCHAR(255),
    action VARCHAR(255),
    resource VARCHAR(500),
    details TEXT
);
```

### New Schema
```sql
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    user_id UUID,                    -- NEW
    username VARCHAR(255) NOT NULL,  -- RENAMED from actor
    action VARCHAR(100) NOT NULL,    -- NOW ENUM
    resource_type VARCHAR(50) NOT NULL,  -- NEW
    resource_id UUID,                -- NEW
    resource_name VARCHAR(500) NOT NULL,  -- RENAMED from resource
    timestamp TIMESTAMPTZ NOT NULL,
    ip_address VARCHAR(45),          -- NEW
    user_agent TEXT,                 -- NEW
    result VARCHAR(20) NOT NULL,     -- NEW
    error_message TEXT,              -- NEW
    metadata JSONB,                  -- RENAMED from details, now JSONB
    
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);
```

## Data Migration

The migration scripts automatically migrate existing data from the old schema to the new schema:

- `actor` → `username`
- `resource` → `resource_name`
- `details` → `metadata` (wrapped in JSON object with key "legacy_details")
- New fields are set to sensible defaults:
  - `user_id`: NULL
  - `resource_type`: "Unknown"
  - `resource_id`: NULL
  - `result`: "Success"
  - `ip_address`: NULL
  - `user_agent`: NULL
  - `error_message`: NULL

## Indexes

The following indexes are created for optimal query performance:

1. **Composite Index**: `(tenant_id, timestamp DESC)` - Primary query pattern
2. **User Index**: `(user_id)` - Filter by user
3. **Action Index**: `(action)` - Filter by action type
4. **Resource Type Index**: `(resource_type)` - Filter by resource type
5. **Resource Composite**: `(resource_type, resource_id)` - Filter by specific resource
6. **Result Index**: `(result)` - Filter by success/failure
7. **Timestamp Index**: `(timestamp DESC)` - Time-based queries
8. **Metadata Index** (PostgreSQL only): GIN index on JSONB for metadata queries

## Rollback

If you need to rollback the migration:

### PostgreSQL
```sql
-- Drop new table
DROP TABLE IF EXISTS audit_logs;

-- Restore old table
ALTER TABLE audit_logs_old RENAME TO audit_logs;
```

### SQLite
```sql
-- Drop new table
DROP TABLE IF EXISTS audit_logs;

-- Restore old table
ALTER TABLE audit_logs_old RENAME TO audit_logs;
```

## Verification

After running the migration, verify the data:

```sql
-- Check total count
SELECT COUNT(*) FROM audit_logs;

-- Check action distribution
SELECT action, COUNT(*) 
FROM audit_logs 
GROUP BY action 
ORDER BY COUNT(*) DESC;

-- Check resource type distribution
SELECT resource_type, COUNT(*) 
FROM audit_logs 
GROUP BY resource_type 
ORDER BY COUNT(*) DESC;

-- Check result distribution
SELECT result, COUNT(*) 
FROM audit_logs 
GROUP BY result;

-- Check for NULL values in required fields
SELECT 
    COUNT(*) FILTER (WHERE username IS NULL) as null_username,
    COUNT(*) FILTER (WHERE action IS NULL) as null_action,
    COUNT(*) FILTER (WHERE resource_name IS NULL) as null_resource_name,
    COUNT(*) FILTER (WHERE result IS NULL) as null_result
FROM audit_logs;
```

## Cleanup

After verifying the migration was successful, you can drop the backup table:

```sql
-- PostgreSQL/SQLite
DROP TABLE IF EXISTS audit_logs_old;
```

**⚠️ Warning**: Only drop the backup table after thoroughly verifying the migration!

## Troubleshooting

### Foreign Key Constraint Errors

If you encounter foreign key constraint errors during migration:

1. Temporarily disable foreign key checks:
   ```sql
   -- PostgreSQL
   SET CONSTRAINTS ALL DEFERRED;
   
   -- SQLite
   PRAGMA foreign_keys = OFF;
   ```

2. Run the migration

3. Re-enable foreign key checks:
   ```sql
   -- PostgreSQL
   SET CONSTRAINTS ALL IMMEDIATE;
   
   -- SQLite
   PRAGMA foreign_keys = ON;
   ```

### Performance Issues

If the migration is slow on large datasets:

1. Run the migration during off-peak hours
2. Consider batching the data migration:
   ```sql
   INSERT INTO audit_logs (...)
   SELECT ...
   FROM audit_logs_old
   WHERE timestamp >= '2024-01-01'
   AND timestamp < '2024-02-01';
   ```

3. Create indexes after data migration completes

## Support

For issues or questions about the migration:
1. Check the migration logs for errors
2. Verify database permissions
3. Ensure foreign key references exist (tenants, users tables)
4. Contact the development team

## Version History

- **v1.0.0** (2025-12-18): Initial enhanced audit logging migration
  - Added type-safe enums
  - Added user tracking
  - Added request context
  - Added result tracking
  - Added structured metadata
