# Caching Architecture

To achieve high performance and low latency, particularly for object store operations, Pangolin implements a multi-layered caching strategy.

## 1. Metadata Cache (`MetadataCache`)
**Location**: `pangolin_store/src/metadata_cache.rs`
**Backend**: `moka` (High-performance LRU cache)

Reading Iceberg metadata files (snapshots, manifests) from S3/GCS is a high-latency operation. The `MetadataCache` stores the binary content of these files in memory to avoid repeated network calls.

### Configuration
- **Max Capacity**: 10,000 entries (default).
- **TTL (Time-To-Live)**: 5 minutes (default).
- **Key**: Absolute location string (e.g., `s3://warehouse/db/table/metadata/v1.json`).
- **Value**: Byte vector (`Vec<u8>`).

### Usage
- Used by all CatalogStore implementations (`MemoryStore`, `SqliteStore`, `PostgresStore`, `MongoStore`).
- Implements a `get_or_fetch` pattern to handle cache misses transparently.

```rust
// logical flow
let data = metadata_cache.get_or_fetch(location, || async {
    // Only executed on cache miss
    object_store.get(location).await
}).await?;
```

## 2. Object Store Cache (`ObjectStoreCache`)
**Location**: `pangolin_store/src/object_store_cache.rs`
**Backend**: `DashMap` (Concurrent HashMap)

Creating new connections to object storage (S3 clients, Azure clients) is expensive due to potential TLS handshakes and authentication checks. This cache reuses existing client instances for the same configuration.

### Logic
- **Key**: Hash of (Endpoint + Bucket + AccessKey + Region).
- **Value**: `Arc<dyn ObjectStore>` (Thread-safe shared client).
- **Eviction**: None (lives for the application lifecycle), but lightweight.

### Benefits
- drastically reduces connection overhead for frequently accessed warehouses.
- Crucial for multi-tenant environments where many warehouses exist.

## 3. Database Connection Pooling
While not a custom cache, all persistent stores utilize connection pooling:
- **Postgres**: `sqlx::PgPool`
- **SQLite**: `sqlx::SqlitePool`
- **MongoDB**: `mongodb::Client` internal pooling

## Future Caching Plans
- **Manifest Caching**: Parsing Avro manifest files is CPU intensive. Future optimizations may cache the parsed `ManifestFile` structs rather than just the raw bytes.
- **Distributed Cache**: For horizontal scaling, replacing in-memory `moka` with Redis.
