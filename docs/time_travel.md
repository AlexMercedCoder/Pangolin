# Time Travel Queries

Pangolin supports Iceberg's time-travel capabilities, allowing users to query tables as they existed at a specific point in time.

## Usage

Time-travel is performed via the `loadTable` endpoint (`GET /v1/{prefix}/namespaces/{namespace}/tables/{table}`) by providing query parameters.

### By Snapshot ID
Query the table as it was at a specific snapshot.

`GET .../tables/my_table?snapshot-id=123456789`

### By Timestamp
Query the table as it was at a specific timestamp (milliseconds since epoch).

`GET .../tables/my_table?as-of-timestamp=1678886400000`

## Implementation Details

When a time-travel parameter is provided:
1. Pangolin retrieves the current metadata for the table.
2. It verifies if the requested snapshot (or a snapshot valid at the timestamp) exists in the metadata history.
3. If found, it returns the metadata. The Iceberg client (e.g., Spark, Trino) then uses this metadata to read the correct data files.
4. If the snapshot is not found (e.g., expired and cleaned up), a `404 Not Found` is returned.
