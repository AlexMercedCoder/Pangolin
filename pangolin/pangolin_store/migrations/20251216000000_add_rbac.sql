-- Add missing IDs to match implementation
ALTER TABLE warehouses ADD COLUMN IF NOT EXISTS id UUID;
UPDATE warehouses SET id = gen_random_uuid() WHERE id IS NULL;
ALTER TABLE warehouses ALTER COLUMN id SET NOT NULL;

ALTER TABLE catalogs ADD COLUMN IF NOT EXISTS id UUID;
UPDATE catalogs SET id = gen_random_uuid() WHERE id IS NULL;
ALTER TABLE catalogs ALTER COLUMN id SET NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_catalogs_id ON catalogs(id);

ALTER TABLE namespaces ADD COLUMN IF NOT EXISTS id UUID;
UPDATE namespaces SET id = gen_random_uuid() WHERE id IS NULL;
ALTER TABLE namespaces ALTER COLUMN id SET NOT NULL;

-- Assets table already has id column and unique index in base schema

-- Users
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT,
    oauth_provider TEXT,
    oauth_subject TEXT,
    tenant_id UUID, -- Nullable for Root users
    role TEXT NOT NULL, -- 'Root', 'TenantAdmin', 'TenantUser'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login TIMESTAMPTZ,
    active BOOLEAN NOT NULL DEFAULT TRUE
);

-- Roles (RBAC)
CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    permissions JSONB NOT NULL, -- List of PermissionGrant
    created_by UUID NOT NULL, -- REFERENCES users(id)? No, usually users can be deleted but audit remains.
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- User Role Assignments
CREATE TABLE IF NOT EXISTS user_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    assigned_by UUID NOT NULL,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, role_id)
);

-- Direct Permissions
CREATE TABLE IF NOT EXISTS permissions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    scope JSONB NOT NULL, -- PermissionScope
    actions JSONB NOT NULL, -- HashSet<Action>
    granted_by UUID NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_users_tenant ON users(tenant_id);
CREATE INDEX IF NOT EXISTS idx_roles_tenant ON roles(tenant_id);

-- Access Requests (Discovery)
-- IMPORTANT: This must come AFTER the unique index on assets(id) is created
CREATE TABLE IF NOT EXISTS access_requests (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    reason TEXT,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status TEXT NOT NULL, -- 'Pending', 'Approved', 'Rejected'
    reviewed_by UUID, -- No FK to simplify deletes or cross-tenant scenarios (though internal)
    reviewed_at TIMESTAMPTZ,
    review_comment TEXT
);
CREATE INDEX IF NOT EXISTS idx_access_requests_user ON access_requests(user_id);

