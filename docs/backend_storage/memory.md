# In-Memory Store Backend

The in-memory store is a high-performance, zero-configuration backend for development, testing, and temporary deployments.

## Overview

The in-memory store uses `DashMap` (concurrent HashMap) to store all catalog metadata in RAM with:
- **Zero configuration** - No database setup required
- **Blazing fast** - All operations in memory
- **Thread-safe** - Concurrent access via DashMap
- **Ephemeral** - Data lost on restart
- **Perfect for testing** - Instant setup and teardown

## Pros and Cons

### ✅ Advantages
- **Zero Setup**: No database installation or configuration
- **Maximum Performance**: All operations in RAM
- **Instant Startup**: No connection delays
- **Perfect for CI/CD**: Fast, isolated test runs
- **No Dependencies**: Built into Pangolin
- **Thread-Safe**: Concurrent access without locks
- **Deterministic**: Fresh state on every restart

### ⚠️ Considerations
- **Data Loss on Restart**: All data is ephemeral
- **Memory Usage**: Entire catalog in RAM
- **Single Instance**: No clustering or replication
- **Not for Production**: Data persistence required
- **Limited Scale**: Constrained by available RAM

## Use Cases

**Best For**:
- Local development
- Unit and integration testing
- CI/CD pipelines
- Demos and prototyping
- Temporary environments
- Learning and experimentation

**Not For**:
- Production deployments
- Data that needs to persist
- Multi-instance deployments
- Large catalogs (>10GB metadata)

## Configuration

### Environment Variables

```bash
# No DATABASE_URL needed - in-memory is the default!
# Just don't set DATABASE_URL and Pangolin uses in-memory store

# Optional: Set log level
RUST_LOG=info
```

### Explicit Configuration

If you want to be explicit:

```bash
# In your environment or .env file
# Simply omit DATABASE_URL or set it to empty
DATABASE_URL=

# Or in code, the MemoryStore is used when no DATABASE_URL is provided
```

## Usage

### Starting Pangolin with In-Memory Store

```bash
# Default - uses in-memory store
cd pangolin
cargo run --bin pangolin_api

# Or with explicit log level
RUST_LOG=debug cargo run --bin pangolin_api
```

### Docker

```bash
# Run without DATABASE_URL
docker run -p 8080:8080 pangolin/pangolin-api

# Or with environment file (omit DATABASE_URL)
docker run -p 8080:8080 --env-file .env pangolin/pangolin-api
```

### Testing

```bash
# Run tests with in-memory store (default)
cargo test

# Run specific test
cargo test test_catalog_crud

# Run with output
cargo test -- --nocapture
```

## Performance Characteristics

### Speed
- **Create operations**: ~1-10 microseconds
- **Read operations**: ~1-5 microseconds
- **List operations**: ~10-100 microseconds (depends on size)
- **Delete operations**: ~1-10 microseconds

### Memory Usage
Approximate memory per entity:
- **Tenant**: ~500 bytes
- **Warehouse**: ~1 KB
- **Catalog**: ~1 KB
- **Namespace**: ~500 bytes
- **Asset**: ~2 KB (including properties)

**Example**: 1000 tables = ~2 MB of metadata

### Concurrency
- **Thread-safe**: Multiple concurrent requests
- **Lock-free reads**: DashMap allows concurrent reads
- **Write performance**: Excellent for moderate concurrency
- **No connection pool**: No connection overhead

## Development Workflow

### Quick Start

```bash
# 1. Start Pangolin (uses in-memory by default)
cargo run --bin pangolin_api

# 2. Create a tenant
curl -X POST http://localhost:8080/api/v1/tenants \
  -H "Content-Type: application/json" \
  -d '{"name": "dev-tenant"}'

# 3. Create a catalog
curl -X POST http://localhost:8080/api/v1/catalogs \
  -H "X-Pangolin-Tenant: dev-tenant" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "dev-catalog",
    "type": "local"
  }'

# 4. Use with PyIceberg
python your_script.py

# 5. Restart to reset - all data cleared!
# Ctrl+C and restart
cargo run --bin pangolin_api
```

### Testing Pattern

```rust
#[tokio::test]
async fn test_my_feature() {
    // In-memory store is created automatically in tests
    let store = MemoryStore::new();
    
    // Create test data
    let tenant = Tenant {
        id: Uuid::new_v4(),
        name: "test".to_string(),
        properties: HashMap::new(),
    };
    
    store.create_tenant(tenant.clone()).await.unwrap();
    
    // Test your feature
    let result = store.get_tenant(tenant.id).await.unwrap();
    assert!(result.is_some());
    
    // No cleanup needed - test isolation automatic
}
```

### CI/CD Integration

```yaml
# GitHub Actions example
name: Test
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test
        # Uses in-memory store by default - no DB setup needed!
```

## Switching to Persistent Storage

When you're ready for production:

### To PostgreSQL

```bash
# 1. Set DATABASE_URL
export DATABASE_URL=postgresql://user:password@localhost:5432/pangolin

# 2. Restart Pangolin
cargo run --bin pangolin_api

# Data from in-memory store is NOT migrated
# You'll start with a fresh database
```

### To MongoDB

```bash
# 1. Set DATABASE_URL
export DATABASE_URL=mongodb://user:password@localhost:27017/pangolin

# 2. Restart Pangolin
cargo run --bin pangolin_api
```

### To SQLite

```bash
# 1. Set DATABASE_URL
export DATABASE_URL=sqlite://./pangolin.db

# 2. Restart Pangolin
cargo run --bin pangolin_api
```

## Limitations

### Data Persistence
- ❌ No persistence across restarts
- ❌ No backup/restore
- ❌ No point-in-time recovery
- ❌ Data lost on crash

### Scalability
- ❌ Single instance only
- ❌ No replication
- ❌ No clustering
- ❌ Limited by RAM

### Production Features
- ❌ No audit trail persistence
- ❌ No disaster recovery
- ❌ No multi-region support

## Troubleshooting

### Out of Memory

```
Error: Cannot allocate memory
```

**Solution**: Switch to persistent backend or increase available RAM

### Data Lost After Restart

**This is expected behavior!** In-memory store is ephemeral.

**Solution**: Use PostgreSQL, MongoDB, or SQLite for persistence

### Slow Performance with Large Datasets

**Solution**: In-memory should be fast. If slow:
1. Check available RAM
2. Monitor memory usage
3. Consider persistent backend for large datasets

## Monitoring

### Memory Usage

```bash
# Check Pangolin process memory
ps aux | grep pangolin_api

# Or use top
top -p $(pgrep pangolin_api)
```

### Rust Logging

```bash
# Enable debug logging
RUST_LOG=debug cargo run --bin pangolin_api

# See memory-related logs
RUST_LOG=pangolin_store=debug cargo run --bin pangolin_api
```

## Best Practices

### Development
1. ✅ Use in-memory for local development
2. ✅ Restart to reset state during testing
3. ✅ Use scripts to seed test data
4. ✅ Keep test datasets small

### Testing
1. ✅ Use in-memory for unit tests
2. ✅ Use in-memory for integration tests
3. ✅ Each test gets isolated state
4. ✅ No cleanup needed between tests

### CI/CD
1. ✅ Use in-memory for fast CI runs
2. ✅ No database setup required
3. ✅ Parallel test execution safe
4. ✅ Deterministic test results

## Comparison with Other Backends

| Feature | In-Memory | SQLite | PostgreSQL | MongoDB |
|---------|-----------|--------|------------|---------|
| **Setup Time** | 0 seconds | 0 seconds | Minutes | Minutes |
| **Persistence** | ❌ No | ✅ Yes | ✅ Yes | ✅ Yes |
| **Performance** | Excellent | Excellent | Very Good | Very Good |
| **Memory Usage** | High | Low | Medium | Medium |
| **Production Ready** | ❌ No | ✅ Yes | ✅ Yes | ✅ Yes |
| **Testing** | ✅ Perfect | ✅ Good | ⚠️ Requires setup | ⚠️ Requires setup |

## Migration Path

**Recommended Development Flow**:

1. **Start**: In-memory for rapid development
2. **Local Testing**: SQLite for persistent local testing
3. **Staging**: PostgreSQL or MongoDB
4. **Production**: PostgreSQL or MongoDB

## Additional Resources

- [DashMap Documentation](https://docs.rs/dashmap/)
- [Backend Storage Comparison](comparison.md)
- [PostgreSQL Backend](postgresql.md)
- [MongoDB Backend](mongodb.md)
- [SQLite Backend](sqlite.md)

## Next Steps

- [PostgreSQL Backend](postgresql.md) - For production deployments
- [MongoDB Backend](mongodb.md) - For cloud-native deployments
- [SQLite Backend](sqlite.md) - For embedded deployments
- [Backend Comparison](comparison.md) - Choose the right backend
