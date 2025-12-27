# API Optimization Recommendations

## Executive Summary
An audit of the `pangolin_api` and `pangolin_store` crates revealed several opportunities for significant performance improvements. While the current implementation is functional and modular, specific architectural patterns—particularly around data access loops and blocking I/O—will likely become bottlenecks at scale.

## 1. Algorithmic & Logical Optimizations

### 1.1 Eliminate N+1 Query Patterns
**Severity:** High
**Location:** `pangolin_api/src/pangolin_handlers.rs` (create_branch), `pangolin_api/src/iceberg/tables.rs` (list_tables credentials)

*   **Issue:** The branching logic iterates through namespaces and lists assets one-by-one to copy them.
*   **Recommendation:** Implement `store.copy_assets_bulk(source_branch, target_branch)` to push the loop down to the database layer. This allows a single SQL `INSERT INTO ... SELECT ...` statement, reducing network round-trips from O(N) to O(1).

### 1.2 Optimize Commit History Traversal
**Severity:** Medium
**Location:** `pangolin_api/src/pangolin_handlers.rs` (list_commits)

*   **Issue:** `list_commits` fetches commits one-by-one by following `parent_id` pointers.
*   **Recommendation:** Implement `store.get_commit_ancestry(head_commit_id, limit)` using specific SQL features (e.g., Recursive CTES in Postgres) to fetch the entire history in a single query.

### 1.3 Asynchronous JSON Deserialization
**Severity:** Medium
**Location:** `pangolin_api/src/iceberg/tables.rs`

*   **Issue:** Large Iceberg metadata files (can be MBs) are deserialized using `serde_json::from_slice` directly in the async handler. This blocks the Tokio executor thread.
*   **Recommendation:** Offload deserialization of large metadata payloads to `tokio::task::spawn_blocking`.

## 2. Storage Layer Optimizations

### 2.1 Repetitive Object Store Initialization
**Severity:** High
**Location:** `pangolin_store/src/postgres/main.rs` (write_file)

*   **Issue:** When falling back to environment-based S3 config, `AmazonS3Builder` is instantiated and built on *every write*.
*   **Recommendation:** Cache the `ObjectStore` instance even for the default environment configuration, similar to how it is cached for warehouse-specific configs.

### 2.2 Redundant Metadata Reads in OCC Loops
**Severity:** Low/Medium
**Location:** `pangolin_api/src/iceberg/tables.rs` (update_table)

*   **Issue:** The `update_table` handler fetches the full asset twice: once for permission checks and once inside the optimistic concurrency control (OCC) loop.
*   **Recommendation:** Refactor to perform the permission check once, or optimize `get_asset` to return extensive metadata only when needed.

## 3. Database Schema & Indexing

### 3.1 Missing Indexes
**Severity:** Medium
**Location:** `pangolin_store/migrations`

*   **Issue:**
    *   `commits.parent_id`: Unindexed. Makes traversing history from parent-to-child (if ever needed) slow, and foreign key checks slower.
    *   `active_tokens.user_id`: Essential for `list_active_tokens` performance.
*   **Recommendation:** Add indexes:
    ```sql
    CREATE INDEX idx_commits_parent_id ON commits(parent_id);
    CREATE INDEX idx_active_tokens_user_id ON active_tokens(user_id);
    ```

### 3.2 Pagination Support
**Severity:** Medium
**Location:** All `list_*` endpoints

*   **Issue:** Endpoints like `list_commits`, `list_catalogs`, and `list_active_tokens` return bounded but potentially large result sets without pagination.
*   **Recommendation:** Add `limit` and `offset` (or cursor-based) pagination to all list methods in the `CatalogStore` trait.

## 4. Caching Strategy

### 4.1 Catalog-Warehouse Lookup
**Severity:** Low
**Location:** `pangolin_api/src/iceberg/tables.rs`

*   **Issue:** Credential vending requires joining Catalog -> Warehouse. This is currently done via two sequential DB lookups.
*   **Recommendation:** Cache the `CatalogID -> WarehouseID` mapping in memory, as this relationship changes infrequently.

## 5. Security & Stability

### 5.1 Connection Pool Tuning
**Severity:** Low
**Location:** `pangolin_store/src/postgres/main.rs`

*   **Issue:** Default pool settings (min: 2, max: 5) are very conservative.
*   **Recommendation:** Expose these as environment variables (already done, but defaults should be higher for production, e.g., max 20-50).

## 6. Implementation Roadmap

1.  **Phase 1 (Quick Wins):**
    *   Add missing DB indexes.
    *   Optimize `ObjectStore` caching in `PostgresStore`.
    *   Wrap JSON parsing in `spawn_blocking`.

2.  **Phase 2 (Architectural):**
    *   Extend `CatalogStore` trait with Bulk operations (`copy_assets_bulk`).
    *   Implement Recursive CTEs for commit history.

3.  **Phase 3 (Scalability):**
    *   Implement pagination across the API.
    *   Introduce extensive caching layers.

## 7. Backend Implementation Breakdown

Implementing Phase 2 (Bulk Operations & Recursive Queries) requires distinct approaches for each backend:

### 7.1 MemoryStore (Development)
*   **Bulk Copy:** Simple iteration and `clone()` of assets in the internal DashMap.
*   **Ancestry:** While-loop traversal of the internal commit HashMap.
*   **Effort:** Low

### 7.2 SQLiteStore (Embedded)
*   **Bulk Copy:** `INSERT INTO assets (..., id, ...) SELECT ..., new_uuid(), ... FROM assets WHERE ...`
*   **Ancestry:** Usage of `WITH RECURSIVE` CTEs (supported in modern SQLite).
*   **Effort:** Medium

### 7.3 PostgresStore (Production)
*   **Bulk Copy:** `INSERT INTO assets (..., id, ...) SELECT ..., gen_random_uuid(), ... FROM assets WHERE ...`
*   **Ancestry:** Usage of `WITH RECURSIVE` CTEs.
*   **Effort:** Medium

### 7.4 MongoStore (Production / Document)
*   **Bulk Copy:** Aggregation pipeline using `$match` (to find assets) -> `$replaceRoot` (modify IDs) -> `$merge` (write back to collection).
*   **Ancestry:** Aggregation pipeline using `$graphLookup` to traverse the `parent_id` chain efficiently.
*   **Effort:** High (Complex aggregation logic required)

## 8. Impact on Clients (CLI, UI, PyPangolin)

Most of the recommended optimizations are **internal to the API/Store** and will **not** require changes to the CLI, UI, or PyPangolin SDK. They optimize *throughput* and *latency* without changing the API contract.

### 8.1 Zero-Impact Changes (Internal Only)
These changes are transparent to clients:
*   **Bulk Copy Assets:** `create_branch` request/response remains identical.
*   **Commit History Optimization:** `list_commits` request/response remains identical.
*   **Async JSON / ObjectStore Caching / Indexes:** Purely backend performance tuning.

### 8.2 Low-Impact Changes (Backward Compatible)
*   **Pagination (Phase 3):**
    *   **API Change:** Adding `limit` and `offset` query parameters.
    *   **Client Impact:** Existing clients will continue to work (assuming the API returns a sensible default number of items).
    *   **Recommended Client Updates:** To fully utilize this, the CLI (e.g., `pangolin-admin list-catalogs --limit 50`), UI (infinite scroll or pages), and PyPangolin SDK should eventually be updated to support these parameters. However, the *initial* implementation can be done on the API side without forcing immediate client upgrades.

### 8.3 Conclusion
**No internal optimizations (Phase 1 & 2) require immediate client updates.** You can proceed with the API refactoring without waiting for CLI/UI/SDK work. Phase 3 (Pagination) should be coordinated with client teams when the time comes.

## 9. Implementation Progress Matrix

| Feature | MemoryStore | SqliteStore | PostgresStore | MongoStore | CLI/UI Impact | Status |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **P1: DB Indexes** | N/A | [ ] | [ ] | [ ] | None | Not Started |
| **P1: Async JSON** | [ ] | [ ] | [ ] | [ ] | None | Not Started |
| **P1: ObjStore Cache** | N/A | N/A | [ ] | N/A | None | Not Started |
| **P2: Bulk Copy** | [ ] | [ ] | [ ] | [ ] | None | Not Started |
| **P2: Recursive Ancestry**| [ ] | [ ] | [ ] | [ ] | None | Not Started |
| **P3: Pagination** | [ ] | [ ] | [ ] | [ ] | **Low** | Not Started |
