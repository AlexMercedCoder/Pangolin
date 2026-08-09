# Database schema

There is exactly one source of truth per backend. There used to be three
plausible ones for PostgreSQL, which is how a production drift incident starts
(A-27 in `AUDIT_EXECUTION_PLAN.md`).

| Backend | Source of truth | Applied by |
|---|---|---|
| PostgreSQL | `pangolin_store/migrations/*.sql` (timestamped `sqlx` chain) | `sqlx::migrate!` at `PostgresStore::new` |
| SQLite | `pangolin_store/sql/sqlite_schema.sql` | `SqliteStore::run_migrations` at startup |
| MongoDB | No schema; collections are created on first write | — |
| Memory | No schema | — |

Removed in the 0.6.0 hardening release, because nothing applied them and they
had already diverged:

* `migrations/` at the repository root (`postgres/` and `sqlite/`)
* `pangolin_store/migrations/sqlite/`, skipped by `sqlx::migrate!`, which only
  reads top-level `.sql` files
* `pangolin_store/sql/postgres_schema.sql`, superseded by the migration chain

Anything those trees defined and the surviving source did not — notably the
SQLite `revoked_tokens` table — has been folded into the surviving source.

## Adding a PostgreSQL migration

Create `migrations/<UTC timestamp>_<description>.sql`. `sqlx` records applied
migrations in `_sqlx_migrations` and refuses to run a chain whose checksums
have changed, so never edit a migration that has shipped.

## Adding a SQLite change

Edit `sql/sqlite_schema.sql` using `IF NOT EXISTS`, and bump
`SQLITE_SCHEMA_VERSION` in `src/sqlite/main.rs`. The applied version is
recorded in `_pangolin_schema_version`.
