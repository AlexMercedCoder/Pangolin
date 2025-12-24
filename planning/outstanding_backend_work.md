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

### ✅ 4. Generic Asset Management
- **File**: `asset_handlers.rs`
- **Issue**: Standard Iceberg clients break when encountering non-Table assets in `list_tables` response. UI needs to view details for generic assets.
- **Solution**:
  - ✅ Implemented `list_assets` endpoint returning `AssetSummary` with `kind` field.
  - ✅ Updated `iceberg_handlers::list_tables` to filter ONLY `IcebergTable` types.
  - ✅ Implemented `get_asset` endpoint for generic asset details (CSV, Delta, etc.).
- **Status**: ✅ **COMPLETED** - Verified with `seed_generic_assets.py` and UI Explorer.
- **Date Completed**: 2025-12-23

### 5. Consistent Catalog Scoping for Branches
- **File**: `pangolin_handlers.rs`
- **Issue**: Some branch-related endpoints might assume a "default" catalog if not specified.
- **Goal**: Ensure all Pangolin-specific APIs consistently require or resolve the catalog context to stay aligned with the multi-tenant, multi-catalog architecture.

### 6. Asset ID in Iceberg Table Response
- **File**: `iceberg_handlers.rs`
- **Issue**: The `TableResponse` returned by `load_table` (Iceberg REST) does not currently include the internal Pangolin `asset_id` (UUID).
- **Goal**: Add `id: Uuid` to the `TableResponse` struct and populate it from the `asset` retrieved in the handler. This is required for the UI to bridge from Iceberg metadata to Pangolin business metadata (tags, descriptions).
- **Status**: Required for "Business Info" tab in Explorer.

### ✅ 7. Permission-Aware Dashboard Statistics
- **File**: `dashboard_handlers.rs`
- **Issue**: 
    - `TenantUser` currently sees hardcoded `0` for warehouse counts.
    - Other counts (catalogs, tables, namespaces) currently return tenant-wide totals instead of filtering by what the user actually has access to.
- **Solution**: 
    - Updated `get_dashboard_stats` to count accessible resources.
- **Status**: ✅ **COMPLETED** - Basic implementation done. Enhancements tracked below.
- **Date Completed**: 2025-12-23

### ✅ 10. Fix Permission Action Implications
- **File**: `pangolin_core/src/permission.rs`
- **Issue**: `Action::Read` did not imply `Action::List`.
- **Solution**: Updated `Action::implies` logic.
- **Status**: ✅ **COMPLETED**
- **Date Completed**: 2025-12-23

### ✅ 11. Implement "Discoverable" Metadata Support
- **Files**: `pangolin_api/src/authz_utils.rs`
- **Issue**: `filter_assets` ignored `discoverable` flag.
- **Solution**: Updated logic to allow access if `metadata.discoverable` is true.
- **Status**: ✅ **COMPLETED**
- **Date Completed**: 2025-12-23

## Completed Items (Recent)

### ✅ 12. Tenant Creation with Initial Admin
- **File**: UI only
- **Status**: ✅ **COMPLETED** - Achieved via two sequential API calls in UI (no backend changes needed)
- **Date Completed**: 2025-12-23

### ✅ 13. Root Dashboard Global Aggregation
- **File**: `dashboard_handlers.rs`
- **Status**: ✅ **COMPLETED** - Added `tenants_count` field and implemented cross-tenant aggregation for Root users
- **Implementation**: Root dashboard now aggregates catalogs and warehouses across ALL tenants
- **Date Completed**: 2025-12-23

### ✅ 14. Root Audit Log Visibility
- **File**: `audit_handlers.rs`
- **Status**: ✅ **COMPLETED** - Updated all 3 audit handlers to support cross-tenant queries for Root users
- **Implementation**: 
  - `list_audit_events`: Aggregates events from all tenants for Root users
  - `get_audit_event`: Searches across all tenants for Root users
  - `count_audit_events`: Sums counts from all tenants for Root users
- **Date Completed**: 2025-12-23

### ✅ 15. Tenant-Scoped Login
- **File**: `user_handlers.rs`
- **Status**: ✅ **COMPLETED** - Added optional `tenant_id` parameter to resolve username collisions
- **Implementation**:
  - `LoginRequest` now accepts optional `tenant_id: Option<Uuid>`
  - Login handler validates user belongs to specified tenant
  - Root users must login with `tenant_id: null`
  - Tenant users must login with their `tenant_id`
  - Proper error messages for each scenario
- **UI Documentation**: Created `planning/tenant_scoped_login_ui.md`
- **Date Completed**: 2025-12-23

## New Outstanding Items
