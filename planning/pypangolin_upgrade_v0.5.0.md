# PyPangolin Upgrade Plan (v0.5.0)

**Date**: 2025-12-29
**Parent Doc**: [Client & API Upgrade Audit](./client_upgrade_audit_v0.5.0.md)
**Status**: Planning

## Overview
This plan details the upgrades required for the `pypangolin` Python SDK to support the new features introduced in Pangolin v0.5.0, specifically **Service User Management** and **API Key Authentication**.

## Objectives
1.  **Authentication**: Support `X-API-Key` header authentication in `PangolinClient`.
2.  **Service Users**: Implement `ServiceUsersClient` for full lifecycle management.
3.  **RBAC**: Ensure `assign_role` and `revoke_role` methods work with Service User IDs.
4.  **Verification**: Create a comprehensive verification script using `pypangolin`.

## 1. Authentication Updates
The backend `auth_middleware.rs` supports `X-API-Key`. The SDK must allow initializing a client with an API key instead of (or in addition to) a Bearer token.

*   **File**: `pypangolin/src/client.py` (or `core.py` depending on structure)
*   **Changes**:
    *   Update `__init__` to accept `api_key` (optional).
    *   If `api_key` is present, set `X-API-Key` header on all requests.
    *   Ensure `api_key` auth bypasses the `/login` flow (Service Users don't login to get a JWT, they use the key directly).

## 2. Service User Client
Implement a new sub-client `ServiceUsersClient` attached to `PangolinClient.service_users`.

*   **File**: `pypangolin/src/service_users.py` (New File)
*   **Methods**:
    *   `create(name: str, role: str, description: Optional[str] = None, expires_in_days: Optional[int] = None) -> ServiceUserResponse`
    *   `list(limit: int = 20, offset: int = 0) -> List[ServiceUser]`
    *   `get(id: str) -> ServiceUser`
    *   `update(id: str, name: Optional[str] = None, ...) -> ServiceUser`
    *   `delete(id: str) -> None`
    *   `rotate_key(id: str) -> ServiceUserKeyResponse`

## 3. RBAC Updates
*   **File**: `pypangolin/src/users.py` (or `security.py`)
*   **Check**: Ensure `assign_role` methods verify UUID format but do **not** assume "User" vs "Service User" distinction (the backend handles foreign key logic).
*   **Validation**: The current SDK logic likely just passes the ID string. Verify no regex blocks it.

## 4. Verification Plan
Create `scripts/verify_pypangolin_v0_5_0.py`.

### Scenario
1.  **Admin Auth**: Initialize `PangolinClient` with Admin credentials (JWT).
2.  **Create Service User**: Create `py_bot`, capture API Key.
3.  **Service User Auth**: Initialize **new** `PangolinClient` with `api_key`.
4.  **Operation Check**:
    *   Use the `py_bot` client to `list_warehouses` (should succeed if `tenant-user` role).
    *   Use the `py_bot` client to `create_warehouse` (should **fail** with 403, as `tenant-user` cannot create warehouses, only `tenant-admin` can).
5.  **RBAC Check**:
    *   Admin client assigns `tenant-admin` role to `py_bot`.
    *   `py_bot` client retries `create_warehouse` (should **succeed**).
6.  **Cleanup**: Admin client deletes `py_bot`.

## Implementation Checklist
- [ ] Update `PangolinClient` for `api_key` support.
- [ ] Create `pypangolin/src/service_users.py`.
- [ ] Expose `service_users` in main client.
- [ ] Create verification script.
- [ ] Run verification.
