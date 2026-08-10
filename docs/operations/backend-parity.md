# Backend feature parity

`CatalogStore` is a large trait in which many methods carry a default
implementation returning `Operation not supported by this store`. A missing
method therefore compiles cleanly and surfaces only as an opaque runtime `500`.
This page is the matrix that tells you, before you deploy, which features work
on which backend.

**Recommended for production: PostgreSQL.**

Every ✅ below is now backed by the cross-backend parity suite
(`cargo test -p pangolin_store --test store_integration`) run against a live
instance of each backend, not by inspection. That distinction matters: until
0.7.0 the Postgres and MongoDB tests had never actually been executed, and this
table asserted several capabilities that failed at the first request — see
"What the first live run found" below.

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

**MongoDB.** Functional for core catalog operations. The RBAC and audit-log
failures listed here before 0.7.0 are fixed — all of them were one bug wearing
four hats, described below. Remaining gaps:

* No schema or index management beyond a handful of indexes created at startup.
* **A replica set is strongly recommended.** On a standalone `mongod`:
  * transactions are unavailable, so `delete_catalog` degrades to a sequential,
    non-atomic cascade. It now degrades with a warning; before 0.7.0 the
    degradation path was unreachable and the delete failed outright.
  * retryable writes are unavailable — add `retryWrites=false` to your
    connection string or single-document writes will be rejected.

  Both topologies are tested. CI runs the MongoDB suites twice: once against a
  standalone (the `guardrails` job) and once against a single-node replica set
  (`mongo-replica-set`), because the two exercise *different branches* of
  `delete_catalog` and testing one says nothing about the other. Locally:

  ```bash
  # standalone — the degraded paths
  docker compose -f docker-compose.db-test.yml up -d mongo
  export PANGOLIN_TEST_MONGO_URL='mongodb://testuser:testpass@localhost:27017/?retryWrites=false'

  # replica set — transactions and retryable writes
  docker compose -f docker-compose.mongo-rs.yml up -d
  export PANGOLIN_TEST_MONGO_URL='mongodb://localhost:27018/?replicaSet=rs0&directConnection=true'
  ```

  They use different ports so both can run at once.
* Multi-statement operations other than `delete_catalog` are still not atomic.

## What the first live run found

The parity suite was written against the memory and SQLite backends, which are
the two that CI could run without a service container. The first time it was
pointed at a live PostgreSQL and MongoDB it failed on both, for reasons no
amount of code reading had surfaced:

| Backend | Defect |
|---|---|
| PostgreSQL | **`business_metadata` was never created by any migration**, while `search_assets` joined it. Every asset search failed with `relation "business_metadata" does not exist` — a hard SQL error, not an empty result. The three CRUD methods were unimplemented, so the trait's "not supported" default answered them. |
| MongoDB | **`get_metadata_location` had no fallback to the asset's own `location`**, unlike the other three backends. A table created with a location but no explicit metadata-location property reported none, so its metadata could not be loaded and its commits compared against a different value than the read path returned. |
| MongoDB | **Role assignments were written by serde and queried as BSON Binary.** `bson::to_document` writes a `Uuid` as a string; the deserializer expects Binary. So `get_user_roles` never matched, every role-derived permission silently vanished, and a user holding an admin role was authorized as though they held none. The same asymmetry caused B1 (audit) and B2 (token revocation). |
| MongoDB | **`delete_catalog`'s "fall back when transactions are unavailable" path was unreachable.** `start_transaction` is a local call in the Rust driver, so it cannot fail for want of a replica set; the error arrives on the first operation *inside* the transaction and was propagated instead of caught. |
| MongoDB | Once that fallback *did* run, it carried B21 — the cascade deleted every matching child row before checking the catalog existed, then reported "not found" to a caller with every reason to believe nothing had happened. The same shape had been fixed for SQLite during the roadmap work; it survived here because this branch had never executed. |

Postgres, notably, is the only backend that makes the orphaned-child state
*unreachable*: it carries foreign keys from namespaces to catalogs and from
assets to namespaces, so a mis-ordered cascade has nothing to destroy. The other
three permit orphans, which is why the parity suite asserts the ordering there
and records the asymmetry rather than skipping quietly.

None of these were regressions. They had been present for as long as the code
had, and a per-backend test could not have found the first three: each is one
backend disagreeing with the others, which is only visible when something asserts
they agree.

## Transactions

This is the most important row in the table and it is honest about being
incomplete.

Neither the PostgreSQL nor the MongoDB backend wraps multi-statement operations
in a transaction (A-24). Merging a branch, creating a branch by copying assets,
and cascading a catalog delete are issued as independent statements. A failure
or process death partway through leaves the catalog partially applied, with no
rollback and no repair tooling.

What exists today:

* SQLite uses transactions for branch deletion and the catalog cascade.
* MongoDB uses a transaction for the catalog cascade where the deployment
  supports one, and degrades to a sequential cascade with a warning where it
  does not.
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
