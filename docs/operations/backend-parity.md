# Backend feature parity

`CatalogStore` is a large trait in which many methods carry a default
implementation returning `Operation not supported by this store`. A missing
method therefore compiles cleanly and surfaces only as an opaque runtime `500`.
This page is the matrix that tells you, before you deploy, which features work
on which backend.

**Recommended for production: PostgreSQL.**

| Capability | Memory | SQLite | PostgreSQL | MongoDB |
|---|:--:|:--:|:--:|:--:|
| Tenants, catalogs, namespaces, assets | ✅ | ✅ | ✅ | ✅ |
| Warehouses and credential vending | ✅ | ✅ | ✅ | ✅ |
| Branches, tags, commits | ✅ | ✅ | ✅ | ✅ |
| Branch merge | ✅ | ✅ | ✅ | ✅ |
| Merge operations and conflicts | ✅ | ✅ | ✅ | ⚠️ |
| Users, roles, permissions (RBAC) | ✅ | ✅ | ✅ | ⚠️ |
| Service users and API keys | ✅ | ✅ | ✅ | ✅ |
| Token issue / revoke / list | ✅ | ✅ | ✅ | ✅ |
| Audit logging | ✅ | ✅ | ✅ | ⚠️ |
| Audit filtering and counting | ✅ | ✅ | ✅ | ⚠️ |
| Business metadata and access requests | ✅ | ✅ | ✅ | ✅ |
| Federated catalogs | ✅ | ✅ | ✅ | ✅ |
| Asset search | ✅ | ✅ | ✅ | ✅ |
| Bulk operations | ✅ | ✅ | ✅ | ✅ |
| Versioned schema migrations | n/a | ✅ | ✅ | ❌ |
| Multi-statement transactions | n/a | ⚠️ | ⚠️ | ❌ |
| Survives a restart | ❌ | ✅ | ✅ | ✅ |

✅ implemented and covered by tests · ⚠️ implemented with known gaps ·
❌ not available

## Notes

**Memory.** For development and tests only. Everything is lost on restart. The
server logs a warning at startup when it selects this backend, which happens
whenever `DATABASE_URL` is unset.

**SQLite.** Single-writer. Suitable for a single-node deployment or an
evaluation. The schema lives in `pangolin_store/sql/sqlite_schema.sql` and is
applied by `SqliteStore::run_migrations`, which records a version in
`_pangolin_schema_version`. Pool size honours `DATABASE_MAX_CONNECTIONS`.

**PostgreSQL.** The recommended backend. The schema comes from the timestamped
`sqlx` migration chain in `pangolin_store/migrations/`, applied at startup under
a `pg_advisory_lock` so that concurrent replicas do not race.

> Before 0.6.0 the chain **could not provision a fresh database**: `active_tokens`
> and `federated_sync_stats` were defined only in a schema file no runner
> applied, while a migration created an index on `active_tokens`. `audit_logs`
> also carried a shape the code could not write to, so every audit write failed.
> Both are fixed in 0.6.0; see `migrations/20260809000000_repair_orphaned_schema.sql`.

**MongoDB.** Functional for core catalog operations. Known gaps:

* No schema or index management at all — indexes must be created by hand.
* No transactions: `MongoStore` opens no sessions, so multi-statement
  operations are not atomic.
* RBAC aggregation and audit-log filtering have failing tests
  (`test_mongo_rbac_operations`, `test_mongo_list_user_permissions_aggregation`,
  `test_mongo_audit_log_filtering`, `test_mongo_store_regression`). Treat
  MongoDB as beta.

## Transactions

This is the most important row in the table and it is honest about being
incomplete.

Neither the PostgreSQL nor the MongoDB backend wraps multi-statement operations
in a transaction (A-24). Merging a branch, creating a branch by copying assets,
and cascading a catalog delete are issued as independent statements. A failure
or process death partway through leaves the catalog partially applied, with no
rollback and no repair tooling.

What exists today:

* SQLite uses a transaction in one place.
* PostgreSQL serialises schema setup with an advisory lock, so concurrent
  replicas cannot corrupt the schema.
* The Iceberg table-commit path uses compare-and-swap on the metadata pointer
  with retry, and — from 0.6.0 — enforces `assert-ref-snapshot-id`, so
  concurrent writers cannot fork snapshot lineage. This is the operation most
  likely to race, and it *is* safe.

What does not exist: a general transactional guarantee for the administrative
multi-statement paths. This is Phase 1.7 of `AUDIT_EXECUTION_PLAN.md` and is the
largest remaining item. Until it lands, take a database backup before a large
merge or a cascading delete.

## Choosing a backend

| If you… | Use |
|---|---|
| Are evaluating Pangolin | Memory (the default) or SQLite |
| Run one node and want durability | SQLite |
| Run in production | PostgreSQL |
| Have an existing MongoDB estate | MongoDB, having read the gaps above |
