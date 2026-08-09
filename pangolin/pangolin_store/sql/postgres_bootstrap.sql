-- Tables the PostgreSQL migration chain assumes but never creates.
--
-- `active_tokens` and `federated_sync_stats` were defined only in
-- `sql/postgres_schema.sql`, which no runner ever applied — `sqlx::migrate!`
-- reads `migrations/` and nothing else (A-27). Migration
-- `20251227000000_add_perf_indexes.sql` then creates an index on
-- `active_tokens`, so **the migration chain aborted on any freshly-provisioned
-- database** and the server could not start. Only deployments where somebody
-- had also run `postgres_schema.sql` by hand ever worked.
--
-- This file is applied from `PostgresStore::new` immediately before the
-- migrator, rather than being added to the chain, for two reasons:
--
--   1. A new migration appended at the end runs *after* the migration that
--      already fails, so it cannot repair a fresh database.
--   2. Editing `20251227000000` would change its checksum and `sqlx` would
--      then refuse to start against every database that has already applied
--      it.
--
-- Every statement is `IF NOT EXISTS`, so this is a no-op on a database that
-- already has the tables.

CREATE TABLE IF NOT EXISTS active_tokens (
    token_id   UUID PRIMARY KEY,
    user_id    UUID NOT NULL,
    token      TEXT NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_active_tokens_user ON active_tokens (user_id);
CREATE INDEX IF NOT EXISTS idx_active_tokens_expiry ON active_tokens (expires_at);

CREATE TABLE IF NOT EXISTS federated_sync_stats (
    tenant_id    UUID NOT NULL,
    catalog_name VARCHAR(255) NOT NULL,
    stats        JSONB NOT NULL,
    PRIMARY KEY (tenant_id, catalog_name)
);
