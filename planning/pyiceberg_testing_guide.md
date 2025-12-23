# PyIceberg Integration Testing Guide

**Status:** ✅ **COMPLETED** (2025-12-23)  
**Verified Backends:** 3 of 4 (MemoryStore, SqliteStore, MongoStore)  
**Blocked:** PostgresStore (authentication issue)

\u003e [!NOTE]
\u003e This guide documents the testing process. For current status and results, see [pyiceberg_integration_status.md](file:///home/alexmerced/development/personal/Personal/2026/pangolin/planning/pyiceberg_integration_status.md).

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

*Note: The test script or setup instructions usually handle bucket creation, or you can create them manually via the MinIO console.*

## 2. Test Script

The integration test script is located at: `tests/integration/test_pyiceberg_matrix.py`.

It performs the following for a given backend:
1.  **Setup**: Creates a Tenant, Tenant Admin, and specific Warehouse/Catalog configurations.
2.  **Vended Credentials Test**:
    *   Create Namespace/Table.
    *   Write Data (Append).
    *   Read Data.
    *   Update Schema (Add column).
3.  **Client Credentials Test**:
    *   Repeats the above using directly provided S3 credentials.

## 3. Running Tests by Backend

You must run the tests sequentially for each backend, restarting the Pangolin API with the appropriate configuration each time.

### A. MemoryStore

1.  **Start API**:
    ```bash
    export PANGOLIN_STORE_TYPE=memory
    export PANGOLIN_ROOT_USER=admin
    export PANGOLIN_ROOT_PASSWORD=password
    export AWS_ACCESS_KEY_ID=minioadmin
    export AWS_SECRET_ACCESS_KEY=minioadmin
    export AWS_REGION=us-east-1
    export AWS_ENDPOINT_URL=http://localhost:9000
    
    cargo run --bin pangolin_api
    ```

2.  **Run Test**:
    ```bash
    export PANGOLIN_URL=http://localhost:8080
    export MINIO_URL=http://localhost:9000
    export BUCKET_NAME=memory-bucket
    export BACKEND_NAME=MemoryStore
    
    python3 tests/integration/test_pyiceberg_matrix.py
    ```

### B. SQLiteStore

1.  **Start API**:
    ```bash
    export PANGOLIN_STORE_TYPE=sqlite
    export DATABASE_URL=sqlite://pangolin.db
    # ... same AWS vars as above ...
    
    cargo run --bin pangolin_api
    ```

2.  **Run Test**:
    ```bash
    export BUCKET_NAME=sqlite-bucket
    export BACKEND_NAME=SQLiteStore
    
    python3 tests/integration/test_pyiceberg_matrix.py
    ```

### C. PostgresStore

1.  **Start API**:
    ```bash
    export PANGOLIN_STORE_TYPE=postgres
    export DATABASE_URL=postgres://postgres:postgres@localhost:5432/pangolin
    # ... same AWS vars ...
    
    cargo run --bin pangolin_api
    ```

2.  **Run Test**:
    ```bash
    export BUCKET_NAME=postgres-bucket
    export BACKEND_NAME=PostgresStore
    
    python3 tests/integration/test_pyiceberg_matrix.py
    ```

### D. MongoStore

1.  **Start API**:
    ```bash
    # Note: Ensure DATABASE_URL is set for Mongo, main.rs detection logic relies on it.
    export PANGOLIN_STORE_TYPE=mongo
    export DATABASE_URL=mongodb://root:example@localhost:27017
    export MONGO_DB_NAME=pangolin_test
    # ... same AWS vars ...
    
    cargo run --bin pangolin_api
    ```

2.  **Run Test**:
    ```bash
    export BUCKET_NAME=mongo-bucket
    export BACKEND_NAME=MongoStore
    
    python3 tests/integration/test_pyiceberg_matrix.py
    ```

## 4. Troubleshooting

*   **500 Internal Server Error (Invalid JSON)**: Common on SQLite/Postgres. Indicates a serialization issue when writing the metadata file to the DB column.
*   **Foreign Key Constraint**: Can happen if reusing a DB without cleaning it. Recommended to drop/recreate the DB or delete `pangolin.db` between runs.
*   **Unexpected JSON Payload (Scheme Update)**: The `assert-current-schema-id` requirement from PyIceberg is not yet supported.
