# Storage Backend Logic

This document explains how Pangolin's storage backends (MemoryStore, SQLiteStore, PostgresStore, MongoStore) determine which storage credentials to use when reading and writing Iceberg metadata and data files.

## Overview

Pangolin supports two credential modes:
1. **Credential Vending**: Catalog server provides temporary credentials to clients
2. **Client-Side Credentials**: Clients provide their own storage credentials

The storage backend automatically handles both scenarios using a **warehouse lookup → fallback** pattern.

## How It Works

### Step 1: Warehouse Lookup

When a file operation occurs (read or write), the backend:

1. Extracts the storage location from the file path (e.g., `s3://test-warehouse/catalog/...`)
2. Calls `get_warehouse_for_location(location)` to find a matching warehouse
3. Searches all warehouses for one where the bucket/container name appears in the location path

**Example:**
```rust
// Location: "s3://test-warehouse/catalog/namespace/table/metadata.json"
// Warehouse config: { "s3.bucket": "test-warehouse", "s3.endpoint": "...", ... }
// Match: location.contains("test-warehouse") → true
```

### Step 2: Use Warehouse Config (Credential Vending)

If a warehouse is found:
- Uses the warehouse's `storage_config` to create an object store client
- The `storage_config` contains:
  - `s3.endpoint`, `s3.bucket`, `s3.region`
  - `s3.access-key-id`, `s3.secret-access-key` (if using static credentials)
  - OR `vending_strategy` for temporary credential vending
- **Object store cache** reuses this client for subsequent operations

**Code Path:**
```rust
if let Some(warehouse) = self.get_warehouse_for_location(path).await? {
    let cache_key = self.get_object_store_cache_key(&warehouse.storage_config, path);
    let store = self.object_store_cache.get_or_insert(cache_key, || {
        Arc::new(create_object_store(&warehouse.storage_config, path).unwrap())
    });
    // Use store for read/write
}
```

### Step 3: Fallback to Environment Variables (Client-Side)

If NO warehouse is found (or warehouse lookup fails):
- Falls back to environment-based S3 client
- Reads credentials from environment variables:
  - `S3_ENDPOINT` or AWS default
  - `AWS_ACCESS_KEY_ID`
  - `AWS_SECRET_ACCESS_KEY`
  - `AWS_REGION`
- **Does NOT use AWS credential chain** (no EC2 metadata service, no IAM roles)

**Code Path:**
```rust
// Fallback for S3 paths when no warehouse found
let mut builder = AmazonS3Builder::new()  // NOT from_env()!
    .with_bucket_name(bucket)
    .with_allow_http(true);

if let Ok(endpoint) = std::env::var("S3_ENDPOINT") {
    builder = builder.with_endpoint(endpoint);
}
if let Ok(key_id) = std::env::var("AWS_ACCESS_KEY_ID") {
    builder = builder.with_access_key_id(key_id);
}
// ... etc
```

## Credential Vending vs Client-Side

| Aspect | Credential Vending | Client-Side |
|--------|-------------------|-------------|
| **Warehouse Lookup** | ✅ Succeeds (warehouse config found) | ❌ Fails (no warehouse or catalog without warehouse) |
| **Credentials Source** | Warehouse `storage_config` | Environment variables or PyIceberg config |
| **Security** | ✅ Temporary credentials, server-controlled | ⚠️ Long-lived credentials, client-controlled |
| **Use Case** | Multi-tenant, managed access | Development, testing, simple setups |
| **PyIceberg Config** | Minimal (just token + tenant header) | Full S3 credentials in catalog config |

## Troubleshooting

### Issue: "Failed to write to S3" with AWS credential chain errors

**Symptom:**
```
error sending request for url (http://169.254.169.254/latest/api/token)
```

**Cause:** Using `AmazonS3Builder::from_env()` which tries AWS credential discovery.

**Fix:** Ensure code uses `AmazonS3Builder::new()` and explicitly sets credentials.

### Issue: Warehouse not found, falling back to env credentials

**Symptom:** Files written but warehouse config not used.

**Causes:**
1. **Bucket name mismatch**: Catalog's `storage_location` uses different bucket than warehouse's `s3.bucket`
2. **Catalog without warehouse**: Catalog created without `warehouse_name` field

**Fix:**
```python
# Ensure catalog has matching storage_location
catalog_response = requests.post(
    f"{API_URL}/api/v1/catalogs",
    json={
        "name": "my_catalog",
        "warehouse_name": "my_warehouse",  # ← Links to warehouse
        "catalog_type": "Local",
        "storage_location": "s3://my-bucket/catalog",  # ← Must contain warehouse's bucket
        "properties": {}
    }
)
```

### Issue: PyIceberg writes failing

**Symptom:** Table creation fails with "Invalid JSON" or connection errors.

**Common Causes:**
1. **Port mismatch**: MinIO on non-standard ports (use 9000:9000, not 9010:9000)
2. **Missing path-style access**: Add `"s3.path-style-access": "true"` for MinIO
3. **Wrong catalog endpoint**: Use `"uri": "http://localhost:8080"` with `"prefix": "catalog_name"`

**Working PyIceberg Config:**
```python
catalog = load_catalog(
    "my_catalog",
    **{
        "type": "rest",
        "uri": "http://localhost:8080",
        "prefix": "my_catalog",
        "token": "your_jwt_token",
        "header.X-Pangolin-Tenant": "tenant_id",
        # Client-side credentials (if not using vending)
        "s3.endpoint": "http://localhost:9000",
        "s3.access-key-id": "minioadmin",
        "s3.secret-access-key": "minioadmin",
        "s3.region": "us-east-1",
        "s3.path-style-access": "true",
    }
)
```

## Implementation Details

### MemoryStore

- **Warehouse Lookup**: `get_warehouse_for_location()` - iterates DashMap, uses `location.contains(bucket)`
- **Object Store Cache**: `DashMap<String, Arc<dyn ObjectStore>>`
- **Metadata Cache**: `moka::future::Cache<String, Vec<u8>>`

### SQLiteStore

- **Warehouse Lookup**: `get_warehouse_for_location()` - queries SQLite, uses `location.contains(bucket)`
- **Object Store Cache**: Same as MemoryStore
- **Metadata Cache**: Same as MemoryStore
- **Key Fix**: Changed from exact bucket matching to `contains()` for compatibility

### PostgresStore & MongoStore

- Follow same pattern as SQLiteStore
- Warehouse lookup queries respective database
- Same caching infrastructure

## Best Practices

1. **Always set `storage_location`** when creating catalogs to ensure warehouse lookup succeeds
2. **Use credential vending** for production multi-tenant deployments
3. **Use client-side credentials** for development and testing
4. **Match bucket names** between warehouse config and catalog storage_location
5. **Use standard ports** for MinIO (9000) to avoid PyIceberg issues
6. **Enable path-style access** for MinIO and S3-compatible services
