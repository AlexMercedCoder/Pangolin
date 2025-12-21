# Metadata Persistence Findings & Recommendations

## Observation
*   **Success**: Creating a table and *inserting data* via PyIceberg results in Parquet data files and Avro manifest files appearing in MinIO.
*   **Failure**: The `metadata.json` file (the root of the Iceberg table) does **not** appear in MinIO.
*   **Cause**: PyIceberg talks directly to S3 for data/manifest writes (using vended credentials). However, the **Pangolin API** is responsible for generating and writing the `metadata.json`.

## Root Cause Analysis by Store Type

### 1. MemoryStore (Current Demo)
*   **Behavior**: `write_file` implementation explicitly stores content in a `DashMap` (RAM).
*   **Result**: `metadata.json` exists only in the API container's memory. It is **never** sent to S3.
*   **Impact**: Restarting the container wipes all table metadata, even if data files persist in MinIO.

### 2. PostgresStore, MongoStore, SqliteStore
*   **Behavior**: These stores *do* implement `write_file` for S3 paths, BUT they use a **Global Configuration** pattern.
    *   They initialize `AmazonS3Builder` using *process environment variables* (`S3_ENDPOINT`, `AWS_ACCESS_KEY_ID`, etc.).
    *   They do **not** use the credentials configured in the Tenant's `Warehouse` definition.
*   **Result**: 
    *   Persistence **will work** if and only if the `pangolin-api` container has the correct global S3 environment variables set.
    *   This limits the node to ensuring all Tenants/Warehouses assume the same S3 identity (or compatible permissions) for the API's own writes.
*   **Design Flaw**: There is a disconnect between "Client Writes" (using Warehouse-specific vended credentials) and "Server Writes" (using global env vars).

## Recommendations

### 1. Abstract Object Storage Layer (Priority)
Don't rely on the `CatalogStore` (Database) to handle file I/O for Iceberg metadata. Introduce a dedicated `ObjectStore` wrapper or service.
*   **Action**: Inject an `object_store::ObjectStore` (e.g., `AmazonS3`) into the API handlers or the Store implementations.
*   **Context Aware**: The Store should use the `Warehouse` configuration to initialize the `ObjectStore` client, ensuring it uses the correct credentials for that specific warehouse.

### 2. Update `CatalogStore` Trait (Alternative)
If we want to keep the Store as the single abstraction:
*   Update `write_file` signature or context to allow passing `Warehouse` credentials.
*   Update `MemoryStore` to implementing S3 writing if `S3_ENDPOINT` is detected, or explicitly warn that it is ephemeral.

### 3. Production Recommendation
For Production (`PostgresStore`), the metadata JSON should ideally live in S3, not the Postgres DAL.
*   **Design**: separating "Catalog State" (Pointers, ACLs, DB) from "Table State" (Iceberg Metadata Files).
*   **Implementation**:
    *   Keep using Postgres for Tenants, Warehouses, Catalogs, and Access Control.
    *   Use S3 (via `object_store` crate) for creating and updating `metadata.json`, ensuring the API uses the Warehouse's credentials.

## Summary
The PyIceberg client is working correctly. The API Server needs to be updated to write the Table Metadata it generates to the Warehouse's S3 bucket instead of its internal storage. The `MemoryStore` is strictly ephemeral. Other stores are "Global S3" dependent.
