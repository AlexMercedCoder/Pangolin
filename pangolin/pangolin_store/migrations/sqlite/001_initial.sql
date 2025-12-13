-- Initial schema for SQLite backend
-- Tenants table
CREATE TABLE IF NOT EXISTS tenants (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    properties TEXT -- JSON
);

-- Warehouses table
CREATE TABLE IF NOT EXISTS warehouses (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    name TEXT NOT NULL,
    use_sts INTEGER NOT NULL, -- SQLite uses INTEGER for boolean
    storage_config TEXT NOT NULL, -- JSON
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_warehouses_tenant ON warehouses(tenant_id);

-- Catalogs table
CREATE TABLE IF NOT EXISTS catalogs (
    tenant_id TEXT NOT NULL,
    name TEXT NOT NULL,
    warehouse_name TEXT NOT NULL,
    storage_location TEXT NOT NULL,
    properties TEXT, -- JSON
    PRIMARY KEY (tenant_id, name),
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_catalogs_tenant ON catalogs(tenant_id);

-- Namespaces table
CREATE TABLE IF NOT EXISTS namespaces (
    tenant_id TEXT NOT NULL,
    catalog_name TEXT NOT NULL,
    name TEXT NOT NULL,
    properties TEXT, -- JSON
    PRIMARY KEY (tenant_id, catalog_name, name),
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_namespaces_tenant_catalog ON namespaces(tenant_id, catalog_name);

-- Assets table (tables, views, etc.)
CREATE TABLE IF NOT EXISTS assets (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    catalog_name TEXT NOT NULL,
    namespace_name TEXT NOT NULL,
    name TEXT NOT NULL,
    asset_type TEXT NOT NULL,
    metadata_location TEXT,
    branch_name TEXT,
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_assets_tenant_catalog_ns ON assets(tenant_id, catalog_name, namespace_name);
CREATE INDEX IF NOT EXISTS idx_assets_branch ON assets(branch_name);

-- Branches table
CREATE TABLE IF NOT EXISTS branches (
    tenant_id TEXT NOT NULL,
    catalog_name TEXT NOT NULL,
    asset_id TEXT NOT NULL,
    name TEXT NOT NULL,
    head_commit_id TEXT,
    PRIMARY KEY (tenant_id, catalog_name, asset_id, name),
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
    FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

-- Tags table
CREATE TABLE IF NOT EXISTS tags (
    tenant_id TEXT NOT NULL,
    catalog_name TEXT NOT NULL,
    asset_id TEXT NOT NULL,
    name TEXT NOT NULL,
    commit_id TEXT NOT NULL,
    PRIMARY KEY (tenant_id, catalog_name, asset_id, name),
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
    FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

-- Commits table
CREATE TABLE IF NOT EXISTS commits (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    catalog_name TEXT NOT NULL,
    asset_id TEXT NOT NULL,
    parent_commit_id TEXT,
    message TEXT,
    author TEXT,
    timestamp INTEGER NOT NULL, -- Unix timestamp
    metadata TEXT, -- JSON
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
    FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_commits_asset ON commits(asset_id);
CREATE INDEX IF NOT EXISTS idx_commits_parent ON commits(parent_commit_id);
