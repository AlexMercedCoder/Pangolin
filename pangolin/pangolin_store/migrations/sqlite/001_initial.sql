-- SQLite Schema (Updated for Phase 1 Store Parity)
-- Tenants
CREATE TABLE IF NOT EXISTS tenants (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    properties TEXT -- JSON
);

-- Warehouses
CREATE TABLE IF NOT EXISTS warehouses (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    name TEXT NOT NULL,
    use_sts INTEGER NOT NULL,
    storage_config TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);

CREATE TABLE access_requests (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    asset_id TEXT NOT NULL,
    reason TEXT,
    requested_at TIMESTAMP NOT NULL,
    status TEXT NOT NULL, -- 'Pending', 'Approved', 'Rejected'
    reviewed_by TEXT,
    reviewed_at TIMESTAMP,
    review_comment TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
    -- Note: reviewed_by is nullable and references users(id) but standard FK might be tricky if user deleted. 
    -- Leaving strict FK off for reviewed_by or setting ON DELETE SET NULL is safer.
    -- For simplicity, no strict FK on reviewed_by for now or SET NULL
);

CREATE INDEX IF NOT EXISTS idx_warehouses_tenant ON warehouses(tenant_id);

-- Catalogs (Added ID)
CREATE TABLE IF NOT EXISTS catalogs (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    name TEXT NOT NULL,
    warehouse_name TEXT,
    storage_location TEXT,
    catalog_type TEXT NOT NULL,
    federated_config TEXT, -- JSON
    properties TEXT, -- JSON
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_catalogs_tenant ON catalogs(tenant_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_catalogs_name ON catalogs(tenant_id, name);

-- Namespaces (Added ID)
CREATE TABLE IF NOT EXISTS namespaces (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    catalog_name TEXT NOT NULL,
    namespace_path TEXT NOT NULL, -- JSON array of strings
    properties TEXT, -- JSON
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_namespaces_tenant_catalog ON namespaces(tenant_id, catalog_name);

-- Assets (Added ID)
CREATE TABLE IF NOT EXISTS assets (
    id TEXT,
    tenant_id TEXT NOT NULL,
    catalog_name TEXT NOT NULL,
    namespace_path TEXT NOT NULL, -- JSON array
    name TEXT NOT NULL,
    asset_type TEXT NOT NULL,
    metadata_location TEXT,
    properties TEXT, -- JSON
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_assets_tenant_catalog_ns ON assets(tenant_id, catalog_name, namespace_path);

-- Branches
CREATE TABLE IF NOT EXISTS branches (
    tenant_id TEXT NOT NULL,
    catalog_name TEXT NOT NULL,
    name TEXT NOT NULL,
    head_commit_id TEXT,
    branch_type TEXT NOT NULL,
    assets TEXT, -- JSON array of asset IDs/Names
    PRIMARY KEY (tenant_id, catalog_name, name),
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);

-- Tags
CREATE TABLE IF NOT EXISTS tags (
    tenant_id TEXT NOT NULL,
    catalog_name TEXT NOT NULL,
    name TEXT NOT NULL,
    commit_id TEXT NOT NULL,
    PRIMARY KEY (tenant_id, catalog_name, name),
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);

-- Commits
CREATE TABLE IF NOT EXISTS commits (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    parent_id TEXT,
    timestamp INTEGER NOT NULL,
    author TEXT NOT NULL,
    message TEXT NOT NULL,
    operations TEXT NOT NULL, -- JSON
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_commits_tenant ON commits(tenant_id);

-- Audit Logs
CREATE TABLE IF NOT EXISTS audit_logs (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    actor TEXT NOT NULL,
    action TEXT NOT NULL,
    resource TEXT NOT NULL,
    details TEXT, -- JSON
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_audit_logs_tenant_ts ON audit_logs(tenant_id, timestamp DESC);

-- RBAC: Users
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT,
    oauth_provider TEXT,
    oauth_subject TEXT,
    tenant_id TEXT,
    role TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    last_login INTEGER,
    active INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX IF NOT EXISTS idx_users_tenant ON users(tenant_id);

-- RBAC: Roles
CREATE TABLE IF NOT EXISTS roles (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    permissions TEXT NOT NULL, -- JSON
    created_by TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_roles_tenant ON roles(tenant_id);

-- RBAC: User Roles
CREATE TABLE IF NOT EXISTS user_roles (
    user_id TEXT NOT NULL,
    role_id TEXT NOT NULL,
    assigned_by TEXT NOT NULL,
    assigned_at INTEGER NOT NULL,
    PRIMARY KEY (user_id, role_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
);

-- RBAC: Permissions (Direct)
CREATE TABLE IF NOT EXISTS permissions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    scope TEXT NOT NULL, -- JSON
    actions TEXT NOT NULL, -- JSON
    granted_by TEXT NOT NULL,
    granted_at INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
