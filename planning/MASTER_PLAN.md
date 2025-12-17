# Pangolin Master Plan & Outstanding Work

**Last Updated**: 2025-12-17
**Status**: Consolidated from Audit & Cloud Strategy docs.

This document serves as the **single source of truth** for remaining work on the Pangolin project, consolidating findings from the implementation audit and the cloud strategy roadmap.

---

## 🏗️ Phase 1: Core System Hardening (Vital)

These items address direct security and performance gaps identified in the audit.

### 1.1 Security: Granular Permissions (Partial Implementation)
- **Current State**: Simplified "Is Admin?" checks exist.
- **Goal**: Implement granular, resource-scoped permission checks.
- **Tasks**:
  - [ ] **MANAGE_DISCOVERY Check**: Implement per-catalog permission check for setting discoverability in `business_metadata_handlers.rs`.
  - [ ] **Search Filtering**: Filter search results based on user's specific read permissions for each asset, not just "is discoverable".

### 1.2 Performance: Asset Lookup Optimization (Missing)
- **Current State**: `get_asset_details` uses nested loops (O(n*m*k)). `MemoryStore` has a broken/partial `assets_by_id` index.
- **Goal**: O(1) or O(log n) asset lookup by ID verification.
- **Tasks**:
  - [ ] **Fix MemoryStore Index**: Ensure `assets_by_id` is updated on `create_asset` (currently only updated on rename).
  - [ ] **Implement Trait Method**: Implement `get_asset_by_id` in `CatalogStore` trait for all backends.
  - [ ] **Update Handlers**: Refactor `business_metadata_handlers.rs` to use the optimized lookup.

---

## 🚀 Phase 2: Feature Completeness

### 2.1 Token Management
- **Goal**: Secure session management.
- **Tasks**:
  - [ ] **Token Blacklist**: Implement mechanism to revoke tokens (Redis or DB-backed blacklist/refresh token pattern).

### 2.2 Client-Side Updates (from API Impact Strategy)
- **Goal**: Make clients aware of the new Multi-Catalog API capabilities.
- **Tasks**:
  - [ ] **CLI**: Add global `--catalog` flag to all asset commands.
  - [ ] **UI**: Add "Active Catalog" context selector in navigation.
  - [ ] **UI**: Update data fetching to use selected catalog context.

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

## ✅ Recent Completions (Reference)
- **User CRUD**: Full RBAC implementation for User management.
- **Deletion**: Federated Catalog deletion logic.
- **Merge**: Merge base commit detection (3-way merge support).
