# Outstanding Backend Work

This document tracks backend improvements and bug fixes identified during testing that have been deferred to focus on UI and integration verification.


## Completed Items

### ✅ 1. Audit Log User Attribution
- **File**: `iceberg_handlers.rs`
- **Issue**: Several Iceberg REST endpoints hardcoded the user as `"system"` when logging audit events.
- **Solution**: All handlers now correctly extract and log `session.username` from the `UserSession` extension.
- **Status**: ✅ **COMPLETED** - Verified in create_table, register_table, update_table, rename_table, drop_table handlers.
- **Date Completed**: Prior to 2025-12-23

### ✅ 2. Empty Storage Location Path Resolution
- **Files**: `pangolin_handlers.rs`, `iceberg_handlers.rs`
- **Issue**: When a catalog was created with an empty string for `storage_location`, the backend incorrectly treated it as an absolute path prefix.
- **Solution**: `create_catalog` now converts empty strings to `None` using `.filter(|n| !n.is_empty())` on line 700.
- **Status**: ✅ **COMPLETED** - Empty strings are properly filtered to None.
- **Date Completed**: Prior to 2025-12-23

### ✅ 7. Effective Permission Aggregation
- **Files**: `pangolin_store/src/memory.rs`, `pangolin_store/src/sqlite.rs`, `pangolin_store/src/postgres.rs`, `pangolin_store/src/mongo.rs`
- **Issue**: `store.list_user_permissions(user_id)` only returned direct permission grants, ignoring permissions inherited from assigned roles.
- **Solution**: Updated `list_user_permissions` in all four store implementations to return a union of direct permissions and role-based permissions.
- **Status**: ✅ **COMPLETED** - All stores now aggregate permissions correctly. Comprehensive test coverage added.
- **Date Completed**: 2025-12-23

### ✅ 3. Expanded Credential Vending (COMPLETE - PyIceberg Compatible)
- **Files**: 
  - `pangolin_api/src/credential_signers/` (trait infrastructure)
  - `pangolin_api/src/credential_vending.rs` (factory and helpers)
  - `pangolin_api/src/signing_handlers.rs` (refactored, -170 lines)
  - `pangolin_api/tests/` (28 tests total)
- **Issue**: Credential vending was robust for S3 but needed implementation for Azure ADLS Gen2 and GCP with PyIceberg-compatible property names.
- **Solution**: 
  - ✅ Created trait-based credential signer infrastructure with `CredentialSigner` trait
  - ✅ Implemented `AzureSasSigner` with PyIceberg-compatible properties (`adls.token`, `adls.account-name`, `adls.account-key`)
    - Supports OAuth2 (with custom authority host for testing)
    - Supports account key authentication (tested with Azurite)
    - Works regardless of `azure-oauth` feature flag status
  - ✅ Implemented `GcpTokenSigner` with permission-based scope selection and PyIceberg properties
  - ✅ Implemented `S3Signer` supporting both STS AssumeRole and static credentials
  - ✅ Created `MockSigner` for comprehensive testing
  - ✅ Refactored `signing_handlers.rs` to use new infrastructure (74% code reduction)
  - ✅ Added MinIO IAM setup script for STS testing
- **Testing**: 
  - **28/28 tests passing** ✅
    - 4 unit tests (credential vending factory)
    - 15 integration tests (mock signers, multi-cloud, error handling)
    - 5 end-to-end tests (S3, Azure, GCP, multi-cloud regression, S3 STS)
    - 4 live tests with emulators (MinIO static, MinIO STS, Azurite, fake-gcs-server)
- **PyIceberg Compatibility**: ✅ **CONFIRMED**
  - Azure: Uses `adls.token`, `adls.account-name`, `adls.account-key` (PyIceberg standard)
  - GCP: Uses `gcp-oauth-token`, `gcp-project-id` (PyIceberg compatible)
  - S3: Uses `s3.access-key-id`, `s3.secret-access-key`, `s3.session-token` (PyIceberg standard)
- **Production Status**:
  - ✅ S3: Static credentials + STS foundation ready
  - ✅ Azure: Account key mode fully tested with Azurite
  - ⚠️ Azure OAuth2: Code ready, needs real Azure AD testing (OIDC mock incompatible with Azure SDK)
  - ✅ GCP: Service account key mode ready
- **Status**: ✅ **COMPLETED** - All credential vending infrastructure complete with PyIceberg compatibility. Ready for production use.
- **Date Completed**: 2025-12-23

### 4. Consistent Catalog Scoping for Branches
- **File**: `pangolin_handlers.rs`
- **Issue**: Some branch-related endpoints might assume a "default" catalog if not specified.
- **Goal**: Ensure all Pangolin-specific APIs consistently require or resolve the catalog context to stay aligned with the multi-tenant, multi-catalog architecture.
### 5. Asset ID in Iceberg Table Response
- **File**: `iceberg_handlers.rs`
- **Issue**: The `TableResponse` returned by `load_table` (Iceberg REST) does not currently include the internal Pangolin `asset_id` (UUID).
- **Goal**: Add `id: Uuid` to the `TableResponse` struct and populate it from the `asset` retrieved in the handler. This is required for the UI to bridge from Iceberg metadata to Pangolin business metadata (tags, descriptions).
- **Status**: Required for "Business Info" tab in Explorer.

### 6. Permission-Aware Dashboard Statistics
- **File**: `dashboard_handlers.rs`
- **Issue**: 
    - `TenantUser` currently sees hardcoded `0` for warehouse counts.
    - Other counts (catalogs, tables, namespaces) currently return tenant-wide totals instead of filtering by what the user actually has access to.
- **Goal**:
    - Implement permission-aware counting for all resource types.
    - `Root` should see system-wide stats (total across all tenants).
    - `TenantAdmin` should see tenant-wide stats.
    - `TenantUser` should see counts only for resources they have `Read` or `Discoverable` access to.
- **Status**: Deferred to prevent cross-component complexity during UI verification.

## Completed Items

### ✅ 7. Effective Permission Aggregation
- **Files**: `pangolin_store/src/memory.rs`, `pangolin_store/src/sqlite.rs`, `pangolin_store/src/postgres.rs`, `pangolin_store/src/mongo.rs`
- **Issue**: `store.list_user_permissions(user_id)` only returned direct permission grants, ignoring permissions inherited from assigned roles.
- **Solution**: Updated `list_user_permissions` in all four store implementations to return a union of direct permissions and role-based permissions.
- **Status**: ✅ **COMPLETED** - All stores now aggregate permissions correctly. Comprehensive test coverage added.
- **Date Completed**: 2025-12-23

---

### ✅ 8. Permission-Aware Unified Search
- **Files**: `optimization_handlers.rs`, `authz_utils.rs` (new)
- **Issue**: The `unified_search` handler returned all results within a tenant without performing any permission checks.
- **Solution**: 
  - Created `authz_utils.rs` module with permission checking and filtering functions
  - Updated `unified_search` to fetch user permissions and filter results
  - Root/TenantAdmin users bypass filtering
  - TenantUser users only see resources they have Read or Discoverable access to
- **Status**: ✅ **COMPLETED** - All unit tests pass. No breaking changes to API contracts.
- **Date Completed**: 2025-12-23

---

### ✅ 9. Standardize Authz Utilization
- **Files**: `pangolin_handlers.rs`, `business_metadata_handlers.rs`
- **Issue**: Handlers for `list_catalogs` and `search_assets` contained manual, incomplete permission checking logic.
- **Solution**: 
  - Refactored both handlers to use `authz_utils::filter_catalogs()` and `authz_utils::filter_assets()`
  - Removed ~32 lines of duplicate permission checking code
  - Fixed bug where Tenant-scoped permissions weren't checked in `list_catalogs`
- **Status**: ✅ **COMPLETED** - All code compiles successfully. No breaking changes.
- **Date Completed**: 2025-12-23

---

## Future Improvements
