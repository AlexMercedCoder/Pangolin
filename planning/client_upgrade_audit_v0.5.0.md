# Client & API Upgrade Audit (v0.5.0)

**Date**: 2025-12-29
**Scope**: Verification of API changes from Backend Audit and identification of required upgrades for CLI, Python SDK, and UI.

## 1. API Contract & Utoipa Audit

### Findings
The API contract has remained stable with **additive changes only**. No breaking changes were introduced to existing endpoints.

*   **Service User Endpoints (`/api/v1/service-users/*`)**:
    *   **Status**: Fully implemented.
    *   **Utoipa**: Correctly annotated with `#[utoipa::path]` tags.
    *   **Security**: Correctly restricted to `TenantAdmin`.

*   **Role & Permission Endpoints (`/api/v1/roles`, `/api/v1/permissions`)**:
    *   **Status**: Logic updated to accept Service User IDs (via Foreign Key relaxation).
    *   **Utoipa**: Annotations in `permission_handlers.rs` are accurate.
    *   **Serialization**: Requests use `kebab-case` (`tenant-id`, `role-id`). This was pre-existing and is consistent.

*   **Listing Endpoints**:
    *   **Status**: `list_assets` now deterministically ordered.
    *   **Utoipa**: No changes needed (Output schema checks passed).

### Recommendation
*   **No immediate Utoipa action required.** The OpenAPI spec generated from these annotations should be correct.

---

## 2. Recommended Client Upgrades

While existing clients will continue to function, they cannot yet expose the new *features* (Service Users). The following upgrades are recommended for the next release cycle.

### A. Pangolin CLI
**Priority**: Low (Completed)
**Status**: ✅ **Verified** (v0.5.0 CLI Upgrade Complete)

*   **Implemented Commands**: `create-service-user`, `list-service-users`, `rotate-service-user-key`, `delete-service-user`.
*   **RBAC**: `assign-role`, `revoke-user-role` verified.
*   **Verification**: `scripts/verify_cli_regression_v0_5_0.py` passes.

### B. Pangolin UI
**Priority**: Medium (Admin Convenience)

*   **New Settings Page**: `Settings -> Service Users`
    *   **List View**: Table of service users (Name, Role, Created At, Active status).
    *   **Create Modal**: Form for Name, Description, Role, Expiration.
        *   **Action**: Must display the generated API Key **once** in a copyable field upon success.
    *   **Rotate Action**: Button to rotate key (with warning dialog), displaying new key.
    *   **Revoke/Delete Action**.

### C. Python SDK (`pypangolin`)
**Priority**: High (Completed)
**Status**: ✅ **Verified** (v0.5.0)

*   **Verified Features**:
    *   `ServiceUser` management (Create, List, Get, Update, Delete, Rotate).
    *   `X-API-Key` Authentication support.
    *   Granular Permission Granting (via Tenant Admin).
    *   RBAC Enforcement (Tenant User blocked from Warehouse creation).
    *   Model parity with v0.5.0 backend (Attributes: `description`, `role`, `active`).

*   **Core Logic**:
    *   Update `PangolinClient` to authenticate via `x-api-key` header if provided (instead of Bearer token) to support using the very keys we generate.
    *   Currently, `verify_regression_v0_5_0.py` proves basic Bearer auth works for Service Users (if they login via `/login`? No, Service Users use API Keys usually).
    *   *Correction*: The current Service User implementation generates an API Key hash. We need to ensure the **Auth Middleware** supports API Key authentication (checking `Authorization: ApiKey <KEY>` or similar). **This might be a missing backend feature if not already present.**

### Critical Note on Authentication
**Status**: ✅ **Confirmed**.
The `auth_middleware.rs` explicitly checks for the `X-API-Key` header.
*   **Action**: PyPangolin must implement `X-API-Key` header support.

**Action Item**: Proceed to detailed [PyPangolin Upgrade Plan](./pypangolin_upgrade_v0.5.0.md).
