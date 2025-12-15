# Backend Storage Completion Plan

## Status: Backend Storage ✅ COMPLETE | API CRUD & STS ⚠️ IN PROGRESS

---

## ✅ COMPLETED: Backend Storage Implementation

### ✅ Phase 1: SQLite Implementation (COMPLETE)
**Status**: Production Ready - All 6 tests passing

**Completed Tasks**:
- ✅ Created `sqlite.rs` module with full CRUD operations
- ✅ Created `sql/sqlite_schema.sql` with all tables and indexes
- ✅ Implemented all `CatalogStore` trait methods
- ✅ Implemented `Signer` trait for credential vending
- ✅ Added SQLite feature to `Cargo.toml`
- ✅ Exported `SqliteStore` from `lib.rs`
- ✅ Created comprehensive test suite (6 tests)
- ✅ Fixed schema application with smart SQL parsing
- ✅ All tests passing (6/6) ✅

**Test Results**:
```
test test_sqlite_tenant_crud ... ok
test test_sqlite_warehouse_crud ... ok
test test_sqlite_catalog_crud ... ok
test test_sqlite_namespace_operations ... ok
test test_sqlite_asset_operations ... ok
test test_sqlite_multi_tenant_isolation ... ok

test result: ok. 6 passed; 0 failed
```

### ✅ Phase 2: PostgreSQL Verification (COMPLETE)
**Status**: Production Ready - All 6 tests passing

**Completed Tasks**:
- ✅ Fixed PostgreSQL schema issues
- ✅ Corrected asset table UNIQUE constraint
- ✅ Updated CRUD operations to use correct column names
- ✅ Fixed `AssetType` enum variants
- ✅ All tests passing (6/6) ✅

### ✅ Phase 3: MongoDB Verification (COMPLETE)
**Status**: Production Ready - All 5 tests passing

**Completed Tasks**:
- ✅ Verified MongoDB implementation
- ✅ All CRUD operations working correctly
- ✅ Multi-tenant isolation verified
- ✅ All tests passing (5/5) ✅

### ✅ Phase 4: Documentation (COMPLETE)
**Status**: Comprehensive documentation for all backends

**Completed Tasks**:
- ✅ Created `/docs/backend_storage/` directory (6 files)
- ✅ Created `/docs/warehouse/` directory (4 files)
- ✅ Updated README.md with logo and reorganized structure
- ✅ Updated architecture.md, dependencies.md, env_vars.md
- ✅ Removed old `/docs/storage/` directory

**Backend Storage Status**:

| Backend | Status | Tests | Use Case |
|---------|--------|-------|----------|
| **In-Memory** | ✅ Production Ready | N/A | Development, Testing, CI/CD |
| **SQLite** | ✅ Production Ready | 6/6 ✅ | Development, Embedded, Edge |
| **PostgreSQL** | ✅ Production Ready | 6/6 ✅ | Production, Enterprise |
| **MongoDB** | ✅ Production Ready | 5/5 ✅ | Cloud-Native, Scalable |

---

## ⚠️ OUTSTANDING WORK: API CRUD & STS Implementation

### Current State Analysis

#### ✅ What's Working

**API Handlers (26 files)**:
- ✅ Iceberg REST API (full spec implementation)
- ✅ Warehouses: List, Create, Get, Delete
- ✅ Catalogs: List, Create, Get, Delete
- ✅ Tenants: List, Create, Get
- ✅ Users: Full CRUD + authentication
- ✅ Permissions/Roles: Full CRUD
- ✅ Business Metadata: Full CRUD
- ✅ Federated Catalogs: Full CRUD
- ✅ Merge Operations: Full workflow
- ✅ OAuth: Complete flow

**Store Implementations**:
- ✅ MemoryStore: Fully functional
- ✅ PostgresStore: ✅ Complete
- ✅ MongoStore: ✅ Complete
- ✅ SqliteStore: ✅ Complete

#### ❌ Critical Gaps

##### 1. Missing CRUD Operations

**Warehouses**:
- ❌ `UPDATE /api/v1/warehouses/:name` - No update endpoint
- ❌ `update_warehouse` method missing from CatalogStore trait

**Catalogs**:
- ❌ `PUT /api/v1/catalogs/:name` - No update endpoint  
- ❌ `update_catalog` method missing from CatalogStore trait

**Tenants**:
- ❌ `PUT /api/v1/tenants/:id` - No update endpoint
- ❌ `DELETE /api/v1/tenants/:id` - No delete endpoint
- ❌ `update_tenant` and `delete_tenant` methods missing from CatalogStore trait

##### 2. Incomplete STS Implementation

**Current State** ([`signing_handlers.rs:108-140`](file:///home/alexmerced/development/personal/Personal/2026/pangolin/pangolin/pangolin_api/src/signing_handlers.rs#L108-L140)):
```rust
// TODO: Implement Azure AD OAuth2 token generation
config.insert("adls.oauth2.token".to_string(), "AZURE_OAUTH_TOKEN_PLACEHOLDER".to_string());

// TODO: Implement GCS OAuth2 token generation  
config.insert("gcs.oauth2.token".to_string(), "GCS_OAUTH_TOKEN_PLACEHOLDER".to_string());

// TODO: Implement AWS STS AssumeRole
config.insert("s3.session.token".to_string(), "AWS_SESSION_TOKEN_PLACEHOLDER".to_string());
```

**Missing**:
- ❌ AWS STS AssumeRole integration
- ❌ Azure OAuth2 token generation
- ❌ GCP service account token generation
- ❌ Credential caching/refresh logic
- ❌ Role ARN validation
- ❌ External ID verification

---

## 📋 REMAINING WORK

### Phase 5: Complete API CRUD Operations

#### 5.1 Warehouse Update

**Files to Modify**:
- `pangolin_store/src/lib.rs` - Add trait method
- `pangolin_store/src/memory.rs` - Implement
- `pangolin_store/src/postgres.rs` - Implement
- `pangolin_store/src/mongo.rs` - Implement
- `pangolin_store/src/sqlite.rs` - Implement
- `pangolin_api/src/warehouse_handlers.rs` - Add handler
- `pangolin_api/src/lib.rs` - Add route

**Implementation**:
```rust
// CatalogStore trait
async fn update_warehouse(&self, tenant_id: Uuid, name: String, updates: WarehouseUpdate) -> Result<Warehouse>;

// MemoryStore
async fn update_warehouse(&self, tenant_id: Uuid, name: String, updates: WarehouseUpdate) -> Result<Warehouse> {
    let key = (tenant_id, name.clone());
    if let Some(mut warehouse) = self.warehouses.get_mut(&key) {
        if let Some(use_sts) = updates.use_sts {
            warehouse.use_sts = use_sts;
        }
        if let Some(config) = updates.storage_config {
            warehouse.storage_config.extend(config);
        }
        Ok(warehouse.clone())
    } else {
        Err(anyhow!("Warehouse not found"))
    }
}
```

**API Handler**:
```rust
pub async fn update_warehouse(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Extension(tenant_id): Extension<Uuid>,
    Json(updates): Json<WarehouseUpdate>,
) -> Result<Json<Warehouse>, AppError> {
    let warehouse = state.store.update_warehouse(tenant_id, name, updates).await?;
    Ok(Json(warehouse))
}
```

**Route**:
```rust
.route("/api/v1/warehouses/:name", put(update_warehouse))
```

#### 5.2 Catalog Update

Similar pattern to warehouse update.

**Files to Modify**:
- `pangolin_store/src/lib.rs` - Add trait method
- All store implementations
- `pangolin_api/src/catalog_handlers.rs` - Add handler
- `pangolin_api/src/lib.rs` - Add route

#### 5.3 Tenant Update/Delete

Add missing tenant operations following same pattern.

**Files to Modify**:
- `pangolin_store/src/lib.rs` - Add trait methods
- All store implementations
- `pangolin_api/src/tenant_handlers.rs` - Add handlers
- `pangolin_api/src/lib.rs` - Add routes

---

### Phase 6: Implement STS Credential Vending

#### 6.1 AWS STS Integration

**New Dependencies** (`Cargo.toml`):
```toml
aws-config = "1.0"
aws-sdk-sts = "1.0"
aws-credential-types = "1.0"
```

**Implementation** ([`signing_handlers.rs`](file:///home/alexmerced/development/personal/Personal/2026/pangolin/pangolin/pangolin_api/src/signing_handlers.rs)):
```rust
async fn assume_role_aws(
    role_arn: &str,
    external_id: Option<&str>,
    session_name: &str,
) -> Result<AwsCredentials> {
    let config = aws_config::load_from_env().await;
    let sts_client = aws_sdk_sts::Client::new(&config);
    
    let mut request = sts_client
        .assume_role()
        .role_arn(role_arn)
        .role_session_name(session_name)
        .duration_seconds(3600); // 1 hour
    
    if let Some(ext_id) = external_id {
        request = request.external_id(ext_id);
    }
    
    let response = request.send().await?;
    let creds = response.credentials().ok_or_else(|| anyhow!("No credentials returned"))?;
    
    Ok(AwsCredentials {
        access_key_id: creds.access_key_id().to_string(),
        secret_access_key: creds.secret_access_key().to_string(),
        session_token: Some(creds.session_token().to_string()),
        expiration: creds.expiration().map(|e| e.secs()),
    })
}
```

**Update `get_table_credentials` handler**:
```rust
pub async fn get_table_credentials(
    State(state): State<AppState>,
    Extension(tenant_id): Extension<Uuid>,
    Path((catalog_name, table_name)): Path<(String, String)>,
) -> Result<Json<HashMap<String, String>>, AppError> {
    // Get warehouse config
    let catalog = state.store.get_catalog(tenant_id, catalog_name.clone()).await?
        .ok_or_else(|| anyhow!("Catalog not found"))?;
    
    let warehouse_name = catalog.warehouse_name
        .ok_or_else(|| anyhow!("No warehouse attached to catalog"))?;
    
    let warehouse = state.store.get_warehouse(tenant_id, warehouse_name).await?
        .ok_or_else(|| anyhow!("Warehouse not found"))?;
    
    let mut config = HashMap::new();
    
    // Handle different storage types
    match warehouse.storage_config.get("type").map(|s| s.as_str()) {
        Some("s3") => {
            if warehouse.use_sts {
                // Use STS to assume role
                let role_arn = warehouse.storage_config.get("role_arn")
                    .ok_or_else(|| anyhow!("role_arn required for STS"))?;
                
                let session_name = format!("pangolin-{}-{}", tenant_id, table_name);
                let creds = assume_role_aws(role_arn, None, &session_name).await?;
                
                config.insert("s3.access-key-id".to_string(), creds.access_key_id);
                config.insert("s3.secret-access-key".to_string(), creds.secret_access_key);
                if let Some(token) = creds.session_token {
                    config.insert("s3.session-token".to_string(), token);
                }
            } else {
                // Use static credentials
                if let Some(key) = warehouse.storage_config.get("access_key_id") {
                    config.insert("s3.access-key-id".to_string(), key.clone());
                }
                if let Some(secret) = warehouse.storage_config.get("secret_access_key") {
                    config.insert("s3.secret-access-key".to_string(), secret.clone());
                }
            }
            
            if let Some(region) = warehouse.storage_config.get("region") {
                config.insert("s3.region".to_string(), region.clone());
            }
        }
        Some("azure") => {
            // TODO: Implement Azure OAuth2
            config.insert("adls.oauth2.token".to_string(), "AZURE_OAUTH_TOKEN_PLACEHOLDER".to_string());
        }
        Some("gcs") => {
            // TODO: Implement GCS OAuth2
            config.insert("gcs.oauth2.token".to_string(), "GCS_OAUTH_TOKEN_PLACEHOLDER".to_string());
        }
        _ => return Err(anyhow!("Unsupported storage type").into()),
    }
    
    Ok(Json(config))
}
```

#### 6.2 Azure OAuth2 Integration (Stretch Goal)

**New Dependencies**:
```toml
azure_identity = "0.17"
azure_core = "0.17"
```

**Implementation**:
```rust
async fn get_azure_token(
    tenant_id: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<String> {
    // Use Azure Identity SDK to get OAuth2 token
    // Implementation details...
}
```

#### 6.3 GCP Service Account Tokens (Stretch Goal)

**New Dependencies**:
```toml
gcp_auth = "0.10"
```

**Implementation**:
```rust
async fn get_gcp_token(
    service_account_key: &str,
) -> Result<String> {
    // Use GCP Auth SDK to get service account token
    // Implementation details...
}
```

---

## Verification Plan

### Unit Tests

#### Test 1: Warehouse Update
**File**: `pangolin_store/tests/catalog_store_tests.rs`
**Command**: `cd pangolin/pangolin_store && cargo test test_warehouse_update`

```rust
#[tokio::test]
async fn test_warehouse_update() {
    let store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();
    
    // Create warehouse
    let warehouse = Warehouse { /* ... */ };
    store.create_warehouse(tenant_id, warehouse).await.unwrap();
    
    // Update
    let updates = WarehouseUpdate {
        use_sts: Some(true),
        storage_config: Some(HashMap::from([
            ("new_key".to_string(), "new_value".to_string())
        ])),
    };
    let updated = store.update_warehouse(tenant_id, "test".to_string(), updates).await.unwrap();
    
    assert_eq!(updated.use_sts, true);
    assert!(updated.storage_config.contains_key("new_key"));
}
```

### Manual Testing

#### Test 2: Warehouse Update via API
1. Start API server: `cd pangolin && PANGOLIN_NO_AUTH=true cargo run --bin pangolin_api`
2. Create warehouse:
   ```bash
   curl -X POST http://localhost:8080/api/v1/warehouses \
     -H "Content-Type: application/json" \
     -d '{"name":"test-wh","use_sts":false,"storage_config":{"type":"s3","bucket":"test"}}'
   ```
3. Update warehouse:
   ```bash
   curl -X PUT http://localhost:8080/api/v1/warehouses/test-wh \
     -H "Content-Type: application/json" \
     -d '{"use_sts":true,"storage_config":{"role_arn":"arn:aws:iam::123:role/test"}}'
   ```
4. Verify update:
   ```bash
   curl http://localhost:8080/api/v1/warehouses/test-wh | jq '.use_sts'
   # Should return: true
   ```

#### Test 3: STS Credential Vending
1. Configure warehouse with STS:
   ```bash
   curl -X POST http://localhost:8080/api/v1/warehouses \
     -H "Content-Type: application/json" \
     -d '{
       "name":"sts-wh",
       "use_sts":true,
       "storage_config":{
         "type":"s3",
         "role_arn":"arn:aws:iam::123456789:role/PangolinDataAccess",
         "bucket":"my-bucket",
         "region":"us-east-1"
       }
     }'
   ```
2. Create catalog with warehouse:
   ```bash
   curl -X POST http://localhost:8080/api/v1/catalogs \
     -H "Content-Type: application/json" \
     -d '{"name":"test-catalog","warehouse":"sts-wh"}'
   ```
3. Request credentials:
   ```bash
   curl http://localhost:8080/api/v1/catalogs/test-catalog/tables/db.table/credentials
   # Should return temporary AWS credentials with session token
   ```

---

## Success Criteria

### Backend Storage (✅ COMPLETE)
- [x] SQLite implementation complete
- [x] PostgreSQL verified
- [x] MongoDB verified
- [x] All tests passing (17/17)
- [x] Documentation complete

### API CRUD (⚠️ TODO)
- [ ] Warehouse update endpoint implemented
- [ ] Catalog update endpoint implemented
- [ ] Tenant update/delete endpoints implemented
- [ ] All CRUD operations complete for Warehouses, Catalogs, Tenants
- [ ] Unit tests passing
- [ ] Manual testing checklist complete

### STS Implementation (⚠️ TODO)
- [ ] AWS STS credential vending functional
- [ ] Credential caching implemented
- [ ] Role ARN validation
- [ ] Azure OAuth2 credential vending functional (stretch)
- [ ] GCP token generation functional (stretch)
- [ ] Integration tests passing

### Documentation (⚠️ TODO)
- [ ] API documentation updated with new endpoints
- [ ] STS configuration guide created
- [ ] Client examples updated (PyIceberg/PySpark with STS)

---

## Timeline Estimate

### Completed
- ✅ **Backend Storage**: Complete (SQLite, PostgreSQL, MongoDB, Documentation)

### Remaining
- **Phase 5** (API CRUD completion): 4-6 hours
- **Phase 6** (STS AWS): 6-8 hours
- **Phase 6** (Azure/GCP): 4-6 hours each (stretch)
- **Testing & Documentation**: 4-6 hours

**Total Remaining**: 14-20 hours (2-3 days)

---

## Summary

### ✅ Completed Work
- **4 backend implementations**: In-Memory, SQLite, PostgreSQL, MongoDB
- **17 tests passing**: All CRUD operations verified
- **16 documentation files**: Comprehensive guides for all backends
- **Production-ready**: All backends tested and documented

### ⚠️ Outstanding Work
- **API CRUD**: Update endpoints for warehouses, catalogs, tenants
- **STS Implementation**: AWS credential vending (Azure/GCP stretch goals)
- **Testing**: Unit and integration tests for new features
- **Documentation**: API docs and STS configuration guides

---

**Last Updated**: 2025-12-14
**Backend Storage Status**: ✅ COMPLETE
**API CRUD Status**: ⚠️ TODO
**STS Status**: ⚠️ TODO
