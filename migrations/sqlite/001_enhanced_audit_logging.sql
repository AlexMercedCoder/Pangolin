-- Migration: Enhanced Audit Logging Schema for SQLite
-- Date: 2025-12-18
-- Description: Updates audit_logs table to support enhanced audit logging with type-safe enums,
--              comprehensive user tracking, request context, and structured metadata

-- Step 1: Rename old table as backup
ALTER TABLE audit_logs RENAME TO audit_logs_old;

-- Step 2: Create new audit_logs table with enhanced schema
CREATE TABLE audit_logs (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    user_id TEXT,
    username TEXT NOT NULL,
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT,
    resource_name TEXT NOT NULL,
    timestamp INTEGER NOT NULL,  -- Unix timestamp in milliseconds
    ip_address TEXT,
    user_agent TEXT,
    result TEXT NOT NULL CHECK(result IN ('Success', 'Failure')),
    error_message TEXT,
    metadata TEXT,  -- JSON string
    
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);

-- Step 3: Create indexes for efficient querying
CREATE INDEX idx_audit_tenant_time ON audit_logs(tenant_id, timestamp DESC);
CREATE INDEX idx_audit_user ON audit_logs(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX idx_audit_action ON audit_logs(action);
CREATE INDEX idx_audit_resource_type ON audit_logs(resource_type);
CREATE INDEX idx_audit_resource ON audit_logs(resource_type, resource_id) WHERE resource_id IS NOT NULL;
CREATE INDEX idx_audit_result ON audit_logs(result);
CREATE INDEX idx_audit_timestamp ON audit_logs(timestamp DESC);

-- Step 4: Migrate data from old table (if it exists)
-- This attempts to map old fields to new fields with sensible defaults
INSERT INTO audit_logs (
    id,
    tenant_id,
    user_id,
    username,
    action,
    resource_type,
    resource_id,
    resource_name,
    timestamp,
    ip_address,
    user_agent,
    result,
    error_message,
    metadata
)
SELECT 
    id,
    tenant_id,
    NULL as user_id,  -- Old schema didn't have user_id
    COALESCE(actor, 'unknown') as username,
    COALESCE(action, 'Unknown') as action,
    'Unknown' as resource_type,  -- Old schema didn't have resource_type
    NULL as resource_id,  -- Old schema didn't have resource_id
    COALESCE(resource, 'unknown') as resource_name,
    timestamp,
    NULL as ip_address,  -- Old schema didn't have ip_address
    NULL as user_agent,  -- Old schema didn't have user_agent
    'Success' as result,  -- Assume success for old entries
    NULL as error_message,  -- Old schema didn't have error_message
    CASE 
        WHEN details IS NOT NULL THEN json_object('legacy_details', details)
        ELSE NULL
    END as metadata
FROM audit_logs_old
WHERE EXISTS (SELECT 1 FROM audit_logs_old LIMIT 1);

-- Step 5: Drop old table after verification (uncomment when ready)
-- DROP TABLE IF EXISTS audit_logs_old;

-- Verification queries:
-- SELECT COUNT(*) FROM audit_logs;
-- SELECT action, COUNT(*) FROM audit_logs GROUP BY action ORDER BY COUNT(*) DESC;
-- SELECT resource_type, COUNT(*) FROM audit_logs GROUP BY resource_type ORDER BY COUNT(*) DESC;
-- SELECT result, COUNT(*) FROM audit_logs GROUP BY result;
