# Enhanced Audit Logging - Deployment Guide

**Version**: 1.0.0  
**Date**: December 18, 2025  
**Status**: Production Ready ✅

---

## 📋 Table of Contents

1. [Pre-Deployment Checklist](#pre-deployment-checklist)
2. [Database Migrations](#database-migrations)
3. [API Integration](#api-integration)
4. [Testing](#testing)
5. [Production Deployment](#production-deployment)
6. [Monitoring & Verification](#monitoring--verification)
7. [Rollback Procedures](#rollback-procedures)
8. [Troubleshooting](#troubleshooting)

---

## 🔍 Pre-Deployment Checklist

Before deploying the enhanced audit logging system, ensure:

- [ ] All code changes are committed and reviewed
- [ ] Database backup is created
- [ ] Test environment is available
- [ ] Deployment window is scheduled (recommended: off-peak hours)
- [ ] Rollback plan is documented
- [ ] Team is notified of deployment

---

## 🗄️ Database Migrations

### Step 1: Backup Current Database

**PostgreSQL**:
```bash
pg_dump -U postgres -d pangolin > pangolin_backup_$(date +%Y%m%d_%H%M%S).sql
```

**SQLite**:
```bash
cp pangolin.db pangolin_backup_$(date +%Y%m%d_%H%M%S).db
```

**MongoDB**:
```bash
mongodump --db pangolin --out=pangolin_backup_$(date +%Y%m%d_%H%M%S)
```

### Step 2: Run Migrations

#### PostgreSQL

```bash
# Navigate to migrations directory
cd migrations/postgres

# Review the migration script
cat 001_enhanced_audit_logging.sql

# Run the migration
psql -U postgres -d pangolin < 001_enhanced_audit_logging.sql

# Verify the migration
psql -U postgres -d pangolin -c "SELECT COUNT(*) FROM audit_logs;"
psql -U postgres -d pangolin -c "SELECT action, COUNT(*) FROM audit_logs GROUP BY action LIMIT 10;"
```

#### SQLite

```bash
# Navigate to migrations directory
cd migrations/sqlite

# Review the migration script
cat 001_enhanced_audit_logging.sql

# Run the migration
sqlite3 pangolin.db < 001_enhanced_audit_logging.sql

# Verify the migration
sqlite3 pangolin.db "SELECT COUNT(*) FROM audit_logs;"
sqlite3 pangolin.db "SELECT action, COUNT(*) FROM audit_logs GROUP BY action LIMIT 10;"
```

#### MongoDB

No migration needed - the enhanced schema is automatically applied when new audit events are logged.

### Step 3: Verify Migration Success

Run these verification queries:

**PostgreSQL/SQLite**:
```sql
-- Check table structure
SELECT column_name, data_type 
FROM information_schema.columns 
WHERE table_name = 'audit_logs';

-- Check for NULL values in required fields
SELECT 
    COUNT(*) FILTER (WHERE username IS NULL) as null_username,
    COUNT(*) FILTER (WHERE action IS NULL) as null_action,
    COUNT(*) FILTER (WHERE resource_name IS NULL) as null_resource_name,
    COUNT(*) FILTER (WHERE result IS NULL) as null_result
FROM audit_logs;

-- Check index creation
SELECT indexname FROM pg_indexes WHERE tablename = 'audit_logs';
```

**Expected Results**:
- All required fields should have 0 NULL values
- At least 7 indexes should be created
- Old data should be migrated successfully

---

## 🔌 API Integration

### Step 1: Verify Code Changes

The following files have been updated:

1. **`pangolin_api/src/lib.rs`**:
   - Added `pub mod audit_handlers;`
   - Updated audit routes to use new handlers

2. **`pangolin_api/src/audit_handlers.rs`**:
   - New file with 3 endpoint handlers
   - Supports filtering, pagination, and counting

### Step 2: Compile and Test

```bash
# Compile the API
cargo build --release --manifest-path pangolin/Cargo.toml -p pangolin_api

# Run unit tests
cargo test --manifest-path pangolin/Cargo.toml -p pangolin_store audit

# Run integration tests (if DATABASE_URL is set)
export DATABASE_URL="postgresql://postgres:postgres@localhost/pangolin"
cargo test --manifest-path pangolin/Cargo.toml -p pangolin_store postgres_audit
```

### Step 3: Start the API Server

```bash
# Set environment variables
export DATABASE_URL="postgresql://postgres:postgres@localhost/pangolin"
export PANGOLIN_STORAGE_TYPE="postgres"  # or "mongodb", "sqlite"

# Start the server
cargo run --release --manifest-path pangolin/Cargo.toml -p pangolin_api
```

---

## 🧪 Testing

### Manual API Testing

#### 1. List Audit Events (No Filter)

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
     -H "X-Pangolin-Tenant: YOUR_TENANT_ID" \
     "http://localhost:8080/api/v1/audit"
```

**Expected Response**: JSON array of audit events

#### 2. Filter by Action

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
     -H "X-Pangolin-Tenant: YOUR_TENANT_ID" \
     "http://localhost:8080/api/v1/audit?action=create_table&limit=10"
```

**Expected Response**: JSON array of CreateTable audit events (max 10)

#### 3. Filter by Result

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
     -H "X-Pangolin-Tenant: YOUR_TENANT_ID" \
     "http://localhost:8080/api/v1/audit?result=failure"
```

**Expected Response**: JSON array of failed audit events

#### 4. Filter by Time Range

```bash
START_TIME=$(date -u -d '7 days ago' +%Y-%m-%dT%H:%M:%SZ)
END_TIME=$(date -u +%Y-%m-%dT%H:%M:%SZ)

curl -H "Authorization: Bearer YOUR_TOKEN" \
     -H "X-Pangolin-Tenant: YOUR_TENANT_ID" \
     "http://localhost:8080/api/v1/audit?start_time=$START_TIME&end_time=$END_TIME"
```

**Expected Response**: JSON array of audit events from the last 7 days

#### 5. Pagination

```bash
# First page
curl -H "Authorization: Bearer YOUR_TOKEN" \
     -H "X-Pangolin-Tenant: YOUR_TENANT_ID" \
     "http://localhost:8080/api/v1/audit?limit=50&offset=0"

# Second page
curl -H "Authorization: Bearer YOUR_TOKEN" \
     -H "X-Pangolin-Tenant: YOUR_TENANT_ID" \
     "http://localhost:8080/api/v1/audit?limit=50&offset=50"
```

**Expected Response**: Different sets of 50 audit events

#### 6. Count Events

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
     -H "X-Pangolin-Tenant: YOUR_TENANT_ID" \
     "http://localhost:8080/api/v1/audit/count?action=create_table"
```

**Expected Response**: `{"count": N}` where N is the number of CreateTable events

#### 7. Get Specific Event

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
     -H "X-Pangolin-Tenant: YOUR_TENANT_ID" \
     "http://localhost:8080/api/v1/audit/EVENT_UUID"
```

**Expected Response**: Single audit event object or 404 if not found

### Automated Testing

Run the live test script:

```bash
cd scripts
chmod +x test_audit_logging.sh
./test_audit_logging.sh
```

**Expected Output**: All test scenarios should pass

---

## 🚀 Production Deployment

### Deployment Steps

1. **Stop the API Server**:
   ```bash
   systemctl stop pangolin-api
   # or
   pkill -f pangolin_api
   ```

2. **Run Database Migrations**:
   ```bash
   # Follow steps in "Database Migrations" section above
   ```

3. **Deploy New Code**:
   ```bash
   # Pull latest code
   git pull origin main
   
   # Build release binary
   cargo build --release --manifest-path pangolin/Cargo.toml -p pangolin_api
   
   # Copy binary to deployment location
   cp target/release/pangolin_api /usr/local/bin/
   ```

4. **Start the API Server**:
   ```bash
   systemctl start pangolin-api
   # or
   /usr/local/bin/pangolin_api &
   ```

5. **Verify Deployment**:
   ```bash
   # Check server is running
   curl http://localhost:8080/health
   
   # Test audit endpoint
   curl -H "Authorization: Bearer TOKEN" \
        -H "X-Pangolin-Tenant: TENANT_ID" \
        "http://localhost:8080/api/v1/audit?limit=1"
   ```

### Environment Variables

Ensure these environment variables are set:

```bash
# Database connection
export DATABASE_URL="postgresql://user:pass@host:5432/pangolin"

# Storage type
export PANGOLIN_STORAGE_TYPE="postgres"  # or "mongodb", "sqlite"

# Optional: Logging level
export RUST_LOG="info,pangolin_api=debug"
```

---

## 📊 Monitoring & Verification

### Health Checks

1. **API Health**:
   ```bash
   curl http://localhost:8080/health
   ```
   Expected: `OK`

2. **Database Connectivity**:
   ```bash
   psql -U postgres -d pangolin -c "SELECT 1;"
   ```
   Expected: `1`

3. **Audit Logging**:
   ```bash
   # Create a test event (e.g., create a catalog)
   curl -X POST -H "Authorization: Bearer TOKEN" \
        -H "Content-Type: application/json" \
        -H "X-Pangolin-Tenant: TENANT_ID" \
        -d '{"name":"test_catalog"}' \
        "http://localhost:8080/api/v1/catalogs"
   
   # Verify it was logged
   curl -H "Authorization: Bearer TOKEN" \
        -H "X-Pangolin-Tenant: TENANT_ID" \
        "http://localhost:8080/api/v1/audit?action=create_catalog&limit=1"
   ```
   Expected: Audit event for catalog creation

### Metrics to Monitor

- **Audit Event Count**: Should increase with system activity
- **Query Performance**: Audit queries should complete in <100ms
- **Error Rate**: Should be <0.1% for audit operations
- **Database Size**: Monitor audit_logs table growth

### Performance Benchmarks

Expected performance (with proper indexes):

- List audit events (100 records): <50ms
- Count audit events: <20ms
- Get specific event: <10ms
- Insert audit event: <5ms

---

## 🔄 Rollback Procedures

If issues are encountered, follow these rollback steps:

### Step 1: Stop the API Server

```bash
systemctl stop pangolin-api
```

### Step 2: Restore Database

**PostgreSQL**:
```bash
psql -U postgres -d pangolin < pangolin_backup_TIMESTAMP.sql
```

**SQLite**:
```bash
cp pangolin_backup_TIMESTAMP.db pangolin.db
```

**MongoDB**:
```bash
mongorestore --db pangolin pangolin_backup_TIMESTAMP/pangolin
```

### Step 3: Revert Code Changes

```bash
git revert HEAD  # or specific commit
cargo build --release --manifest-path pangolin/Cargo.toml -p pangolin_api
```

### Step 4: Restart API Server

```bash
systemctl start pangolin-api
```

### Step 5: Verify Rollback

```bash
curl http://localhost:8080/health
curl -H "Authorization: Bearer TOKEN" \
     -H "X-Pangolin-Tenant: TENANT_ID" \
     "http://localhost:8080/api/v1/audit?limit=1"
```

---

## 🔧 Troubleshooting

### Issue: Migration Fails with Foreign Key Error

**Cause**: Referenced tables (tenants, users) don't exist

**Solution**:
```sql
-- Temporarily disable foreign key checks
SET CONSTRAINTS ALL DEFERRED;  -- PostgreSQL
PRAGMA foreign_keys = OFF;     -- SQLite

-- Run migration
-- ...

-- Re-enable foreign key checks
SET CONSTRAINTS ALL IMMEDIATE;  -- PostgreSQL
PRAGMA foreign_keys = ON;       -- SQLite
```

### Issue: API Returns 500 Error

**Cause**: Database connection issue or missing indexes

**Solution**:
1. Check database connectivity:
   ```bash
   psql -U postgres -d pangolin -c "SELECT 1;"
   ```

2. Verify indexes exist:
   ```sql
   SELECT indexname FROM pg_indexes WHERE tablename = 'audit_logs';
   ```

3. Check API logs:
   ```bash
   journalctl -u pangolin-api -n 100
   ```

### Issue: Slow Query Performance

**Cause**: Missing indexes or large dataset

**Solution**:
1. Verify indexes:
   ```sql
   EXPLAIN ANALYZE SELECT * FROM audit_logs WHERE tenant_id = 'UUID' ORDER BY timestamp DESC LIMIT 100;
   ```

2. Add missing indexes if needed
3. Consider partitioning for very large tables (>10M rows)

### Issue: Type Inference Errors in Tests

**Cause**: Missing type annotations

**Solution**: Add explicit type annotations:
```rust
let store: PostgresStore = PostgresStore::new(&connection_string).await?;
```

---

## 📚 Additional Resources

- **API Documentation**: http://localhost:8080/swagger-ui
- **Migration Scripts**: `/migrations/`
- **Test Scripts**: `/scripts/test_audit_logging.sh`
- **Implementation Guide**: `.gemini/brain/.../final_summary.md`

---

## ✅ Post-Deployment Checklist

After deployment, verify:

- [ ] Database migration completed successfully
- [ ] API server is running
- [ ] Health check passes
- [ ] Audit endpoints return data
- [ ] Filtering works correctly
- [ ] Pagination works correctly
- [ ] Performance is acceptable (<100ms for queries)
- [ ] No errors in logs
- [ ] Monitoring is active
- [ ] Team is notified of successful deployment

---

## 🎉 Success Criteria

Deployment is successful when:

1. ✅ All database migrations complete without errors
2. ✅ API compiles and starts successfully
3. ✅ All audit endpoints return expected responses
4. ✅ Query performance meets benchmarks
5. ✅ No errors in application logs
6. ✅ Existing functionality remains unaffected

---

**Deployment completed successfully!** 🚀

For questions or issues, contact the development team.
