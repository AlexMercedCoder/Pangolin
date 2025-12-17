# Pangolin Master Plan & Outstanding Work

**Last Updated**: 2025-12-17
**Status**: Consolidated from Audit & Cloud Strategy docs.

This document serves as the **single source of truth** for remaining work on the Pangolin project, consolidating findings from the implementation audit and the cloud strategy roadmap.

---

## ✅ Phase 1: Core System Hardening (COMPLETE)

**Status**: All tasks completed and verified on 2025-12-17.

These items addressed direct security and performance gaps identified in the audit.

### 1.1 Security: Granular Permissions ✅
- **Current State**: ✅ **IMPLEMENTED AND VERIFIED**
- **Goal**: Implement granular, resource-scoped permission checks.
- **Tasks**:
  - [x] **MANAGE_DISCOVERY Check**: Implemented per-catalog permission check for setting discoverability in `business_metadata_handlers.rs`.
  - [x] **Search Filtering**: Implemented search result filtering based on user's specific read permissions for each asset.

### 1.2 Performance: Asset Lookup Optimization ✅
- **Current State**: ✅ **IMPLEMENTED AND VERIFIED** (0.0031s lookup time)
- **Goal**: O(1) or O(log n) asset lookup by ID.
- **Tasks**:
  - [x] **Fix MemoryStore Index**: Fixed `assets_by_id` index to update on `create_asset`.
  - [x] **Implement Trait Method**: Implemented `get_asset_by_id` in `CatalogStore` trait for all backends.
  - [x] **Update Handlers**: Refactored `business_metadata_handlers.rs` to use optimized lookup.

### 1.3 Verification Results
- ✅ **Permission-Based Search Filtering**: Restricted user correctly sees only authorized assets
- ✅ **Write Protection**: Unauthorized access correctly returns 403 Forbidden
- ✅ **Performance**: Asset lookup completes in 0.0031s (target: < 0.01s)

### 1.4 Key Findings
> **IMPORTANT**: `PANGOLIN_NO_AUTH=true` mode bypasses ALL authentication, including JWT tokens. This mode:
> - Ignores any JWT tokens in requests
> - Always creates admin sessions regardless of credentials
> - Should ONLY be used for quick testing/evaluation
> - Should NEVER be used for permission testing or production

**Recommendation**: Document this limitation in README and deployment guides.

---

## ✅ Phase 2: Feature Completeness (COMPLETE)

**Status**: Token Management completed and verified on 2025-12-17.

### 2.1 Token Management ✅
- **Current State**: ✅ **IMPLEMENTED AND VERIFIED**
- **Goal**: Secure session management with token revocation.
- **Tasks**:
  - [x] **Token Blacklist**: Implemented DB-backed token revocation for all backends (Memory, Postgres, Mongo, SQLite).
  - [x] **JWT Claims Update**: Added optional `jti` field for backward compatibility.
  - [x] **Auth Middleware Integration**: Token revocation check integrated into auth flow.
  - [x] **API Endpoints**: Created self-revoke, admin-revoke, and cleanup endpoints.
  - [x] **Background Cleanup Job**: Hourly job to remove expired tokens.
  - [x] **Comprehensive Tests**: Integration tests for all revocation scenarios.

### 2.2 Client-Side Updates (DEFERRED)
- **Goal**: Make clients aware of the new Multi-Catalog API capabilities.
- **Status**: Deferred to future phase - not critical for MVP.
- **Tasks**:
  - [ ] **CLI**: Add global `--catalog` flag to all asset commands.
  - [ ] **UI**: Add "Active Catalog" context selector in navigation.
  - [ ] **UI**: Update data fetching to use selected catalog context.

### 2.3 Verification Results
- ✅ **Token Revocation**: Self-revoke and admin-revoke working correctly
- ✅ **Auth Middleware**: Revoked tokens correctly rejected with 401
- ✅ **Cleanup Job**: Expired tokens successfully removed
- ✅ **Backward Compatibility**: Old tokens without `jti` still work
- ✅ **PyIceberg Compatible**: Confirmed no breaking changes
- ✅ **All Backends**: MemoryStore, PostgresStore, MongoStore, SqliteStore implemented

### 2.4 Key Implementation Details
- **Approach**: Database-backed blacklist (no Redis dependency)
- **Migrations**: Created for PostgreSQL and SQLite
- **Performance**: O(1) revocation check in all backends
- **Security**: Admin-only endpoints for revoking others' tokens

---

## ☁️ Phase 3: Multi-Cloud Strategy

### 3.1 Credential Vending Expansion
- **Goal**: Support Azure and GCP for native Iceberg access.
- **Current State**: Only AWS S3 (STS/Static) is supported.
- **Tasks**:
  - [ ] **Refactor Signer**: Update `Warehouse` config to support "Vending Strategy" selection.
  - [ ] **Azure Implementation**: Implement `Signer` for `az://` paths using SAS Tokens (requires `azure_storage_blobs` crate).
  - [ ] **GCP Implementation**: Implement `Signer` for `gs://` paths using Downscoped Credentials (requires `google-cloud-storage` crate).

---

## 🎨 Phase 4: User Experience & Polish

### 4.1 Ease of Access
- **Goal**: Make authentication and documentation accessible.
- **Tasks**:
  - [ ] **Token Generation**: Implement easy generation of auth tokens from CLI and UI.
  - [ ] **Documentation Links**: Add direct links to documentation in both UI and CLI.

### 4.2 UI Improvements
- **Goal**: Clean up interface.
- **Tasks**:
  - [ ] **Remove Stats**: Remove "Total Users/Catalogs" boxes from the UI dashboard.

---

## ✅ Recent Completions (Reference)
- **Phase 1: Core System Hardening** (2025-12-17): Granular permissions, search filtering, O(1) asset lookup - all verified.
- **User CRUD**: Full RBAC implementation for User management.
- **Deletion**: Federated Catalog deletion logic.
- **Merge**: Merge base commit detection (3-way merge support).
