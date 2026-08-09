-- Fold the orphaned migration trees back into the one chain that actually runs,
-- and reconcile `audit_logs` with the code that writes to it.
--
-- Two schema changes lived only in `migrations/postgres/` at the repository
-- root, which no runner ever referenced (A-27 in AUDIT_EXECUTION_PLAN.md). The
-- Rust code was written against them, so on any database provisioned from this
-- chain every access-request query failed with `column "tenant_id" of relation
-- "access_requests" does not exist`, and every audit write failed with `column
-- "user_id" of relation "audit_logs" does not exist`.
--
-- `audit_logs` needs more than added columns. `20251229030000` re-created it as
-- a table partitioned by `timestamp BIGINT` carrying the original
-- actor/resource/details shape, while `pangolin_store/src/postgres/audit.rs`
-- inserts the enhanced shape with a `TIMESTAMPTZ` timestamp. A partition key's
-- type cannot be altered in place, so the table is rebuilt here. No audit row
-- provisioned by this chain can ever have been written — every insert failed —
-- so nothing is lost; rows from a hand-provisioned `audit_logs_archive` are
-- carried over where they exist.

-- --------------------------------------------------------------------------
-- audit_logs: enhanced schema, partitioned on a real timestamp
-- --------------------------------------------------------------------------

DROP TABLE IF EXISTS audit_logs CASCADE;

CREATE TABLE audit_logs (
    id            UUID NOT NULL,
    tenant_id     UUID NOT NULL,
    user_id       UUID,
    username      TEXT NOT NULL,
    -- TEXT rather than VARCHAR(n): AuditAction, ResourceType and AuditResult
    -- all derive `sqlx::Type` with `type_name = "text"`, and sqlx rejects a
    -- VARCHAR column as a type mismatch when decoding them.
    action        TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id   UUID,
    resource_name TEXT NOT NULL,
    timestamp     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address    TEXT,
    user_agent    TEXT,
    result        TEXT NOT NULL,
    error_message TEXT,
    metadata      JSONB,
    -- The partition key has to be part of the primary key.
    PRIMARY KEY (id, timestamp),
    -- Deliberately no foreign key on tenant_id. The previous definition used
    -- ON DELETE CASCADE, which erases the audit trail of a tenant at exactly
    -- the moment it becomes most interesting, and a referential constraint can
    -- reject an audit write outright — an audit record must never be lost
    -- because of one (C-18/C-20). `20251229010000_relax_rbac_fks.sql` relaxed
    -- the RBAC foreign keys for the same reason.
    -- `AuditResult` derives `sqlx::Type` with `rename_all = "snake_case"`, so
    -- the stored values are lowercase. The orphaned root migration constrained
    -- this to 'Success'/'Failure', which no insert would ever have satisfied.
    CONSTRAINT chk_audit_result CHECK (result IN ('success', 'failure'))
) PARTITION BY RANGE (timestamp);

-- Catch-all partition, so a write never fails for want of a range. Retention is
-- implemented by attaching dated partitions and dropping the old ones; see
-- docs/operations/runbook.md.
CREATE TABLE IF NOT EXISTS audit_logs_default PARTITION OF audit_logs DEFAULT;

CREATE INDEX IF NOT EXISTS idx_audit_tenant_time
    ON audit_logs (tenant_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_user ON audit_logs (user_id)
    WHERE user_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_logs (action);
CREATE INDEX IF NOT EXISTS idx_audit_resource_type ON audit_logs (resource_type);
CREATE INDEX IF NOT EXISTS idx_audit_resource
    ON audit_logs (resource_type, resource_id) WHERE resource_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_result ON audit_logs (result);
CREATE INDEX IF NOT EXISTS idx_audit_metadata ON audit_logs USING GIN (metadata)
    WHERE metadata IS NOT NULL;

-- Carry over rows from a pre-partitioning table if one is present and has the
-- original column set. The shape of `audit_logs_archive` depends on which
-- historical path a database took, so a failure to copy must not block the
-- migration: the schema repair matters, the historical rows do not.
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = current_schema()
          AND table_name = 'audit_logs_archive'
          AND column_name = 'actor'
    ) THEN
        BEGIN
            INSERT INTO audit_logs (
                id, tenant_id, user_id, username, action, resource_type,
                resource_id, resource_name, timestamp, result, metadata
            )
            SELECT
                a.id,
                a.tenant_id,
                NULL,
                a.actor,
                a.action,
                'metadata',
                NULL,
                a.resource,
                to_timestamp(a.timestamp / 1000.0),
                'success',
                a.details
            FROM audit_logs_archive a
            ON CONFLICT DO NOTHING;
        EXCEPTION WHEN OTHERS THEN
            RAISE NOTICE 'skipping audit_logs_archive carry-over: %', SQLERRM;
        END;
    END IF;
END
$$;

-- --------------------------------------------------------------------------
-- access_requests: tenant scoping
-- --------------------------------------------------------------------------

ALTER TABLE access_requests ADD COLUMN IF NOT EXISTS tenant_id UUID;

-- Backfill from the requesting user, which is where the tenant came from
-- before the column existed.
UPDATE access_requests ar
SET tenant_id = u.tenant_id
FROM users u
WHERE ar.user_id = u.id AND ar.tenant_id IS NULL;

CREATE INDEX IF NOT EXISTS idx_access_requests_tenant
    ON access_requests (tenant_id);
