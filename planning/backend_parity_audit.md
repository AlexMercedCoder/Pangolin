# Backend Parity Audit

## Executive Summary
This audit identifies critical gaps in feature implementation across Pangolin's four backend stores: `MemoryStore`, `PostgresStore`, `MongoStore`, and `SqliteStore`. The primary issue is that **the API layer is already fully implemented** for Service Users, System Settings, and enhanced Audit Logs, but **only MemoryStore has complete backend support**. This causes runtime failures when using PostgreSQL, MongoDB, or SQLite in production.

**Critical Finding**: The UI is currently broken in production (PostgreSQL) mode due to missing backend implementations, not missing API endpoints.

---

## Summary of Findings

| Feature | Memory | Postgres | Mongo | SQLite | API Exists? | CLI Exists? | UI Exists? | PyPangolin Exists? |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **Core Catalog** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **User Management** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Role Management** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Service Users** | ✅ | ✅ ✓ | ✅ ✓ | ❌ | ✅ | ❌ | ✅ | ✅ |
| **System Settings** | ✅ | ✅ ✓ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ |
| **Audit Logs (Enhanced)** | ✅ | ✅ ✓ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| **Token Management** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Merge Operations** | ✅ | ✅ ✓ | ✅ ✓ | ❌ | ✅ | ✅ | ⚠️ | ❌ |
| **Business Metadata** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

**Legend**: ✅ = Complete, ✅ ✓ = Complete with Regression Tests, ❌ = Missing, ⚠️ = Partial

---

## Detailed Gaps & Impact Analysis

### 1. Service Users (Machine-to-Machine Auth)
**Current State**: API fully implemented, UI functional, pypangolin client exists.

**Backend Status**:
- ✅ **MemoryStore**: Complete implementation
- ✅ **PostgresStore**: **COMPLETE** with regression tests (15 tests)
- ✅ **MongoStore**: **COMPLETE** with regression tests (7 tests)
- ❌ **SqliteStore**: Methods missing entirely

**PostgreSQL Implementation** (✅ Complete - Dec 26, 2025):
- ✅ Migration: `20251226000000_add_service_users_audit_system.sql`
- ✅ All 7 trait methods implemented in `postgres.rs`
- ✅ Regression tests: `tests/postgres_parity_tests.rs`
- ✅ Verified: Create, Get, List, Update, Delete, Get by API key hash, Update last used

**API Surface** (Already Exists):
- `POST /api/v1/service-users` - Create service user
- `GET /api/v1/service-users` - List service users
- `GET /api/v1/service-users/{id}` - Get service user
- `PUT /api/v1/service-users/{id}` - Update service user
- `DELETE /api/v1/service-users/{id}` - Delete service user
- `POST /api/v1/service-users/{id}/rotate` - Rotate API key

**Files**:
- API: `pangolin_api/src/service_user_handlers.rs` (✅ Complete with utoipa docs)
- Core: `pangolin_core/src/user.rs` (✅ `ServiceUser` struct defined)
- Store: `pangolin_store/src/postgres.rs` (✅ All methods implemented)
- Tests: `pangolin_store/src/tests/postgres_parity_tests.rs` (✅ 15 regression tests)
- Migration: `pangolin_store/migrations/20251226000000_add_service_users_audit_system.sql` (✅ Applied)
- UI: `pangolin_ui/src/routes/+layout.svelte` (✅ Service Users nav link exists)
- PyPangolin: `pypangolin/src/pypangolin/governance.py` (✅ `ServiceUserClient` implemented)

**Impact of Fixing**:
- ✅ **API**: No changes needed - already complete
- ✅ **utoipa/OpenAPI**: No regeneration needed - already documented
- ❌ **CLI**: **NEEDS IMPLEMENTATION** - No CLI commands exist for service users
- ✅ **UI**: No changes needed - already functional
- ✅ **pypangolin**: No changes needed - already implemented

**Action Required**:
1. ~~Implement `service_users` table in PostgreSQL migration~~ ✅ DONE
2. ~~Implement 7 trait methods in `PostgresStore`~~ ✅ DONE
3. ~~Implement 7 trait methods in `MongoStore`~~ ✅ DONE (Dec 26, 2025)
4. Port implementation to `SqliteStore`
5. **NEW**: Implement CLI commands for service user management

---

### 2. System Settings
**Current State**: API fully implemented, UI functional.

**Backend Status**:
- ✅ **MemoryStore**: Complete
- ✅ **PostgresStore**: **COMPLETE** with regression tests (3 tests)
- ✅ **MongoStore**: Complete
- ✅ **SqliteStore**: Complete

**PostgreSQL Implementation** (✅ Complete - Dec 26, 2025):
- ✅ Migration: `20251226000000_add_service_users_audit_system.sql`
- ✅ Both trait methods implemented in `postgres.rs`
- ✅ Regression tests: `tests/postgres_parity_tests.rs`
- ✅ Verified: Get default settings, Update settings, Upsert behavior

**API Surface** (Already Exists):
- `GET /api/v1/config/settings` - Get system settings
- `PUT /api/v1/config/settings` - Update system settings

**Files**:
- API: `pangolin_api/src/system_config_handlers.rs` (✅ Complete with utoipa docs)
- Core: `pangolin_core/src/model.rs` (✅ `SystemSettings` struct defined)
- UI: Routes exist for system config

**Impact of Fixing**:
- ✅ **API**: No changes needed
- ✅ **utoipa/OpenAPI**: No regeneration needed
- ❌ **CLI**: **NEEDS IMPLEMENTATION** - No CLI commands exist
- ✅ **UI**: No changes needed
- ❌ **pypangolin**: **NEEDS IMPLEMENTATION** - No client methods exist

**Action Required**:
1. ~~Implement `system_settings` table in PostgreSQL migration~~ ✅ DONE
2. ~~Implement 2 trait methods in `PostgresStore` (get, update)~~ ✅ DONE
3. **NEW**: Implement CLI commands for system settings
4. **NEW**: Add `SystemSettingsClient` to pypangolin

---

### 3. Audit Logs (Enhanced Schema)
**Current State**: API exists, PostgreSQL schema updated.

**Backend Status**:
- ✅ **MemoryStore**: Uses current `AuditLogEntry` struct
- ✅ **PostgresStore**: **COMPLETE** with regression tests (2 tests)
- ✅ **MongoStore**: Schema-less, handles current struct
- ✅ **SqliteStore**: Appears to use current schema

**PostgreSQL Implementation** (✅ Complete - Dec 26, 2025):
- ✅ Migration: `20251226000000_add_service_users_audit_system.sql`
- ✅ Schema updated with 8 new columns
- ✅ Renamed columns: `actor` → `username`, `resource` → `resource_name`
- ✅ Regression tests: `tests/postgres_parity_tests.rs`
- ✅ Verified: Enhanced schema logging, Filtering by user_id

**Schema Mismatch**:
```sql
-- Current Postgres (OUTDATED)
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    actor TEXT NOT NULL,          -- Should be 'username'
    action TEXT NOT NULL,
    resource TEXT,                 -- Should be 'resource_name'
    details JSONB
);

-- Required (NEW COLUMNS)
ALTER TABLE audit_logs ADD COLUMN user_id UUID;
ALTER TABLE audit_logs ADD COLUMN resource_type TEXT;
ALTER TABLE audit_logs ADD COLUMN resource_id UUID;
ALTER TABLE audit_logs ADD COLUMN ip_address TEXT;
ALTER TABLE audit_logs ADD COLUMN user_agent TEXT;
ALTER TABLE audit_logs ADD COLUMN result TEXT;
ALTER TABLE audit_logs ADD COLUMN error_message TEXT;
ALTER TABLE audit_logs ADD COLUMN metadata JSONB;
ALTER TABLE audit_logs RENAME COLUMN actor TO username;
ALTER TABLE audit_logs RENAME COLUMN resource TO resource_name;
```

**Impact of Fixing**:
- ✅ **API**: No changes needed
- ✅ **utoipa/OpenAPI**: No regeneration needed
- ✅ **CLI**: Already implemented
- ✅ **UI**: No changes needed
- ❌ **pypangolin**: **NEEDS IMPLEMENTATION** - No audit log client exists

**Action Required**:
1. ~~Create migration to update `audit_logs` table schema~~ ✅ DONE
2. ~~Update `log_audit_event` and `list_audit_events` in `PostgresStore`~~ ✅ DONE (methods were already updated)
3. **NEW**: Add `AuditLogClient` to pypangolin

---

### 4. Merge Operations & Conflicts
**Current State**: API exists, CLI exists, UI partial.

**Backend Status**:
- ✅ **MemoryStore**: Complete implementation
- ✅ **PostgresStore**: **COMPLETE** with regression tests (12 tests)
- ✅ **MongoStore**: **COMPLETE** with regression tests (11 tests)
- ❌ **SqliteStore**: Methods missing entirely

**PostgreSQL Implementation** (✅ Complete - Dec 26, 2025):
- ✅ Migration: `20251226010000_add_merge_operations.sql`
- ✅ All 11 trait methods implemented in `postgres.rs` (6 for operations, 5 for conflicts)
- ✅ Regression tests: `tests/postgres_merge_tests.rs`
- ✅ Verified: All CRUD operations for both MergeOperation and MergeConflict

**MongoDB Implementation** (✅ Complete - Dec 26, 2025):
- ✅ All 11 trait methods implemented in `mongo.rs`
- ✅ Regression tests: `tests/mongo_parity_tests.rs`
- ✅ Live tested: Verified via API with MongoDB backend
- ✅ No migration needed (schema-less)

**Impact of Fixing**:
- ✅ **API**: No changes needed
- ✅ **utoipa/OpenAPI**: No regeneration needed
- ✅ **CLI**: Already implemented
- ⚠️ **UI**: Partial implementation
- ❌ **pypangolin**: **NEEDS IMPLEMENTATION**

**Action Required**:
1. ~~Implement `merge_operations` and `merge_conflicts` tables in PostgreSQL~~ ✅ DONE
2. ~~Implement 11 trait methods in `PostgresStore`~~ ✅ DONE
3. ~~Implement 11 trait methods in `MongoStore`~~ ✅ DONE (Dec 26, 2025)
4. Port implementation to `SqliteStore`
5. **NEW**: Add merge operation support to pypangolin

---

## Recommendations

### Immediate Priorities (Fix Production)
1. **PostgresStore Service Users**: Unblock UI service user management
2. **PostgresStore System Settings**: Unblock UI system config
3. **PostgresStore Audit Logs Schema**: Fix data loss and enable proper audit trails

### Medium-Term (Feature Parity)
4. **MongoStore & SqliteStore Service Users**: Enable multi-backend support
5. **CLI for Service Users & System Settings**: Enable headless management
6. **PyPangolin Audit & Settings Clients**: Enable SDK-based management

### Long-Term (Architecture)
7. **Shared Integration Tests**: Prevent future parity drift
8. **Merge Operations Persistence**: Complete advanced branching features

---

## Context for Future Sessions

### Key Files to Review
1. **Trait Definition**: `pangolin_store/src/lib.rs` - `CatalogStore` trait
2. **MemoryStore Reference**: `pangolin_store/src/memory.rs` - Complete implementation
3. **PostgresStore**: `pangolin_store/src/postgres.rs` - Needs updates
4. **Migration Template**: `pangolin_store/migrations/20251226000000_add_service_users_audit_system.sql` - Already created
5. **API Handlers**: `pangolin_api/src/service_user_handlers.rs`, `system_config_handlers.rs`

### Current Error Messages
- Service Users: `"Operation not supported by this store"`
- System Settings: `"relation "system_settings" does not exist"`
- Audit Logs: `"column "user_id" does not exist"`

### Testing Strategy
1. **Compile**: `cargo check -p pangolin_store`
2. **Migration**: Start API to apply migrations
3. **API Test**: `curl -X GET http://localhost:8080/api/v1/service-users -H "Authorization: Bearer $TOKEN"`
4. **UI Test**: Navigate to Service Users page, verify no errors

### Database Connection
```bash
# PostgreSQL (Docker)
DATABASE_URL=postgres://admin:password@localhost:5432/pangolin

# Apply migrations automatically on API startup
cd pangolin && cargo run
```

### Why This Matters
- **Production Impact**: UI is broken for PostgreSQL users
- **Multi-Tenancy**: Service users are critical for machine-to-machine auth
- **Compliance**: Enhanced audit logs are required for enterprise deployments
- **Backend Flexibility**: Users expect to switch stores without losing features
