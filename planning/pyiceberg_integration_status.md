# PyIceberg Backend Integration Status

This document tracks the verification of PyIceberg functionality across all Pangolin storage backends.

## Test Matrix

| Backend | Auth Mode | Vending (With Warehouse) | Client Creds (No Warehouse) | Create | Write | Read | Update | Status |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Memory** | ✅ | ✅ (Create/Write/Read) ❌ (Update) | ✅ (Create/Write/Read) ❌ (Update) | ✅ | ✅ | ✅ | ❌ | 🟡 Partial |
| **SQLite** | ✅ | ❌ Failed (Create Table 500) | ❌ Failed | ❌ | ❌ | ❌ | ❌ | 🔴 Failed |
| **Postgres** | ✅ | ❌ Failed (Create Table 500) | ❌ Failed | ❌ | ❌ | ❌ | ❌ | 🔴 Failed |
| **Mongo** | ✅ | ❌ Failed (Create Tenant 500) | ❌ Failed | ❌ | ❌ | ❌ | ❌ | 🔴 Failed |

## Detailed Findings

### MemoryStore
*   **Vending**:
    *   Create Namespace/Table: ✅
    *   Write Data: ✅
    *   Read Data: ✅
    *   Update Schema: ❌ Failed with `RESTError 422: ... unknown variant 'assert-current-schema-id'`.
*   **Client Creds**:
    *   Same results as Vending. Update fails with same error.

### SQLiteStore
*   **Vending**:
    *   Auth/Warehouse/Catalog/Namespace: ✅
    *   Create Table: ❌ Failed (500 Internal Server Error: "Failed to write metadata ... Invalid JSON").
*   **Client Creds**: ❌ Failed (Same error).

### PostgresStore
*   **Vending**:
    *   Auth/Warehouse/Catalog/Namespace: ✅
    *   Create Table: ❌ Failed (500 Internal Server Error: "Failed to write metadata ... Invalid JSON").
*   **Client Creds**: ❌ Failed (Same error).

### MongoStore
*   **Vending**: ❌ Failed at Tenant Creation (500 Internal Server Error).
*   **Client Creds**: ❌ Failed (Same error).

## Issues & Notes
*   **Constraint Violations**: Postgres and SQLite tests initially failed due to generated ID mismatches and unique constraints on users. Randomizing users and using server-returned IDs fixed this.
*   **JSON Serialization**: SQLite and Postgres fail with "Invalid JSON" when writing metadata, likely an issue with how the `DashMap` or `Sqlx` implementation handles JSONB serialization for iceberg metadata.
*   **Update Schema**: Fails on MemoryStore due to missing `assert-current-schema-id` support in the commit logic.
