# Table Maintenance

Pangolin provides maintenance utilities to optimize storage and manage metadata for Iceberg tables. These operations help keep your data lake healthy and performant.

## Features

### 1. Expire Snapshots
Removes old snapshots that are no longer needed, freeing up space and reducing metadata size.

-   **API Endpoint**: `POST /v1/{prefix}/namespaces/{namespace}/tables/{table}/maintenance`
-   **Action**: `expire_snapshots`
-   **Parameters**:
    -   `older_than_timestamp`: (Optional) Timestamp (ms) to expire snapshots created before this time.
    -   `retain_last`: (Optional) Number of recent snapshots to keep regardless of age.

### 2. Remove Orphan Files
Identifies and deletes files in the storage layer (e.g., S3) that are not referenced by any valid snapshot in the table metadata. This cleans up data left behind by failed jobs or uncommitted writes.

-   **API Endpoint**: `POST /v1/{prefix}/namespaces/{namespace}/tables/{table}/maintenance`
-   **Action**: `remove_orphan_files`
-   **Parameters**:
    -   `older_than_timestamp`: (Optional) Safety buffer; only remove files older than this timestamp.

## Usage Example

```bash
curl -X POST http://localhost:8080/v1/my_catalog/namespaces/db/tables/my_table/maintenance \
  -H "Content-Type: application/json" \
  -d '{
    "action": "expire_snapshots",
    "retain_last": 5
  }'
```

## Implementation Details

-   **S3Store**: Implements physical deletion of objects from S3.
-   **Postgres/Mongo**: Metadata updates are transactional. Physical file deletion depends on the underlying object store configuration.
