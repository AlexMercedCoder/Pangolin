
# Backend Audit & Optimization Plan (v0.5.0)

**Date**: December 29, 2025
**Scope**: SQLite, PostgreSQL, MongoDB, Memory
**Goal**: Identify architectural bottlenecks, schema limitations, and consistency issues without breaking API contracts.

## 1. Critical Issues (SQL Backends)

These issues affect SQLite and PostgreSQL implementations. MongoDB and Memory backends are largely immune due to their document/object-based nature.

### 1.1. Token Listing for Ephemeral & Service Users
*   **Problem**: `list_active_tokens` relies on an `INNER JOIN` with the `users` table to filter by `tenant_id`.
*   **Impact**:
    *   **Root Users**: Since Root users are often ephemeral (config-based) and not in the `users` table, their tokens cannot be listed via the API, returning empty lists.
    *   **Service Users**: Service Users reside in a separate `service_users` table. If they generate JWTs (e.g. via API Key exchange), those tokens cannot be listed under `tenant_id` scope because the `JOIN users` fails.
*   **Root Cause**: Lack of denormalized `tenant_id` on the `active_tokens` table.
*   **Fix**: Add `tenant_id` column to `active_tokens`.
    ```sql
    ALTER TABLE active_tokens ADD COLUMN tenant_id UUID; -- Populate from users or context
    CREATE INDEX idx_tokens_tenant ON active_tokens(tenant_id);
    ```

### 1.2. RBAC Incompatibility with Service Users
*   **Problem**: The `user_roles` and `permissions` tables have strict Foreign Key constraints referencing `users(id)`.
*   **Impact**:
    *   **Assignment Failure**: You cannot assign a Role (`user_roles`) or Direct Permission (`permissions`) to a Service User because their ID exists in `service_users`, not `users`. The database will reject the INSERT with a Foreign Key Violation.
    *   **Functional Gap**: Service Users are effectively second-class citizens that cannot use the fine-grained / dynamic RBAC system; they are limited to their static `role` field (`Root`, `TenantAdmin`, `TenantUser`).
*   **Root Cause**: The schema assumes all principals are in the `users` table.
*   **Fix Strategies**:
    1.  **Remove FKs**: Drop the `REFERENCES users(id)` constraint. (Risky - data integrity).
    2.  **Polymorphic FKs**: Hard in SQL.
    3.  **Unified Principals Table** (Recommended): Introduce a `principals` table that both `users` and `service_users` inherit from or reference, and point permissions to `principals`.
    4.  **Short-term Fix**: Create `service_user_roles` and `service_user_permissions` tables (messy).

## 2. Optimization Opportunities

### 2.1. Denormalization for Tenant Isolation
Most "List" operations filter by `tenant_id`. In SQL backends, strict normalization forces JOINs that are expensive or brittle (as seen above).
*   **Recommendation**: Ensure `tenant_id` is present on ALL tenant-scoped resources.
    *   `permissions`: Missing `tenant_id`. Add it.
    *   `user_roles`: Currently has no `tenant_id` (implicit via Role?). Roles have `tenant_id`. A JOIN is acceptable here (`user_roles JOIN roles`) but denormalizing `tenant_id` to `user_roles` could speed up "List all my roles in this tenant" queries.

### 2.2. Indexing Gaps
*   **Permissions**: `permissions` table lacks an index on `user_id`.
    *   *Impact*: `list_user_permissions` performs a sequential scan on the permissions table.
    *   *Fix*: `CREATE INDEX idx_permissions_user ON permissions(user_id);`
*   **Tags**: `tags` table PK is `(tenant_id, catalog_name, name)`. This acts as an index. Optimal.

### 2.3. Audit Log Partitioning (Postgres)
*   **Observation**: `audit_logs` will grow indefinitely.
*   **Recommendation**: For Postgres, implement table partitioning by `timestamp` (e.g., monthly). This allows for efficient dropping of old logs (Data Retention compliance).
*   **Action**: Create a partitioned table migration script for future scaling.

## 3. MongoDB Specifics
*   **Status**: Healthy.
*   **Observation**: MongoDB implementation essentially duplicates `tenant_id` on every document. This is standard NoSQL practice and prevents the issues seen in SQL.
*   **Verification**: `permissions` collection in Mongo stores `user_id` as a UUID. Since Mongo doesn't enforce strict FKs by default, Service Users *can* technically be assigned permissions, but the `list_permissions` query might still need verification if it tries to `$lookup` the user.
    *   *Check*: `list_permissions` in Mongo implementation.

## 4. Implementation Plan (v0.5.0)

### Phase 1: Storage Parity (High Priority)
1.  [x] **Schema Migration (SQL)**: Add `tenant_id` to `active_tokens`.
2.  [x] **Schema Migration (SQL)**: Add `tenant_id` to `permissions`.
3.  [x] **Code Update**: Update `list_active_tokens` to filter by `active_tokens.tenant_id` (Remove JOIN).
4.  [x] **Code Update**: Update `list_permissions` to filter by `permissions.tenant_id` (Remove JOIN).

### Phase 2: RBAC for Service Users
1.  [x] **Schema Migration (PostgreSQL)**: Drop Foreign Key constraints on `user_roles.user_id` and `permissions.user_id` to allow Service User IDs.
2.  [x] **Schema Update (SQLite)**: Update `sqlite_schema.sql` to remove FK constraints for new installs. (Migration for existing is manual/hard, documented in Known Issues).
3.  [x] **Store Verification**: Ensure `assign_role` and `grant_permission` accept Service User IDs (which are valid UUIDs but not in `users` table).
4.  [x] **Principal Verification**: Update API handlers to verify principal existence in EITHER `users` or `service_users` table before granting role/permission (to prevent garbage IDs).


### Phase 3: Performance
1.  [ ] **Indexing (Postgres/SQLite)**: Add missing index on `permissions(user_id)` to speed up `list_user_permissions`.
2.  [ ] **Query Optimization**: Audit and optimize `list_assets` and `list_catalogs` queries to ensure they leverage existing compound indexes.
3.  [ ] **Audit Log Partitioning (Postgres)**: Create a migration to convert `audit_logs` to a partitioned table (by time) for scalable retention management.


