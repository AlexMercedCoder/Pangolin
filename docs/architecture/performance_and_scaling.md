# Performance & Scaling Architecture

Pangolin is designed for high-throughput, low-latency metadata operations. This document details the key architectural patterns used to achieve performance at scale.

## 1. Database Indexing (P1 Optimization)
To support efficient querying for high-cardinality entities, strategic indexes are applied to the backend databases (specifically Postgres).

### Key Indexes (`20251227000000_add_perf_indexes.sql`)
1.  **`commits.parent_id`**: Optimizes `get_commit_ancestry` recursion. Crucial for traversing long commit histories (e.g., Iceberg snapshots).
2.  **`active_tokens.user_id`**: Optimizes `list_active_tokens` and token validation lookups, essential for high-concurrency API authentication.

## 2. Pagination Strategy
All list operations in the `CatalogStore` trait enforce pagination to prevent memory exhaustion and ensure predictable response times.

### `PaginationParams` Struct
- **Limit**: Maximum number of items to return (default 100, max 1000).
- **Offset**: Number of items to skip.

### Implementation
- **API Layer**: Extracts `page` and `page_size` query params, converts to `PaginationParams`.
- **Store Layer**: Applies `LIMIT` / `OFFSET` (SQL) or cursor-based skipping (Mongo).
- **Client SDKs**: Paginators are provided to abstract this logic for users.

## 3. Asynchronous I/O
The entire backend stack is fully asynchronous, built on `tokio` and `axum`.

- **Non-blocking DB Calls**: Uses `sqlx` (Postgres/SQLite) and `mongodb::Client` in async mode.
- **Concurrent Object Store Access**: `object_store` crate usage allows parallel fetching of metadata files.
- **Background Tasks**: Tasks like `cleanup_expired_tokens` run in detached Tokio tasks to avoid blocking request paths.

## 4. Caching Layers
Performance is further augmented by a multi-tier caching strategy explained in [Caching Architecture](./caching.md).
- **Level 1**: Wrapper Cache (`CachedCatalogStore`) - Configuration data.
- **Level 2**: Metadata Cache (`moka`) - Iceberg Manifest bytes.
- **Level 3**: Connection Pools (`ObjectStoreCache`, DB Pools).
