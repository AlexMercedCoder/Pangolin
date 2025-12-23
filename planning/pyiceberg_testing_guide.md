# PyIceberg Integration Testing Guide

**Status:** ✅ **COMPLETED** (2025-12-22)  
**Verified Backends:** All 4 (MemoryStore, SqliteStore, MongoStore, PostgresStore)  
**Multi-Cloud Support:** ✅ S3, Azure, GCS

> [!NOTE]
> This guide documents the testing process. For current status and results, see [pyiceberg_integration_status.md](file:///home/alexmerced/development/personal/Personal/2026/pangolin/planning/pyiceberg_integration_status.md).

This guide outlines the process for running integration tests to verify PyIceberg compatibility with Pangolin across all supported storage backends.

## Prerequisites

*   **Docker**: Required for MinIO, Postgres, and MongoDB.
*   **Python 3.8+**: Required for running the test script.
*   **Rust Toolchain**: Required for building and running the Pangolin API.

## 1. Infrastructure Setup

Ensure the necessary services are running via Docker.

```bash
# Start MinIO, Postgres, and Mongo
docker-compose up -d minio pangolin-postgres pangolin-mongo
```

### MinIO Configuration
The test script assumes a local MinIO instance at `http://localhost:9000` with:
*   **User**: `minioadmin`
*   **Password**: `minioadmin`
*   **Buckets**: `memory-bucket`, `sqlite-bucket`, `postgres-bucket`, `mongo-bucket`.

Create buckets:
```bash
docker exec pangolin-minio-1 mc mb minio/memory-bucket --ignore-existing
docker exec pangolin-minio-1 mc mb minio/sqlite-bucket --ignore-existing
docker exec pangolin-minio-1 mc mb minio/postgres-bucket --ignore-existing
docker exec pangolin-minio-1 mc mb minio/mongo-bucket --ignore-existing
```

## 2. Test Script

The integration test script is located at: `tests/integration/test_pyiceberg_matrix.py`.

It performs the following for a given backend:
1.  **Setup**: Creates a Tenant, Tenant Admin, and specific Warehouse/Catalog configurations.
2.  **Vended Credentials Test**:
    *   Create Namespace/Table.
    *   Write Data (Append).
    *   Read Data.
    *   Update Schema (Add column) - Expected to fail with 409 (known limitation).
3.  **Client Credentials Test**:
    *   Repeats the above using directly provided S3 credentials.

## 3. Running Tests by Backend

You must run the tests sequentially for each backend, restarting the Pangolin API with the appropriate configuration each time.

### A. MemoryStore

1.  **Start API**:
    ```bash
    export PANGOLIN_STORAGE_TYPE=memory
    export PANGOLIN_ROOT_USER=admin
    export PANGOLIN_ROOT_PASSWORD=password
    export AWS_ACCESS_KEY_ID=minioadmin
    export AWS_SECRET_ACCESS_KEY=minioadmin
    export AWS_REGION=us-east-1
    export AWS_ENDPOINT_URL=http://localhost:9000
    export S3_ENDPOINT=http://localhost:9000
    
    cargo run --bin pangolin_api
    ```

2.  **Run Test**:
    ```bash
    export BACKEND_NAME=MemoryStore
    export BUCKET_NAME=memory-bucket
    export MINIO_URL=http://localhost:9000
    
    python3 tests/integration/test_pyiceberg_matrix.py
    ```

### B. SQLiteStore

1.  **Start API**:
    ```bash
    export PANGOLIN_STORAGE_TYPE=sqlite
    export DATABASE_URL=sqlite://pangolin.db
    export PANGOLIN_ROOT_USER=admin
    export PANGOLIN_ROOT_PASSWORD=password
    # ... same AWS vars as above ...
    
    cargo run --bin pangolin_api
    ```

2.  **Run Test**:
    ```bash
    export BACKEND_NAME=SQLiteStore
    export BUCKET_NAME=sqlite-bucket
    
    python3 tests/integration/test_pyiceberg_matrix.py
    ```

### C. PostgresStore

**IMPORTANT**: PostgresStore requires `PANGOLIN_ROOT_USER` and `PANGOLIN_ROOT_PASSWORD` for Basic Auth.

1.  **Prepare Database**:
    ```bash
    docker exec pangolin-postgres psql -U postgres -c "DROP DATABASE IF EXISTS pangolin;"
    docker exec pangolin-postgres psql -U postgres -c "CREATE DATABASE pangolin OWNER pangolin;"
    ```

2.  **Start API**:
    ```bash
    export PANGOLIN_STORAGE_TYPE=postgres
    export DATABASE_URL=postgres://pangolin:password@localhost:5432/pangolin
    export PANGOLIN_ROOT_USER=admin
    export PANGOLIN_ROOT_PASSWORD=password
    # ... same AWS vars ...
    
    cargo run --bin pangolin_api
    ```

3.  **Run Test**:
    ```bash
    export BACKEND_NAME=PostgresStore
    export BUCKET_NAME=postgres-bucket
    
    python3 tests/integration/test_pyiceberg_matrix.py
    ```

### D. MongoStore

1.  **Start API**:
    ```bash
    export PANGOLIN_STORAGE_TYPE=mongo
    export DATABASE_URL=mongodb://root:example@localhost:27017
    export MONGO_DB_NAME=pangolin_test
    export PANGOLIN_ROOT_USER=admin
    export PANGOLIN_ROOT_PASSWORD=password
    # ... same AWS vars ...
    
    cargo run --bin pangolin_api
    ```

2.  **Run Test**:
    ```bash
    export BACKEND_NAME=MongoStore
    export BUCKET_NAME=mongo-bucket
    
    python3 tests/integration/test_pyiceberg_matrix.py
    ```

## 4. Expected Results

### ✅ Passing Tests
- Tenant creation
- Warehouse creation
- Catalog creation
- Namespace creation
- Table creation
- Data write (append)
- Data read

### ⚠️ Known Failures
- **Schema Update**: Fails with 409 conflict due to PyIceberg client caching
  - This is expected and not a server issue
  - Workaround: Call `table.refresh()` between operations

## 5. Troubleshooting

### Authentication Errors
**Issue**: "Missing or invalid authorization header"  
**Solution**: Ensure `PANGOLIN_ROOT_USER` and `PANGOLIN_ROOT_PASSWORD` are set

### Connection Refused
**Issue**: Tests fail with connection refused  
**Solution**: Wait longer for API to start (PostgresStore takes ~10-15 seconds)

### Invalid JSON Errors
**Issue**: "Invalid JSON: expected value at line 1 column 1"  
**Solution**: Ensure MinIO is running and bucket exists

### NoSuchBucket
**Issue**: S3 operations fail with "The specified bucket does not exist"  
**Solution**: Create the bucket using `mc mb` command above

## 6. Multi-Cloud Testing

To test Azure or GCS credential vending:

1. Update warehouse `storage_config` with appropriate credentials
2. Verify properties are extracted correctly in API logs
3. Check `TableResponse.config` includes correct property names

See [storage_and_connectivity.md](file:///home/alexmerced/development/personal/Personal/2026/pangolin/docs/architecture/storage_and_connectivity.md) for detailed multi-cloud configuration.
