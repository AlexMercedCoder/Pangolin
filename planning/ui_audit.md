# UI Audit Report

**Date**: 2025-12-17
**Scope**: `pangolin_ui` (SvelteKit)

## Overview
The UI has progressed significantly since the previous audit (2025-12-14), with placeholders for branching and discovery now being filled. However, it still lacks exposure for some advanced management and security features.

## Feature Status

| Feature | Status | Route/Component | Missing/Issues |
| :--- | :---: | :--- | :--- |
| **Token Generation** | ❌ | None | No UI for generating API tokens (`/api/v1/tokens`). |
| **Federated Catalogs** | ❌ | None | `catalogsApi` and creation form lack federated fields. |
| **Branching** | 🚧 | `/branches`, `/branches/new` | API client exists; UI implementation is partial. |
| **Business Metadata** | 🚧 | `/search`, `/discovery` | `business_metadata.ts` exists; UI is in progress. |
| **CRUD (Core)** | ✅ | `/tenants`, `/warehouses`, `/catalogs` | Basic listing and creation functional. |
| **RBAC / Permissions** | 🚧 | `/permissions`, `/roles` | Routes and API clients exist; UI implementation in progress. |
| **Access Requests** | 🚧 | `/access-requests` | Route exists; logic integration in progress. |

## Gap Analysis & Recommendations

### 1. Token Generation UI
**Gap**: There is no dedicated page for users to manage their own API tokens or for admins to generate tokens for service users.
**Recommendation**: Create a "Profile" or "Tokens" page under user settings to call `POST /api/v1/tokens`.

### 2. Federated Catalog Support
**Gap**: The "Create Catalog" form assumes internal `pangea` type catalogs only.
**Recommendation**: Update the catalog creation form to support "Internal" vs "Federated" toggles. Federated mode should expose fields for `base_url`, `auth_type`, and `credentials`.

### 3. Branching Full-Cycle
**Gap**: While branching routes exist, the implementation of "Partial Branching" (selecting specific assets) is not fully exposed in the `new branch` form.
**Recommendation**: Add asset multi-select to the `new branch` form.

### 4. Edit/Update Capabilities
**Gap**: Most core entities (Tenants, Warehouses, Catalogs) support creation but lack "Edit" or "Delete" buttons in the UI despite API support.
**Recommendation**: Add Edit and Delete actions to the DataTables.

## Conclusion
The UI is approximately 60% complete in terms of feature exposure. The most critical missing pieces are **Federated Catalog management** and **Token generation/management**, followed by completing the **Partial Branching** and **RBAC** UI flows.
