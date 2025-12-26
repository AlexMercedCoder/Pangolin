-- Add Service Users table for machine-to-machine authentication
CREATE TABLE IF NOT EXISTS service_users (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    api_key_hash TEXT NOT NULL,
    role TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    created_by UUID NOT NULL,
    last_used TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    UNIQUE(tenant_id, name)
);

CREATE INDEX idx_service_users_tenant ON service_users(tenant_id);
CREATE INDEX idx_service_users_api_key_hash ON service_users(api_key_hash);

-- Add System Settings table for tenant-level configuration
CREATE TABLE IF NOT EXISTS system_settings (
    tenant_id UUID PRIMARY KEY REFERENCES tenants(id),
    settings JSONB NOT NULL DEFAULT '{}'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Enhance Audit Logs table with additional fields
-- First, check if columns exist before adding them
DO $$
BEGIN
    -- Add user_id column if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name='audit_logs' AND column_name='user_id') THEN
        ALTER TABLE audit_logs ADD COLUMN user_id UUID;
    END IF;

    -- Add resource_type column if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name='audit_logs' AND column_name='resource_type') THEN
        ALTER TABLE audit_logs ADD COLUMN resource_type TEXT;
    END IF;

    -- Add resource_id column if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name='audit_logs' AND column_name='resource_id') THEN
        ALTER TABLE audit_logs ADD COLUMN resource_id UUID;
    END IF;

    -- Add ip_address column if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name='audit_logs' AND column_name='ip_address') THEN
        ALTER TABLE audit_logs ADD COLUMN ip_address TEXT;
    END IF;

    -- Add user_agent column if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name='audit_logs' AND column_name='user_agent') THEN
        ALTER TABLE audit_logs ADD COLUMN user_agent TEXT;
    END IF;

    -- Add result column if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name='audit_logs' AND column_name='result') THEN
        ALTER TABLE audit_logs ADD COLUMN result TEXT;
    END IF;

    -- Add error_message column if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name='audit_logs' AND column_name='error_message') THEN
        ALTER TABLE audit_logs ADD COLUMN error_message TEXT;
    END IF;

    -- Add metadata column if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name='audit_logs' AND column_name='metadata') THEN
        ALTER TABLE audit_logs ADD COLUMN metadata JSONB;
    END IF;

    -- Rename actor to username if actor exists and username doesn't
    IF EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name='audit_logs' AND column_name='actor')
       AND NOT EXISTS (SELECT 1 FROM information_schema.columns 
                       WHERE table_name='audit_logs' AND column_name='username') THEN
        ALTER TABLE audit_logs RENAME COLUMN actor TO username;
    END IF;

    -- Rename resource to resource_name if resource exists and resource_name doesn't
    IF EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name='audit_logs' AND column_name='resource')
       AND NOT EXISTS (SELECT 1 FROM information_schema.columns 
                       WHERE table_name='audit_logs' AND column_name='resource_name') THEN
        ALTER TABLE audit_logs RENAME COLUMN resource TO resource_name;
    END IF;
END $$;

-- Add indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource_type ON audit_logs(resource_type);
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource_id ON audit_logs(resource_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_result ON audit_logs(result);
