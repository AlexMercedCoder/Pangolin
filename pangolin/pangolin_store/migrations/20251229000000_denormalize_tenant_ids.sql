-- Add tenant_id to active_tokens
ALTER TABLE active_tokens ADD COLUMN IF NOT EXISTS tenant_id UUID;

-- Backfill active_tokens from users
-- Note: This only works for tokens associated with users in the users table.
-- Tokens for Root users (who are not in users table) will have NULL tenant_id initially.
-- They might need to be cleared or manually updated if they are scoped to a tenant.
-- Standard Root tokens are often global (no tenant_id), but if they are listing tokens, they list based on context.
UPDATE active_tokens 
SET tenant_id = u.tenant_id 
FROM users u 
WHERE active_tokens.user_id = u.id;

-- Create Index
CREATE INDEX IF NOT EXISTS idx_active_tokens_tenant ON active_tokens(tenant_id);

-- Add tenant_id to permissions
ALTER TABLE permissions ADD COLUMN IF NOT EXISTS tenant_id UUID;

-- Backfill permissions from users?
-- Actually, permission logic usually implies the user belongs to the tenant.
-- But wait, a User can belong to Tenant A but be granted permissions in Tenant B?
-- No, currently Users are scoped to a single Tenant.
UPDATE permissions 
SET tenant_id = u.tenant_id 
FROM users u 
WHERE permissions.user_id = u.id;

-- Create Indexes
CREATE INDEX IF NOT EXISTS idx_permissions_tenant ON permissions(tenant_id);
CREATE INDEX IF NOT EXISTS idx_permissions_user ON permissions(user_id);
