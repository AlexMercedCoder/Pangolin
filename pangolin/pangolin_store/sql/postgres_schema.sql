-- Pangolin PostgreSQL Schema
-- This schema supports multi-tenant Iceberg catalog metadata storage

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Tenants table
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL,
    properties JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Warehouses table
CREATE TABLE IF NOT EXISTS warehouses (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    use_sts BOOLEAN NOT NULL DEFAULT FALSE,
    vending_strategy VARCHAR(50),
    storage_config JSONB NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, name)
);

CREATE INDEX IF NOT EXISTS idx_warehouses_tenant ON warehouses(tenant_id);

-- Catalogs table
CREATE TABLE IF NOT EXISTS catalogs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    catalog_type VARCHAR(50) NOT NULL,
    warehouse_name VARCHAR(255),
    storage_location TEXT,
    federated_config JSONB,
    properties JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, name)
);

CREATE INDEX IF NOT EXISTS idx_catalogs_tenant ON catalogs(tenant_id);
CREATE INDEX IF NOT EXISTS idx_catalogs_warehouse ON catalogs(warehouse_name);

-- Namespaces table
CREATE TABLE IF NOT EXISTS namespaces (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    catalog_name VARCHAR(255) NOT NULL,
    namespace_path TEXT[] NOT NULL,
    properties JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, catalog_name, namespace_path)
);

CREATE INDEX IF NOT EXISTS idx_namespaces_tenant_catalog ON namespaces(tenant_id, catalog_name);
CREATE INDEX IF NOT EXISTS idx_namespaces_path ON namespaces USING GIN(namespace_path);

-- Assets table (tables and views)
CREATE TABLE IF NOT EXISTS assets (
    id UUID NOT NULL DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    catalog_name VARCHAR(255) NOT NULL,
    branch VARCHAR(255),
    namespace_path TEXT[] NOT NULL,
    name VARCHAR(255) NOT NULL,
    asset_type VARCHAR(50) NOT NULL,
    metadata_location TEXT,
    properties JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tenant_id, catalog_name, branch, namespace_path, name)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_assets_id ON assets(id);
CREATE INDEX IF NOT EXISTS idx_assets_tenant_catalog ON assets(tenant_id, catalog_name);
CREATE INDEX IF NOT EXISTS idx_assets_namespace ON assets USING GIN(namespace_path);
CREATE INDEX IF NOT EXISTS idx_assets_branch ON assets(branch) WHERE branch IS NOT NULL;

-- Branches table
CREATE TABLE IF NOT EXISTS branches (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    catalog_name VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    commit_id UUID,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, catalog_name, name)
);

CREATE INDEX IF NOT EXISTS idx_branches_tenant_catalog ON branches(tenant_id, catalog_name);

-- Tags table
CREATE TABLE IF NOT EXISTS tags (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    catalog_name VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    commit_id UUID,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, catalog_name, name)
);

CREATE INDEX IF NOT EXISTS idx_tags_tenant_catalog ON tags(tenant_id, catalog_name);

-- Commits table
CREATE TABLE IF NOT EXISTS commits (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    message TEXT,
    author VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_commits_tenant ON commits(tenant_id);

-- Metadata locations table (for Iceberg table metadata files)
CREATE TABLE IF NOT EXISTS metadata_locations (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    catalog_name VARCHAR(255) NOT NULL,
    branch VARCHAR(255),
    namespace_path TEXT[] NOT NULL,
    table_name VARCHAR(255) NOT NULL,
    location TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, catalog_name, COALESCE(branch, ''), namespace_path, table_name)
);

CREATE INDEX IF NOT EXISTS idx_metadata_locations_lookup ON metadata_locations(tenant_id, catalog_name, table_name);

-- Active Tokens
CREATE TABLE IF NOT EXISTS active_tokens (
    token_id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token TEXT NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_active_tokens_user ON active_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_active_tokens_expiry ON active_tokens(expires_at);

-- System Settings
CREATE TABLE IF NOT EXISTS system_settings (
    tenant_id UUID PRIMARY KEY REFERENCES tenants(id) ON DELETE CASCADE,
    settings JSONB NOT NULL
);

-- Federated Catalog Stats
CREATE TABLE IF NOT EXISTS federated_sync_stats (
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    catalog_name VARCHAR(255) NOT NULL,
    stats JSONB NOT NULL,
    PRIMARY KEY (tenant_id, catalog_name)
);
