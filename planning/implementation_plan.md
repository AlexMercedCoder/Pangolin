# Implementation Plan - v0.4.0 Optimizations

## Goal
Implement "Phase 1" (Quick Wins) and "Phase 2" (Architectural) optimizations identified in the `api_optimization_recommendations.md` audit.
**Constraint:** Current API signatures are **LOCKED**. No changes to HTTP request/response formats.

## User Review Required
> [!IMPORTANT]
> The `CatalogStore` trait will be modified to add `copy_assets_bulk` and `get_commit_ancestry`. This requires updates to all four storage backends (Memory, Postgres, SQLite, Mongo).

## Proposed Changes

### 1. Database Schema (P1)
**Files:** `pangolin_store/migrations/*`
#### [NEW] `pangolin_store/migrations/20251227000000_add_perf_indexes.sql`
- Add index on `commits(parent_id)`.
- Add index on `active_tokens(user_id)`.

### 2. Store Trait Updates (P2)
**Files:** `pangolin_store/src/lib.rs`
#### [MODIFY] `CatalogStore` Trait
- Add `async fn copy_assets_bulk(&self, tenant_id: Uuid, catalog: &str, src_branch: &str, dest_branch: &str, namespace: Option<String>) -> Result<usize>`
- Add `async fn get_commit_ancestry(&self, tenant_id: Uuid, commit_id: Uuid, limit: usize) -> Result<Vec<Commit>>`

### 3. Backend Implementation (P1 & P2)

#### MemoryStore
**Files:** `pangolin_store/src/memory/mod.rs`
- Implement new trait methods using simple logic (existing `HashMap` iteration).

#### PostgresStore
**Files:** `pangolin_store/src/postgres/main.rs`
- **P1:** Implement caching in `write_file` using `OnceCell` or similar mechanism for the fallback S3 builder.
- **P2:** Implement `copy_assets_bulk` using single SQL `INSERT INTO ... SELECT`.
- **P2:** Implement `get_commit_ancestry` using `WITH RECURSIVE` CTE.

#### SQLiteStore
**Files:** `pangolin_store/src/sqlite/main.rs`
- **P2:** Implement `copy_assets_bulk` using SQL.
- **P2:** Implement `get_commit_ancestry` using `WITH RECURSIVE` CTE.

#### MongoStore
**Files:** `pangolin_store/src/mongo/main.rs`
- **P2:** Implement `copy_assets_bulk` using Aggregation framework.
- **P2:** Implement `get_commit_ancestry` using `$graphLookup`.

### 4. API Handler Updates (P1 & P2)

#### Branching Logic
**Files:** `pangolin_api/src/pangolin_handlers.rs`
- In `create_branch`: Remove the loop that calls `list_assets` -> `create_asset`. Replace with `store.copy_assets_bulk`.

#### Commit Listing
**Files:** `pangolin_api/src/pangolin_handlers.rs`
- In `list_commits`: Remove the `while` loop fetching parents. Replace with `store.get_commit_ancestry`.

#### JSON Offloading
**Files:** `pangolin_api/src/iceberg/tables.rs`
- Wrap `serde_json::from_slice` calls in `tokio::task::spawn_blocking` to prevent blocking the async runtime with large metadata files.

## Verification Plan

### Automated Tests
- Run existing test suite to ensure no regressions.
- Add unit tests for new store methods (`copy_assets_bulk`, `get_commit_ancestry`).

### Manual Verification
1.  **Branch Creation:** Verify `create_branch` works correctly and copies assets for all backends.
2.  **Commit History:** Verify `list_commits` returns correct ordering.
3.  **Performance:** (Optional) Benchmark `create_branch` with a large number of tables to verify O(1) behavior vs O(N).
