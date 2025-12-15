# SQLite Backend Implementation Plan

## Overview
Add SQLite as a lightweight, file-based storage backend for Pangolin, perfect for testing, development, and single-node deployments.

## Benefits

### For Testing
- **No External Dependencies** - No Docker, no separate database server
- **Fast Setup** - Single file, instant startup
- **Reproducible** - Easy to reset, copy, version control test databases
- **CI/CD Friendly** - Perfect for automated testing pipelines
- **Portable** - Test databases can be shared as files

### For Development
- **Quick Iteration** - Faster than spinning up Postgres/Mongo
- **Debugging** - Easy to inspect with SQLite tools
- **Offline Development** - No network dependencies

### For Production (Small Scale)
- **Edge Deployments** - Single-node catalog servers
- **Embedded Use Cases** - Catalog embedded in applications
- **Low Resource** - Minimal memory footprint

## Implementation

### 1. Add Dependencies

```toml
# pangolin_store/Cargo.toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite"], optional = true }

[features]
sqlite = ["sqlx"]
```

### 2. Create SQLiteStore

```rust
// pangolin_store/src/sqlite.rs
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use crate::CatalogStore;

pub struct SQLiteStore {
    pool: SqlitePool,
}

impl SQLiteStore {
    pub async fn new(path: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&format!("sqlite://{}", path))
            .await?;
        
        // Run migrations
        sqlx::migrate!("./migrations/sqlite")
            .run(&pool)
            .await?;
        
        Ok(Self { pool })
    }
}
```

### 3. Schema Migrations

```sql
-- migrations/sqlite/001_initial.sql
CREATE TABLE IF NOT EXISTS tenants (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    properties TEXT -- JSON
);

CREATE TABLE IF NOT EXISTS warehouses (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    name TEXT NOT NULL,
    use_sts BOOLEAN NOT NULL,
    storage_config TEXT NOT NULL, -- JSON
    FOREIGN KEY (tenant_id) REFERENCES tenants(id)
);

CREATE TABLE IF NOT EXISTS catalogs (
    tenant_id TEXT NOT NULL,
    name TEXT NOT NULL,
    warehouse_name TEXT NOT NULL,
    storage_location TEXT NOT NULL,
    properties TEXT, -- JSON
    PRIMARY KEY (tenant_id, name),
    FOREIGN KEY (tenant_id) REFERENCES tenants(id)
);

CREATE TABLE IF NOT EXISTS namespaces (
    tenant_id TEXT NOT NULL,
    catalog_name TEXT NOT NULL,
    name TEXT NOT NULL,
    properties TEXT, -- JSON
    PRIMARY KEY (tenant_id, catalog_name, name)
);

CREATE TABLE IF NOT EXISTS assets (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    catalog_name TEXT NOT NULL,
    namespace_name TEXT NOT NULL,
    name TEXT NOT NULL,
    asset_type TEXT NOT NULL,
    metadata_location TEXT,
    branch_name TEXT,
    FOREIGN KEY (tenant_id) REFERENCES tenants(id)
);

-- Indexes for common queries
CREATE INDEX idx_warehouses_tenant ON warehouses(tenant_id);
CREATE INDEX idx_catalogs_tenant ON catalogs(tenant_id);
CREATE INDEX idx_namespaces_tenant_catalog ON namespaces(tenant_id, catalog_name);
CREATE INDEX idx_assets_tenant_catalog_ns ON assets(tenant_id, catalog_name, namespace_name);
```

### 4. Implement CatalogStore Trait

```rust
#[async_trait]
impl CatalogStore for SQLiteStore {
    async fn create_tenant(&self, tenant: Tenant) -> Result<()> {
        sqlx::query!(
            "INSERT INTO tenants (id, name, properties) VALUES (?, ?, ?)",
            tenant.id.to_string(),
            tenant.name,
            serde_json::to_string(&tenant.properties)?
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    
    async fn get_tenant(&self, id: Uuid) -> Result<Option<Tenant>> {
        let row = sqlx::query!(
            "SELECT id, name, properties FROM tenants WHERE id = ?",
            id.to_string()
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row.map(|r| Tenant {
            id: Uuid::parse_str(&r.id).unwrap(),
            name: r.name,
            properties: serde_json::from_str(&r.properties.unwrap_or_default()).unwrap_or_default(),
        }))
    }
    
    // ... implement all other trait methods
}
```

### 5. Update Main.rs

```rust
// pangolin_api/src/main.rs
let store: Arc<dyn CatalogStore + Send + Sync> = if db_url.starts_with("sqlite://") {
    tracing::info!("Using SQLite Storage: {}", db_url);
    Arc::new(SQLiteStore::new(&db_url[9..]).await.expect("Failed to connect to SQLite"))
} else if db_url.starts_with("postgres://") {
    // ... existing postgres code
} else {
    // ... memory store fallback
};
```

## Usage

### Environment Variable
```bash
# Use SQLite file
export DATABASE_URL="sqlite://pangolin.db"

# Use in-memory SQLite (testing)
export DATABASE_URL="sqlite::memory:"

# Start server
cargo run --bin pangolin_api
```

### Testing
```bash
# Create test database
export DATABASE_URL="sqlite://test.db"
PANGOLIN_NO_AUTH=1 cargo run --bin pangolin_api &

# Run tests
python3 test_pyiceberg_full.py

# Clean up
rm test.db
```

### CI/CD
```yaml
# .github/workflows/test.yml
- name: Run Integration Tests
  env:
    DATABASE_URL: sqlite::memory:
    PANGOLIN_NO_AUTH: 1
  run: |
    cargo run --bin pangolin_api &
    sleep 5
    python3 test_pyiceberg_full.py
```

## Comparison with Other Backends

| Feature | SQLite | Postgres | MongoDB | Memory |
|---------|--------|----------|---------|--------|
| Setup Complexity | ⭐ Easiest | ⭐⭐⭐ | ⭐⭐⭐ | ⭐ Easiest |
| External Deps | None | Docker/Server | Docker/Server | None |
| Persistence | File | Database | Database | None |
| Concurrent Writes | Limited | Excellent | Excellent | Excellent |
| Production Ready | Small scale | Yes | Yes | No |
| Testing | ⭐⭐⭐ Perfect | ⭐⭐ Good | ⭐⭐ Good | ⭐⭐⭐ Perfect |
| Portability | ⭐⭐⭐ File-based | ⭐ Server | ⭐ Server | ⭐⭐⭐ Built-in |

## Recommended Use Cases

### SQLite Perfect For:
- ✅ Local development
- ✅ Unit/integration testing
- ✅ CI/CD pipelines
- ✅ Single-node deployments
- ✅ Edge computing
- ✅ Embedded catalogs
- ✅ Quick demos

### Use Postgres/Mongo For:
- ✅ Multi-node deployments
- ✅ High concurrent writes
- ✅ Large-scale production
- ✅ Distributed systems

## Implementation Effort

**Estimated Time:** 4-6 hours

1. **Setup (1h)**
   - Add sqlx dependency
   - Create migration files
   - Basic SQLiteStore struct

2. **Trait Implementation (2-3h)**
   - Implement all CatalogStore methods
   - Handle JSON serialization
   - Error handling

3. **Testing (1-2h)**
   - Unit tests for SQLiteStore
   - Integration tests
   - CI/CD configuration

## Next Steps

1. Add sqlx dependency with sqlite feature
2. Create migration files
3. Implement SQLiteStore
4. Add tests
5. Update documentation
6. Add to CI/CD pipeline

## Questions to Consider

1. **File Location**: Where should default SQLite file be stored?
   - Current directory?
   - `~/.pangolin/pangolin.db`?
   - Configurable via env var?

2. **WAL Mode**: Enable Write-Ahead Logging for better concurrency?
   ```sql
   PRAGMA journal_mode=WAL;
   ```

3. **Connection Pool Size**: SQLite has limited concurrent writes
   - Keep pool small (5 connections)?
   - Document limitations?

4. **Backup Strategy**: How to backup SQLite databases?
   - Simple file copy when server stopped
   - Online backup using SQLite backup API?

Would you like me to implement this SQLite backend?
