# Phase 1 Implementation Plan: Core System Hardening

**Goal**: Address critical Security and Performance gaps identified in the audit.
**Focus**: Asset Lookup Optimization (Performance) & Granular Permissions (Security).

## 1. Performance: Asset Lookup Optimization

**Problem**: Asset lookups currently scan catalogues (O(N)). `MemoryStore` has an index (`assets_by_id`) but it is not consistently maintained.
**Solution**: Enforce O(1) lookup via `get_asset_by_id`.

### Changes

#### 1.1 `pangolin_store/src/lib.rs`
- **Modify**: `CatalogStore` trait.
- **Change**: Remove default implementation of `get_asset_by_id`. Force concrete implementations.
- **Signature**: `async fn get_asset_by_id(&self, tenant_id: Uuid, asset_id: Uuid) -> Result<Option<(Asset, String)>>` (Returns Asset + CatalogName).

#### 1.2 `pangolin_store/src/memory.rs`
- **Fix**: Update `create_asset` to populate `self.assets_by_id`.
- **Implement**: `get_asset_by_id` using the `assets_by_id` DashMap.
- **verify**: Ensure `delete_asset` removes from index (already seems to, but verify).

#### 1.3 Other Stores (Postgres, Mongo, Sqlite)
- **Implement**: Add `get_asset_by_id` implementation (or strict TODO stub) to satisfy trait definition.

---

## 2. Security: Granular Permissions

**Problem**: Handlers use simplified "Is Admin?" checks.
**Solution**: Implement resource-scoped permission checks.

### Changes

#### 2.1 `pangolin_api/src/business_metadata_handlers.rs`

**Endpoint: `add_business_metadata`**
- **Current**: Checks `is_admin()`.
- **New Logic**:
    1. Call `store.get_asset_by_id(asset_id)` (New Optimized Method).
    2. Extract `catalog_name`.
    3. Check `MANAGE_DISCOVERY` permission scoped to that `catalog_id`.

**Endpoint: `search_assets`**
- **Current**: Checks `is_admin()` OR `discoverable`.
- **New Logic**:
    1. Fetch user permissions: `store.list_user_permissions(user_id)`.
    2. In search loop, filter:
       - Keep if `discoverable == true`
       - OR `has_permission(user_perms, READ, asset.catalog, asset.namespace)`

---

## 3. Verification Plan

### 3.1 Live Integration Test (`tests/scripts/test_phase1_verification.py`)

A new Python script using `requests` to simulate a real client verification flow.

**Scenario 1: Performance (Asset Lookup)**
1. **Setup**: Create 1000 assets in `MemoryStore`.
2. **Action**: Call `GET /api/v1/assets/{id}` for a random specific asset.
3. **Verify**: Response is successful and fast. (Validates `get_asset_by_id` is working).

**Scenario 2: Granular Security**
1. **Setup**:
    - Create `restricted_user`.
    - Create `CatalogA` (User has NO access) with `SecretTable` (Not discoverable).
    - Create `CatalogB` (User has READ access) with `PublicTable`.
2. **Action**:
    - `restricted_user` searches for "Table".
3. **Verify**:
    - `SecretTable` is **NOT** in results.
    - `PublicTable` **IS** in results.
    - (Prove filtering works).
4. **Action**:
    - `restricted_user` tries to set "discoverable" flag on `SecretTable`.
5. **Verify**:
    - Response is `403 Forbidden`.
    - (Prove `MANAGE_DISCOVERY` check works).

### 3.2 Regression Testing
- Run existing full suite: `python3 tests/scripts/test_audit_fixes.py` (rename/update if needed).
- Ensure no existing flows (Branching, User CRUD) are broken by Trait changes.
