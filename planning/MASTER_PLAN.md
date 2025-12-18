# Pangolin Master Plan & Outstanding Work

**Last Updated**: 2025-12-18
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

## ✅ Phase 3: Multi-Cloud & Federation (COMPLETE)

**Status**: Core infrastructure, Federation, and AWS Vending completed and verified on 2025-12-18. Azure/GCP structured but pending SDK integration.

### 3.1 Federated Catalogs ✅
- **Current State**: ✅ **IMPLEMENTED AND VERIFIED**
- **Goal**: Allow Pangolin to act as a proxy for other Iceberg REST catalogs.
- **Tasks**:
    - [x] **Proxy Implementation**: `FederatedCatalogProxy` handles request forwarding.
    - [x] **Catalog Type**: Support for `Local` vs `Federated` catalogs in API.
    - [x] **Verification**: Validated connectivity and 404 handling from remote catalogs via PyIceberg.
    - [x] **Path Handling**: Fixed critical path resolution bug (relative vs absolute paths).
    - [x] **Cross-Tenant Access**: Fixed federated catalog visibility by adding required `timeout_seconds` field to config.
    - [x] **End-to-End Testing**: Verified Tenant A can successfully query Tenant B's data through federated catalogs.

### 3.2 Credential Vending Expansion 🔄
- **Goal**: Support AWS, Azure, and GCP for native Iceberg access.
- **Current State**: AWS (STS & Static) Verified. Azure/GCP Structured but unverified.
- **Tasks**:
    - [x] **Refactor Signer**: Updated `Warehouse` config to support "Vending Strategy" selection.
    - [x] **AWS Implementation**:
        - ✅ **STS Vending**: Verified working with PyIceberg (Catalog C).
        - ✅ **Static Vending**: Verified working with PyIceberg (Catalog A).
        - ✅ **S3 Bucket Resolution**: Fixed bug where default `warehouse` bucket was used instead of configured bucket.
    - [ ] **Azure Implementation**: Implement `Signer` for `az://` paths using SAS Tokens (requires `azure_storage_blobs` crate).
    - [ ] **GCP Implementation**: Implement `Signer` for `gs://` paths using Downscoped Credentials (requires `google-cloud-storage` crate).

### 3.3 Verification Results & Learnings
- ✅ **AWS Credential Vending**: Confirmed that both `AwsSts` and `AwsStatic` strategies correctly vend tokens that PyIceberg can use for S3 IO.
- ✅ **PyIceberg Branching**: Identified and fixed a client-side validation bug by using keyword arguments in `create_branch`.
- ✅ **Federated Catalogs**: Successfully verified cross-tenant data access. Tenant A can query Tenant B's namespaces through federated catalogs.
  - **Issue Found**: Missing `timeout_seconds` field in `federated_config` caused 422 validation errors.
  - **Resolution**: Added required field to federated catalog creation payload.
  - **Result**: `list_namespaces()` now returns `[('db_b',)]` as expected.
- ⚠️ **S3 Configuration**: Ensure MinIO/S3 buckets explicitly exist. The system falls back to a default `warehouse` bucket if resolution fails or config is missing, which caused initial test failures.

---

## 🎨 Phase 4: User Experience & Polish

### 4.1 Ease of Access
- **Goal**: Make authentication and documentation accessible.
- **Tasks**:
    - [ ] **Token Generation**: Implement easy generation of auth tokens from CLI and UI.
    - [ ] **Documentation Links**: Add direct links to documentation in both UI and CLI.
    - [ ] **Catalog Creation UI**: Ensure UI supports creating both `Local` and `Federated` catalogs with appropriate config fields.

### 4.2 UI Improvements
- **Goal**: Clean up interface.
- **Tasks**:
    - [ ] **Remove Stats**: Remove "Total Users/Catalogs" boxes from the UI dashboard.
    - [ ] **Live Verification**: Run E2E tests for UI features with PyIceberg and MinIO.

---

## ✅ Recent Completions (Reference)
- **Phase 1: Core System Hardening** (2025-12-17): Granular permissions, search filtering, O(1) asset lookup - all verified.
- **User CRUD**: Full RBAC implementation for User management.
- **Deletion**: Federated Catalog deletion logic.
- **Merge**: Merge base commit detection (3-way merge support).
- **Phase 3: Federated Catalog Access** (2025-12-18): 
  - ✅ Fixed credential vending for S3/MinIO access
  - ✅ Resolved JWT token validation issues (secret mismatch)
  - ✅ Implemented federation proxy logic in all Iceberg handlers
  - ✅ Fixed permission listing (500/400 errors)
  - ✅ Added unit tests for serialization and permission filtering
  - ✅ E2E test (`test_cli_live.sh`) passing all 15 steps

---

## 🧪 Phase 5: Test Suite Maintenance

### 5.1 Test Audit Findings (2025-12-18)
- **Status**: Audit complete. 8 compilation errors identified.
- **Primary Issues**:
  - Outdated `Claims` struct usage (missing `username`, `jti`, `iat` fields)
  - Missing `User` struct fields (`active`, `last_login`, `oauth_provider`, `mfa_enabled`)
  - Deprecated `PermissionScope::System` variant
  - Missing `Warehouse.vending_strategy` field

### 5.2 Recommended Actions
- **Priority 1**: Fix struct definitions in test files
- **Priority 2**: Consolidate test files (move `src/*_test.rs` to `tests/` directory)
- **Priority 3**: Add missing test coverage for federation proxy logic

**Reference**: See [`test_audit_report.md`](file:///home/alexmerced/.gemini/antigravity/brain/a61bc4b8-f11a-4118-b176-848228ad3b68/test_audit_report.md) for full details.

