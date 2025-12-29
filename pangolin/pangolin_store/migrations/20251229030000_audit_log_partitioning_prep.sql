-- Audit Log Partitioning (Preparation)
-- Goals: Enable efficient data retention (dropping old partitions)
-- Strategy: Range Partitioning on 'timestamp' (BIGINT, epoch ms)

BEGIN;

-- 1. Rename existing table if it's not already partitioned
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'audit_logs') THEN
        -- Check if it is already partitioned (relkind = 'p')
        IF NOT EXISTS (
            SELECT 1 FROM pg_class c 
            JOIN pg_namespace n ON n.oid = c.relnamespace 
            WHERE c.relname = 'audit_logs' AND n.nspname = current_schema() AND c.relkind = 'p'
        ) THEN
            ALTER TABLE audit_logs RENAME TO audit_logs_archive;
        END IF;
    END IF;
END
$$;

-- 2. Create the Partitioned Table
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    timestamp BIGINT NOT NULL,
    actor TEXT NOT NULL,
    action TEXT NOT NULL,
    resource TEXT NOT NULL,
    details JSONB,
    PRIMARY KEY (id, timestamp), -- Partition key must be part of PK
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
) PARTITION BY RANGE (timestamp);

-- 3. Create Default Partition (Catch-all)
CREATE TABLE IF NOT EXISTS audit_logs_default PARTITION OF audit_logs DEFAULT;

-- 4. Create Monthly Partitions (Example: Jan 2026, Feb 2026)
-- Timestamp is MS since epoch. 
-- 2026-01-01 = 1767225600000
-- 2026-02-01 = 1769904000000
CREATE TABLE IF NOT EXISTS audit_logs_2026_01 PARTITION OF audit_logs 
    FOR VALUES FROM (1767225600000) TO (1769904000000);

-- Data Migration (Optional - user to execute if desired)
-- INSERT INTO audit_logs SELECT * FROM audit_logs_archive;

COMMIT;
