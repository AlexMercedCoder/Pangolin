# Outstanding Backend Work

This document tracks backend improvements and bug fixes identified during testing that have been deferred to focus on UI and integration verification.

## High Priority Fixes

### 1. Audit Log User Attribution
- **File**: `iceberg_handlers.rs`
- **Issue**: Several Iceberg REST endpoints (create/update/delete table) currently hardcode the user as `"system"` when logging audit events.
- **Goal**: Update all handlers to correctly extract and log `session.username` from the `UserSession` extension.
- **Affected Handlers**:
  - `create_table`
  - `register_table`
  - `update_table`
  - `rename_table`
  - `drop_table`

### 2. Empty Storage Location Path Resolution
- **Files**: `pangolin_handlers.rs`, `iceberg_handlers.rs`
- **Issue**: When a catalog is created with an empty string for `storage_location` (instead of `None`), the backend incorrectly treats it as an absolute path prefix (`/`). This causes PyIceberg to fail when trying to write to local system roots.
- **Goal**: 
  - Update `create_catalog` and `update_catalog` to convert empty strings to `None`.
  - Update `create_table` logic to skip prefixes if the location is an empty string.

---

## Future Improvements

### 3. Expanded Credential Vending
- **File**: `iceberg_handlers.rs`
- **Issue**: Credential vending is currently robust for S3 but needs implementation/verification for Azure ADLS Gen2 (SAS tokens) and GCP (Downscoped tokens) in the `load_table` and `create_table` flows.
- **Status**: Research needed on specific PyIceberg requirements for Azure/GCS token headers.

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

### 7. Effective Permission Aggregation
- **Files**: `pangolin_store/src/memory.rs`, `pangolin_store/src/postgres.rs`, etc.
- **Issue**: `store.list_user_permissions(user_id)` currently only returns direct permission grants, completely ignoring permissions inherited from assigned roles.
- **Goal**: Update `list_user_permissions` in all store implementations to return a union of direct permissions and role-based permissions. This ensures that callers (like Catalog listing and Discovery Search) correctly respect RBAC.

### 8. Permission-Aware Unified Search
- **File**: `optimization_handlers.rs`
- **Issue**: The `unified_search` handler (Global Search) returns results for the entire tenant without performing any permission checks.
- **Goal**: Implement filtering in the handler to ensure users only see search results for Catalogs, Namespaces, and Assets for which they have at least `Read` or `Discoverable` access.

### 9. Standardize Authz Utilization
- **Files**: `pangolin_handlers.rs`, `business_metadata_handlers.rs`
- **Issue**: Handlers for `list_catalogs` and `search_assets` contain manual, incomplete permission checking logic.
- **Goal**: Refactor these handlers to use `authz::check_permission` or a consistent effective-permission set to determine visibility.
