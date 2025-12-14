# Backend Completion Implementation Plan

## Executive Summary

This plan addresses critical gaps in the Pangolin backend to ensure production readiness, focusing on completing CRUD operations, implementing STS credential vending, and ensuring all store backends are functional.

## Current State Analysis

### ✅ What's Working

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
- ✅ MemoryStore: Fully functional (used in development)
- ⚠️ PostgresStore: Partial (many stubs)
- ⚠️ MongoStore: Partial (many stubs)
- ⚠️ S3Store: Minimal (mostly stubs)

### ❌ Critical Gaps

#### 1. Missing CRUD Operations

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

#### 2. Incomplete STS Implementation

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

#### 3. Store Backend Gaps

**PostgresStore** - Missing implementations:
- All warehouse operations (create, get, list, delete, update)
- All catalog operations (create, get, list, delete, update)
- All tenant operations (update, delete)
- Namespace operations
- Asset operations
- Branch/Tag operations

**MongoStore** - Same gaps as PostgresStore

**S3Store** - Almost entirely stubs, only basic tenant/catalog structure

## Proposed Changes

### Phase 1: Complete Core CRUD Operations

#### 1.1 Warehouse Update

**Files to Modify**:
- [`pangolin_store/src/lib.rs`](file:///home/alexmerced/development/personal/Personal/2026/pangolin/pangolin/pangolin_store/src/lib.rs#L31) - Add trait method
- [`pangolin_store/src/memory.rs`](file:///home/alexmerced/development/personal/Personal/2026/pangolin/pangolin/pangolin_store/src/memory.rs#L114) - Implement
- [`pangolin_api/src/warehouse_handlers.rs`](file:///home/alexmerced/development/personal/Personal/2026/pangolin/pangolin/pangolin_api/src/warehouse_handlers.rs) - Add handler
- [`pangolin_api/src/lib.rs`](file:///home/alexmerced/development/personal/Personal/2026/pangolin/pangolin/pangolin_api/src/lib.rs#L102) - Add route

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

#### 1.2 Catalog Update

Similar pattern to warehouse update.

#### 1.3 Tenant Update/Delete

Add missing tenant operations following same pattern.

### Phase 2: Implement STS Credential Vending

#### 2.1 AWS STS Integration

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

#### 2.2 Azure OAuth2 Integration

**New Dependencies**:
```toml
azure_identity = "0.17"
azure_core = "0.17"
```

#### 2.3 GCP Service Account Tokens

**New Dependencies**:
```toml
gcp_auth = "0.10"
```

### Phase 3: Complete Store Backends

#### 3.1 PostgresStore Priority Methods

Implement in order of importance:
1. Warehouse CRUD (for production deployments)
2. Catalog CRUD
3. Tenant operations
4. Namespace operations
5. Asset/Table metadata operations

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

## Success Criteria

- [ ] All CRUD operations complete for Warehouses, Catalogs, Tenants
- [ ] AWS STS credential vending functional
- [ ] Azure OAuth2 credential vending functional (stretch)
- [ ] GCP token generation functional (stretch)
- [ ] PostgresStore implements core operations
- [ ] All unit tests passing
- [ ] Manual testing checklist complete
- [ ] Documentation updated

## Timeline Estimate

- **Phase 1** (CRUD completion): 4-6 hours
- **Phase 2** (STS AWS): 6-8 hours
- **Phase 2** (Azure/GCP): 4-6 hours each
- **Phase 3** (PostgresStore): 8-12 hours
- **Testing & Documentation**: 4-6 hours

**Total**: 26-38 hours (3-5 days)
