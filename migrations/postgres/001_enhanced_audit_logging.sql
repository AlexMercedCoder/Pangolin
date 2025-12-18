-- Migration: Enhanced Audit Logging Schema
-- Date: 2025-12-18
-- Description: Updates audit_logs table to support enhanced audit logging with type-safe enums,
--              comprehensive user tracking, request context, and structured metadata

-- Step 1: Rename old table as backup
ALTER TABLE IF EXISTS audit_logs RENAME TO audit_logs_old;

-- Step 2: Create new audit_logs table with enhanced schema
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    user_id UUID,
    username VARCHAR(255) NOT NULL,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id UUID,
    resource_name VARCHAR(500) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address VARCHAR(45),
    user_agent TEXT,
    result VARCHAR(20) NOT NULL,
    error_message TEXT,
    metadata JSONB,
    
    CONSTRAINT fk_audit_tenant FOREIGN KEY (tenant_id) 
        REFERENCES tenants(id) ON DELETE CASCADE,
    CONSTRAINT fk_audit_user FOREIGN KEY (user_id) 
        REFERENCES users(id) ON DELETE SET NULL,
    CONSTRAINT chk_result CHECK (result IN ('Success', 'Failure'))
);

-- Step 3: Create indexes for efficient querying
CREATE INDEX idx_audit_tenant_time ON audit_logs(tenant_id, timestamp DESC);
CREATE INDEX idx_audit_user ON audit_logs(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX idx_audit_action ON audit_logs(action);
CREATE INDEX idx_audit_resource_type ON audit_logs(resource_type);
CREATE INDEX idx_audit_resource ON audit_logs(resource_type, resource_id) WHERE resource_id IS NOT NULL;
CREATE INDEX idx_audit_result ON audit_logs(result);
CREATE INDEX idx_audit_timestamp ON audit_logs(timestamp DESC);
CREATE INDEX idx_audit_metadata ON audit_logs USING GIN (metadata) WHERE metadata IS NOT NULL;

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
        WHEN details IS NOT NULL THEN jsonb_build_object('legacy_details', details)
        ELSE NULL
    END as metadata
FROM audit_logs_old
WHERE EXISTS (SELECT 1 FROM audit_logs_old LIMIT 1);

-- Step 5: Add comments for documentation
COMMENT ON TABLE audit_logs IS 'Enhanced audit log entries tracking all system operations with comprehensive context';
COMMENT ON COLUMN audit_logs.id IS 'Unique identifier for the audit log entry';
COMMENT ON COLUMN audit_logs.tenant_id IS 'Tenant that owns this audit log entry';
COMMENT ON COLUMN audit_logs.user_id IS 'User who performed the action (NULL for system actions)';
COMMENT ON COLUMN audit_logs.username IS 'Username of the actor';
COMMENT ON COLUMN audit_logs.action IS 'Type of action performed (e.g., CreateTable, UpdateCatalog)';
COMMENT ON COLUMN audit_logs.resource_type IS 'Type of resource affected (e.g., Table, Catalog, User)';
COMMENT ON COLUMN audit_logs.resource_id IS 'Unique identifier of the affected resource';
COMMENT ON COLUMN audit_logs.resource_name IS 'Human-readable name of the affected resource';
COMMENT ON COLUMN audit_logs.timestamp IS 'When the action occurred';
COMMENT ON COLUMN audit_logs.ip_address IS 'IP address of the client that initiated the action';
COMMENT ON COLUMN audit_logs.user_agent IS 'User agent string of the client';
COMMENT ON COLUMN audit_logs.result IS 'Outcome of the action (Success or Failure)';
COMMENT ON COLUMN audit_logs.error_message IS 'Error message if the action failed';
COMMENT ON COLUMN audit_logs.metadata IS 'Additional structured metadata about the action (JSON)';

-- Step 6: Grant permissions (adjust as needed for your setup)
-- GRANT SELECT ON audit_logs TO audit_reader;
-- GRANT INSERT ON audit_logs TO audit_writer;

-- Step 7: Drop old table after verification (uncomment when ready)
-- DROP TABLE IF EXISTS audit_logs_old;

-- Verification queries:
-- SELECT COUNT(*) FROM audit_logs;
-- SELECT action, COUNT(*) FROM audit_logs GROUP BY action ORDER BY COUNT(*) DESC;
-- SELECT resource_type, COUNT(*) FROM audit_logs GROUP BY resource_type ORDER BY COUNT(*) DESC;
-- SELECT result, COUNT(*) FROM audit_logs GROUP BY result;
