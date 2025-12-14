-- Pangolin SQLite Schema
-- This schema supports multi-tenant Iceberg catalog metadata storage

-- Tenants table
CREATE TABLE IF NOT EXISTS tenants (
    id TEXT PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    properties TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Warehouses table
CREATE TABLE IF NOT EXISTS warehouses (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    use_sts INTEGER NOT NULL DEFAULT 0,
    storage_config TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(tenant_id, name)
);

CREATE INDEX IF NOT EXISTS idx_warehouses_tenant ON warehouses(tenant_id);

-- Catalogs table
CREATE TABLE IF NOT EXISTS catalogs (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    catalog_type TEXT NOT NULL,
    warehouse_name TEXT,
    storage_location TEXT,
    federated_config TEXT,
    properties TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(tenant_id, name)
);

CREATE INDEX IF NOT EXISTS idx_catalogs_tenant ON catalogs(tenant_id);
CREATE INDEX IF NOT EXISTS idx_catalogs_warehouse ON catalogs(warehouse_name);

-- Namespaces table
CREATE TABLE IF NOT EXISTS namespaces (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    catalog_name TEXT NOT NULL,
    namespace_path TEXT NOT NULL,
    properties TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(tenant_id, catalog_name, namespace_path)
);

CREATE INDEX IF NOT EXISTS idx_namespaces_tenant_catalog ON namespaces(tenant_id, catalog_name);

-- Assets table (tables and views)
CREATE TABLE IF NOT EXISTS assets (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    catalog_name TEXT NOT NULL,
    branch TEXT,
    namespace_path TEXT NOT NULL,
    name TEXT NOT NULL,
    asset_type TEXT NOT NULL,
    metadata_location TEXT,
    properties TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_assets_tenant_catalog ON assets(tenant_id, catalog_name);
CREATE INDEX IF NOT EXISTS idx_assets_namespace ON assets(namespace_path);
CREATE INDEX IF NOT EXISTS idx_assets_branch ON assets(branch) WHERE branch IS NOT NULL;

-- Branches table
CREATE TABLE IF NOT EXISTS branches (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    catalog_name TEXT NOT NULL,
    name TEXT NOT NULL,
    commit_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(tenant_id, catalog_name, name)
);

CREATE INDEX IF NOT EXISTS idx_branches_tenant_catalog ON branches(tenant_id, catalog_name);

-- Tags table
CREATE TABLE IF NOT EXISTS tags (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    catalog_name TEXT NOT NULL,
    name TEXT NOT NULL,
    commit_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(tenant_id, catalog_name, name)
);

CREATE INDEX IF NOT EXISTS idx_tags_tenant_catalog ON tags(tenant_id, catalog_name);

-- Commits table
CREATE TABLE IF NOT EXISTS commits (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    message TEXT,
    author TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_commits_tenant ON commits(tenant_id);

-- Metadata locations table (for Iceberg table metadata files)
CREATE TABLE IF NOT EXISTS metadata_locations (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    catalog_name TEXT NOT NULL,
    branch TEXT,
    namespace_path TEXT NOT NULL,
    table_name TEXT NOT NULL,
    location TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(tenant_id, catalog_name, branch, namespace_path, table_name)
);

CREATE INDEX IF NOT EXISTS idx_metadata_locations_lookup ON metadata_locations(tenant_id, catalog_name, table_name);

-- Audit logs table
CREATE TABLE IF NOT EXISTS audit_logs (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    user_id TEXT,
    details TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_audit_logs_tenant ON audit_logs(tenant_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
