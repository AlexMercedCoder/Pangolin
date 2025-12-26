# Backend Parity Audit

## Goal
To identify gaps in feature implementation across the four supported backend stores: `MemoryStore`, `PostgresStore`, `MongoStore`, and `SqliteStore`. The goal is to ensure full feature parity so that users can switch backends without losing functionality.

## Summary of Findings

| Feature | Memory | Postgres | Mongo | SQLite |
| :--- | :---: | :---: | :---: | :---: |
| **Core Catalog (Tenants, Warehouses, Catalogs, Assets)** | ✅ | ✅ | ✅ | ✅ |
| **User Management** | ✅ | ✅ | ✅ | ✅ |
| **Role Management** | ✅ | ✅ | ✅ | ✅ |
| **Service Users** | ✅ | ❌ MISSING | ❌ MISSING | ❌ MISSING |
| **System Settings** | ✅ | ❌ MISSING | ✅ | ✅ |
| **Audit Logs (Schema)** | ✅ | ⚠️ OUTDATED | ✅ | ✅ |
| **Token Management** | ✅ | ✅ | ✅ | ✅ |
| **Merge Operations (Conflicts, etc.)** | ✅ | ❌ MISSING | ❌ MISSING | ❌ MISSING |
| **Business Metadata** | ✅ | ✅ | ✅ | ✅ |

## Detailed Gaps

### 1. Service Users
The `ServiceUser` feature (for machine-to-machine authentication) is currently only implemented in `MemoryStore`.
- **PostgresStore**: Methods return "Operation not supported". No table implementation.
- **MongoStore**: Methods missing from implementation.
- **SqliteStore**: Methods missing from implementation.

**Action Required**:
- Implement `ServiceUser` structs and methods for all persistent stores.
- Create `service_users` table/collection in schemas.

### 2. System Settings
Configuration for system-wide behaviors (e.g., default buckets, SMTP).
- **PostgresStore**: Missing `system_settings` table and method implementations.
- **MongoStore**: Implemented.
- **SqliteStore**: Implemented.

**Action Required**:
- Implement `system_settings` table and methods for key-value storage in Postgres.

### 3. Audit Logs
- **PostgresStore**: Use of an outdated schema. The `AuditLogEntry` struct has evolved (adding `user_id`, `resource_type`, `ip_address`, etc.), but the Postgres table is missing these columns, causing inserts to fail or lose data.
- **Mongo/Sqlite**: Appear to handle the struct correctly (Mongo is schema-less, Sqlite likely updated or generic).

**Action Required**:
- Migrate Postgres `audit_logs` table to match current `AuditLogEntry` fields.

### 4. Merge Operations & Conflicts
Advanced branching features including merge conflict resolution.
- **Postgres/Mongo/Sqlite**: Likely missing robust implementation for `MergeOperation` and `MergeConflict` persistence compared to `MemoryStore`.

**Action Required**:
- Verify and implement persistence for merge operations and conflict resolution states.

## Recommendations

1.  **Prioritize Postgres**: As the primary production store, `PostgresStore` must be brought up to parity immediately. This is addressed in the current `implementation_plan.md`.
2.  **Standardize Trait Tests**: Create a shared integration test suite that runs against ALL stores to enforce parity automatically. Currently, features are often added to Memory/Postgres but missed in others.
3.  **Refactor Mongo/Sqlite**: Once Postgres is fixed, port the logic to Mongo and SQLite to maintain the "run anywhere" promise of Pangolin.
