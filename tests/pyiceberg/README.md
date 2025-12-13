# PyIceberg Integration Tests

This directory contains integration tests for PyIceberg compatibility with the Pangolin catalog.

## Test Files

### Core Integration Tests
- `test_pyiceberg.py` - Basic PyIceberg integration test with Bearer token auth
- `test_pyiceberg_full.py` - Full integration test suite
- `test_read_fix.py` - Tests for snapshot tracking and read operations
- `test_client_credentials.py` - Tests client-provided S3 credentials scenario
- `test_warehouse_credentials.py` - Tests warehouse-based credential vending

### Authentication Tests
- `test_no_auth.py` - Tests NO_AUTH mode
- `test_token_auth.py` - Tests Bearer token authentication
- `test_headers.py` - Tests custom header handling

### Utility Scripts
- `check_session.py` - Session validation utility
- `debug_requests.py` - Request debugging utility
- `verify_pyiceberg.py` - PyIceberg installation verification

### MinIO Integration
- `test_minio_integration.py` - MinIO S3 compatibility tests

## Running Tests

### Prerequisites
```bash
# Install PyIceberg
pip install pyiceberg pyarrow

# Start MinIO (in separate terminal)
docker run -p 9000:9000 -p 9001:9001 \
  -e MINIO_ROOT_USER=minioadmin \
  -e MINIO_ROOT_PASSWORD=minioadmin \
  quay.io/minio/minio server /data --console-address ":9001"

# Start Pangolin API
cd pangolin
PANGOLIN_NO_AUTH=1 cargo run --bin pangolin_api
```

### Run Tests
```bash
# Client-provided credentials (always works)
python3 tests/pyiceberg/test_client_credentials.py

# Warehouse credential vending
python3 tests/pyiceberg/test_warehouse_credentials.py

# Read operations
python3 tests/pyiceberg/test_read_fix.py

# Full integration suite
python3 tests/pyiceberg/test_pyiceberg_full.py
```

## Test Results

All tests passing ✅:
- Client-provided credentials: 3 rows written/read
- Warehouse credential vending: 5 rows written/read
- Read operations: Working perfectly
- Snapshot tracking: All tests passing

## Key Features Tested

1. **Authentication**
   - Bearer token auth
   - NO_AUTH mode
   - Custom headers (X-Pangolin-Tenant)

2. **Credential Vending**
   - Warehouse-based credentials
   - Client-provided credentials
   - S3 endpoint configuration

3. **Data Operations**
   - Table creation
   - Data writes (append)
   - Data reads
   - Snapshot tracking

4. **Iceberg Features**
   - Namespace management
   - Table metadata
   - Schema evolution
   - Partition specs
