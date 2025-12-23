# PyIceberg Transaction & Integration Fix Plan

## Goal Description
Address critical integration issues between PyIceberg and Pangolin API to enable full transaction support and robust testing across all storage backends.

## User Review Required
> [!IMPORTANT]
> **Breaking Change**: `CommitRequirement` enum will be updated. This shouldn't affect existing clients if they strictly follow the Iceberg REST spec, but verify if any custom clients rely on the current enum.

## Proposed Changes

### 1. Fix S3 Configuration in Integration Tests
**Problem**: `test_pyiceberg_matrix.py` uses underscore-separated keys (`access_key_id`) for S3 config, but `object_store_factory.rs` expects dot-separated keys (`s3.access-key-id`). This causes 403 Forbidden errors (Invalid Access Key) in `SqliteStore`/`PostgresStore` because the credentials aren't found/used correctly.
**Fix**: Update `setup_warehouse` in `test_pyiceberg_matrix.py` to use `s3.access-key-id`, `s3.secret-access-key`, `s3.region`, `s3.endpoint`.

### 2. Implement `AssertCurrentSchemaId`
**Problem**: PyIceberg sends `assert-current-schema-id` requirement during schema updates, which triggers a 422 error because the variant is missing in `CommitRequirement` enum.
**Fix**:
#### [MODIFY] [model.rs](file:///home/alexmerced/development/personal/Personal/2026/pangolin/pangolin/pangolin_core/src/model.rs)
- Add `AssertCurrentSchemaId { current_schema_id: i32 }` to `CommitRequirement` enum.

#### [MODIFY] [iceberg_handlers.rs](file:///home/alexmerced/development/personal/Personal/2026/pangolin/pangolin/pangolin_api/src/iceberg_handlers.rs)
- Handle `AssertCurrentSchemaId` in `update_table` handler. Check if `metadata.current_schema_id` matches the requirement.

### 3. Expand Integration Test Matrix
**Problem**: Testing is limited to simple create/read/write. User requested broader coverage.
**Fix**:
#### [MODIFY] [test_pyiceberg_matrix.py](file:///home/alexmerced/development/personal/Personal/2026/pangolin/tests/integration/test_pyiceberg_matrix.py)
- **Fix S3 Keys**: Correct the key names in `setup_warehouse`.
- **Add Partitioning Test**: Create table with partition spec, write data, verify.
- **Add Snapshot Test**: Verify snapshot list after writes.
- **Add Sort Order Test**: Update sort order, verify metadata.
- **Add Branching Test**: Create a branch via PyIceberg (if supported) or verify refs.

### Fix "Invalid JSON" / Metadata Write Error
**Analysis**: The "Invalid JSON" error (500) observed in logs was masking the underlying S3 403 error. Fixing the S3 keys in step 1 should resolve this. If issues persist, `write_file` implementation in `sqlite.rs` is sound but error propagation needs to be verified.

## Verification Plan

### Automated Tests
1. **Run Expanded Matrix**: Execute `tests/integration/test_pyiceberg_matrix.py` against all backends (Memory, SQLite, Postgres, Mongo).
   - Verify `cat_vended` and `cat_client` pass.
   - Verify new tests (Partitioning, Snapshots, Sort Order) pass.

### Manual Verification
- Inspect API logs for successful `write_file` operations (no 403s).
- Check MinIO headers/content if debugging is needed.
