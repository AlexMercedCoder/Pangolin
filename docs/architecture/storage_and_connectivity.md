# Storage Backend Connectivity & Integration Guide

## Overview

This document explains how Pangolin's storage backends connect to object storage (S3/MinIO), how credential vending works, and common pitfalls to avoid. It consolidates lessons learned from debugging PostgresStore integration issues.

## Architecture Overview

```mermaid
graph TD
    A[PyIceberg Client] -->|1. Request Table| B[Iceberg REST API]
    B -->|2. Authenticate| C[Auth Middleware]
    C -->|3. Load Metadata| D[Storage Backend]
    D -->|4. Read from S3| E[Object Store]
    B -->|5. Vend Credentials| F[Warehouse Config]
    F -->|6. Return Creds| A
    A -->|7. Write Data| E
```

### Key Components

1. **Storage Backend** (`CatalogStore` trait): PostgreSQL, MongoDB, SQLite, or Memory
2. **Object Store Factory**: Creates S3/Azure/GCS clients from warehouse config
3. **Credential Vending**: Returns temporary credentials to PyIceberg
4. **Warehouse Configuration**: Stores S3 endpoint, bucket, and access credentials

## Critical Configuration Requirements

### 1. Environment Variables for Testing

For PostgresStore (and other backends) to work with PyIceberg integration tests:

```bash
# Database Connection
export DATABASE_URL=postgres://pangolin:password@localhost:5432/pangolin
export PANGOLIN_STORAGE_TYPE=postgres

# Authentication (REQUIRED for Basic Auth)
export PANGOLIN_ROOT_USER=admin
export PANGOLIN_ROOT_PASSWORD=password

# S3/MinIO Configuration
export AWS_ACCESS_KEY_ID=minioadmin
export AWS_SECRET_ACCESS_KEY=minioadmin
export AWS_REGION=us-east-1
export AWS_ENDPOINT_URL=http://localhost:9000
export S3_ENDPOINT=http://localhost:9000
```

> [!IMPORTANT]
> **Basic Auth requires BOTH `PANGOLIN_ROOT_USER` and `PANGOLIN_ROOT_PASSWORD`**. Without these, the auth middleware will reject all Basic Auth requests, even if a tenant admin user exists in the database.

### 2. Warehouse Configuration Format

Warehouse `storage_config` must use specific key names:

```json
{
  "type": "s3",
  "s3.bucket": "postgres-bucket",
  "s3.region": "us-east-1",
  "s3.endpoint": "http://localhost:9000",
  "s3.access-key-id": "minioadmin",
  "s3.secret-access-key": "minioadmin"
}
```

> [!CAUTION]
> **Key names MUST match exactly**: `s3.access-key-id` and `s3.secret-access-key`, NOT `access_key_id` or `AWS_ACCESS_KEY_ID`. The code extracts credentials using these exact keys.

### 3. Bucket Naming Convention

Each backend should use its own bucket in MinIO:

- **MemoryStore**: `memory-bucket`
- **SqliteStore**: `sqlite-bucket`
- **MongoStore**: `mongo-bucket`
- **PostgresStore**: `postgres-bucket`

> [!WARNING]
> Do NOT use a generic `test-bucket` unless it actually exists in MinIO. The test script's `BUCKET_NAME` environment variable must match an existing bucket.

## Common Issues & Solutions

### Issue 1: "Missing or invalid authorization header"

**Symptoms**: All API requests return 401, even with valid credentials

**Root Causes**:
1. `PANGOLIN_ROOT_USER` and `PANGOLIN_ROOT_PASSWORD` not set
2. Using Basic Auth for tenant users (only works for Root)
3. Tenant users must use Bearer tokens from `/api/v1/users/login`

**Solution**:
```bash
# For Root user Basic Auth
export PANGOLIN_ROOT_USER=admin
export PANGOLIN_ROOT_PASSWORD=password

# For tenant users, get a token
TOKEN=$(curl -X POST http://localhost:8080/api/v1/users/login \
  -H "Content-Type: application/json" \
  -d '{"username":"tenant_admin","password":"password123"}' \
  | jq -r '.token')

curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/v1/catalogs
```

**Code Reference**: [auth_middleware.rs:L174-210](file:///home/alexmerced/development/personal/Personal/2026/pangolin/pangolin/pangolin_api/src/auth_middleware.rs#L174-210)

### Issue 2: "NoSuchBucket" errors

**Symptoms**: S3 operations fail with "The specified bucket does not exist"

**Root Causes**:
1. Bucket name mismatch between test config and MinIO
2. `object_store_factory` parsing bucket from S3 path instead of using config
3. Bucket not created in MinIO

**Solution**:
```bash
# Verify bucket exists
docker exec pangolin-minio-1 mc ls minio/

# Create bucket if needed
docker exec pangolin-minio-1 mc mb minio/postgres-bucket --ignore-existing

# Ensure test uses correct bucket name
export BUCKET_NAME=postgres-bucket
```

**Code Fix Applied**: object_store_factory.rs:L25-36 = /pangolin/pangolin_store/src/object_store_factory.rs#L25-36

```rust
// ✅ CORRECT: Use bucket from config first
let bucket = config
    .get("s3.bucket")
    .map(|s| s.as_str())
    .or_else(|| {
        // Fallback: parse from location
        location.strip_prefix("s3://")
            .and_then(|rest| rest.split_once('/'))
            .map(|(b, _)| b)
    })
    .ok_or_else(|| anyhow::anyhow!("Cannot determine S3 bucket"))?;

// ❌ WRONG: Always parsing from path
let (bucket, _) = location.strip_prefix("s3://").unwrap().split_once('/').unwrap();
```

### Issue 3: "INVALID_ACCESS_KEY_ID" when PyIceberg writes data

**Symptoms**: Table creation succeeds, but data writes fail with invalid credentials

**Root Cause**: Credential vending code looks for wrong key names in warehouse config

**Solution**: Fix key extraction in `iceberg_handlers.rs`

```rust
// ✅ CORRECT: Match warehouse config keys
let access_key = warehouse.storage_config.get("s3.access-key-id").cloned();
let secret_key = warehouse.storage_config.get("s3.secret-access-key").cloned();

// ❌ WRONG: These keys don't exist in config
let access_key = warehouse.storage_config.get("access_key_id").cloned();
let secret_key = warehouse.storage_config.get("secret_access_key").cloned();
```

**Code Reference**: [iceberg_handlers.rs:L981-982](file:///home/alexmerced/development/personal/Personal/2026/pangolin/pangolin/pangolin_api/src/iceberg_handlers.rs#L981-982)

### Issue 4: "Invalid JSON: expected value at line 1 column 1"

**Symptoms**: Metadata read operations fail

**Root Causes**:
1. Metadata file was never written to S3 (write failed silently)
2. S3 connectivity issues (wrong endpoint, credentials)
3. Empty response from S3 being parsed as JSON

**Debugging Steps**:
```bash
# 1. Check if file exists in MinIO
docker exec pangolin-minio-1 mc ls minio/postgres-bucket/cat_vended/ns1/table1/metadata/

# 2. Try to read the file
docker exec pangolin-minio-1 mc cat minio/postgres-bucket/cat_vended/ns1/table1/metadata/00000-*.metadata.json

# 3. Test S3 connectivity from Python
python3 -c "
import boto3
s3 = boto3.client('s3', 
    endpoint_url='http://localhost:9000',
    aws_access_key_id='minioadmin',
    aws_secret_access_key='minioadmin')
print('Buckets:', [b['Name'] for b in s3.list_buckets()['Buckets']])
"

# 4. Check API logs for write errors
grep -i "Failed to write\|ERROR" api_postgres.log
```

### Issue 5: PostgreSQL Permission Errors

**Symptoms**: API fails to start with "permission denied for schema public"

**Root Cause**: The `pangolin` database user lacks permissions to create tables

**Solution**:
```bash
docker exec pangolin-postgres psql -U postgres -d pangolin -c \
  "GRANT ALL ON SCHEMA public TO pangolin; \
   GRANT ALL ON ALL TABLES IN SCHEMA public TO pangolin; \
   GRANT ALL ON ALL SEQUENCES IN SCHEMA public TO pangolin;"
```

## Credential Vending Flow

### How It Works

1. **PyIceberg sends `X-Iceberg-Access-Delegation: vended-credentials` header**
2. **API's `load_table` endpoint**:
   - Retrieves warehouse config for the catalog
   - Extracts `s3.access-key-id` and `s3.secret-access-key`
   - Returns credentials in `TableResponse.config`
3. **PyIceberg uses vended credentials** to read/write data directly to S3

### Code Flow

```rust
// 1. Extract credentials from warehouse config
let credentials = match store.get_catalog(tenant_id, catalog_name).await {
    Ok(Some(catalog)) => {
        if let Some(warehouse_name) = catalog.warehouse_name {
            match store.get_warehouse(tenant_id, warehouse_name).await {
                Ok(Some(warehouse)) => {
                    let access_key = warehouse.storage_config
                        .get("s3.access-key-id").cloned();
                    let secret_key = warehouse.storage_config
                        .get("s3.secret-access-key").cloned();
                    
                    if let (Some(ak), Some(sk)) = (access_key, secret_key) {
                        Some((ak, sk))
                    } else {
                        None
                    }
                },
                _ => None
            }
        } else {
            None
        }
    },
    _ => None
};

// 2. Return credentials in response
TableResponse::with_credentials(location, metadata, credentials)
```

### Vending Strategy Configuration

The warehouse's `vending_strategy` field determines how credentials are generated:

```json
{
  "vending_strategy": {
    "AwsStatic": {
      "access_key_id": "minioadmin",
      "secret_access_key": "minioadmin"
    }
  }
}
```

> [!NOTE]
> For MinIO/local testing, `AwsStatic` is sufficient. Production should use `AwsSts` for temporary credentials.

## Testing Checklist

Before running PyIceberg integration tests against a new backend:

- [ ] MinIO container is running (`docker ps | grep minio`)
- [ ] Bucket exists (`docker exec pangolin-minio-1 mc ls minio/`)
- [ ] Database is accessible (for PostgreSQL/MongoDB)
- [ ] Database user has proper permissions
- [ ] `PANGOLIN_ROOT_USER` and `PANGOLIN_ROOT_PASSWORD` are set
- [ ] S3 environment variables are set (`AWS_ACCESS_KEY_ID`, `S3_ENDPOINT`)
- [ ] Warehouse config uses correct key names (`s3.access-key-id`)
- [ ] `BUCKET_NAME` environment variable matches existing bucket
- [ ] API starts successfully and listens on port 8080

## Network Considerations

### Host vs Docker Network

- **API running on host**: Use `localhost:9000` for MinIO endpoint
- **API running in Docker**: Use `minio:9000` (Docker network hostname)
- **PyIceberg on host**: Always uses `localhost:9000`

> [!TIP]
> For local development, run the API on the host (not in Docker) to avoid network complexity. Use `localhost` for all endpoints.

## Debugging Workflow

When a backend fails PyIceberg tests:

1. **Verify Authentication**
   ```bash
   curl -u admin:password http://localhost:8080/api/v1/tenants
   ```
   - If fails: Check `PANGOLIN_ROOT_USER`/`PASSWORD` env vars

2. **Verify S3 Connectivity**
   ```bash
   docker exec pangolin-minio-1 mc ls minio/
   ```
   - If fails: Start MinIO container

3. **Verify Bucket Exists**
   ```bash
   docker exec pangolin-minio-1 mc mb minio/postgres-bucket --ignore-existing
   ```

4. **Check Warehouse Config**
   ```sql
   SELECT storage_config FROM warehouses WHERE name='test-warehouse';
   ```
   - Verify keys are `s3.access-key-id` not `access_key_id`

5. **Test Credential Vending**
   ```bash
   curl -H "Authorization: Bearer $TOKEN" \
     "http://localhost:8080/v1/cat_vended/v1/config" | jq '.defaults'
   ```
   - Should include `X-Iceberg-Access-Delegation: vended-credentials`

6. **Check API Logs**
   ```bash
   grep -E "ERROR|Failed|panic" api_postgres.log
   ```

## Key Files Reference

| File | Purpose | Common Issues |
|------|---------|---------------|
| [auth_middleware.rs](file:///home/alexmerced/development/personal/Personal/2026/pangolin/pangolin/pangolin_api/src/auth_middleware.rs) | Authentication logic | Basic Auth only works for Root users |
| [iceberg_handlers.rs](file:///home/alexmerced/development/personal/Personal/2026/pangolin/pangolin/pangolin_api/src/iceberg_handlers.rs) | Iceberg REST endpoints | Credential vending key names |
| [object_store_factory.rs](file:///home/alexmerced/development/personal/Personal/2026/pangolin/pangolin/pangolin_store/src/object_store_factory.rs) | S3 client creation | Bucket extraction logic |
| [postgres.rs](file:///home/alexmerced/development/personal/Personal/2026/pangolin/pangolin/pangolin_store/src/postgres.rs) | PostgreSQL backend | `write_file`/`read_file` implementations |
| [test_pyiceberg_matrix.py](file:///home/alexmerced/development/personal/Personal/2026/pangolin/tests/integration/test_pyiceberg_matrix.py) | Integration tests | Environment variable configuration |

## Lessons Learned

### 1. Configuration Key Consistency

**Problem**: Different parts of the codebase used different key names for the same configuration.

**Solution**: Standardize on warehouse config format (`s3.access-key-id`) and update all code to use these exact keys.

> [!NOTE]
> **Current Limitation**: The credential vending logic in `iceberg_handlers.rs` currently only extracts S3 credentials (`s3.access-key-id` and `s3.secret-access-key`). While `object_store_factory.rs` supports Azure (`azure.account-name`, `azure.account-key`) and GCP (`gcp.service-account`), the `load_table` and `create_table` endpoints don't yet extract and vend these credentials. This means:
> - ✅ S3/MinIO: Full credential vending support
> - ⚠️ Azure/GCP: Object store factory works, but credentials must be client-provided (not vended)
>
> To add Azure/GCP credential vending, update `iceberg_handlers.rs:L976-997` to detect storage type and extract appropriate credentials.

### 2. Silent Errors

**Problem**: Errors in `create_user` and S3 operations were swallowed with `Err(_) => log::debug!()`.

**Solution**: Always log errors at appropriate levels and propagate them when they indicate real failures.

### 3. Environment Variable Dependencies

**Problem**: Tests worked for some backends but not others due to missing environment variables.

**Solution**: Document ALL required environment variables and create a comprehensive testing checklist.

### 4. Network Hostname Confusion

**Problem**: Using `localhost` vs Docker network hostnames caused connectivity issues.

**Solution**: For local development, run everything on the host network to avoid Docker networking complexity.

### 5. Bucket Naming Assumptions

**Problem**: Tests assumed a `test-bucket` existed, but each backend had its own bucket.

**Solution**: Use backend-specific bucket names and make `BUCKET_NAME` configurable via environment variable.

## Related Documentation

- [Authentication Architecture](file:///home/alexmerced/development/personal/Personal/2026/pangolin/docs/architecture/authentication.md)
- [Warehouse Configuration](file:///home/alexmerced/development/personal/Personal/2026/pangolin/docs/warehouse/README.md)
- [PyIceberg Integration](file:///home/alexmerced/development/personal/Personal/2026/pangolin/docs/pyiceberg/README.md)
- [Security & Vending](file:///home/alexmerced/development/personal/Personal/2026/pangolin/docs/features/security_vending.md)
