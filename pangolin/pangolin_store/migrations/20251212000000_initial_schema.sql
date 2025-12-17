-- Tenants
CREATE TABLE tenants (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    properties JSONB NOT NULL
);

-- Warehouses
CREATE TABLE warehouses (
    id UUID NOT NULL,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name TEXT NOT NULL,
    storage_config JSONB NOT NULL,
    use_sts BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (tenant_id, name)
);

-- Catalogs
CREATE TABLE catalogs (
    id UUID NOT NULL,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name TEXT NOT NULL,
    catalog_type TEXT NOT NULL,
    warehouse_name TEXT,
    storage_location TEXT,
    federated_config JSONB,
    properties JSONB NOT NULL,
    PRIMARY KEY (tenant_id, name)
);

-- Namespaces
CREATE TABLE namespaces (
    id UUID NOT NULL,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    catalog_name TEXT NOT NULL,
    namespace_path TEXT[] NOT NULL,
    properties JSONB NOT NULL,
    PRIMARY KEY (tenant_id, catalog_name, namespace_path),
    FOREIGN KEY (tenant_id, catalog_name) REFERENCES catalogs(tenant_id, name)
);

-- Assets (Tables, Views, etc.)
CREATE TABLE assets (
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    catalog_name TEXT NOT NULL,
    namespace_path TEXT[] NOT NULL,
    name TEXT NOT NULL,
    asset_type TEXT NOT NULL, -- 'IcebergTable', 'View', etc.
    metadata_location TEXT,
    properties JSONB NOT NULL,
    active_branch TEXT, -- Optional: if asset is tied to a specific branch (though usually assets are branch-agnostic in definition, but their state is branch-dependent)
    -- Actually, assets are global in a namespace, but their *state* (metadata pointer) is tracked by branches/commits.
    -- However, for simple listing, we need to know they exist.
    PRIMARY KEY (tenant_id, catalog_name, namespace_path, name),
    FOREIGN KEY (tenant_id, catalog_name, namespace_path) REFERENCES namespaces(tenant_id, catalog_name, namespace_path)
);

CREATE UNIQUE INDEX idx_assets_id ON assets(id);

-- Branches
CREATE TABLE branches (
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    catalog_name TEXT NOT NULL,
    name TEXT NOT NULL,
    head_commit_id UUID,
    branch_type TEXT NOT NULL,
    assets TEXT[] NOT NULL, -- Array of asset identifiers this branch tracks
    PRIMARY KEY (tenant_id, catalog_name, name),
    FOREIGN KEY (tenant_id, catalog_name) REFERENCES catalogs(tenant_id, name)
);

-- Tags
CREATE TABLE tags (
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    catalog_name TEXT NOT NULL,
    name TEXT NOT NULL,
    commit_id UUID NOT NULL,
    PRIMARY KEY (tenant_id, catalog_name, name),
    FOREIGN KEY (tenant_id, catalog_name) REFERENCES catalogs(tenant_id, name)
);

-- Commits
CREATE TABLE commits (
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    id UUID PRIMARY KEY,
    parent_id UUID,
    timestamp TIMESTAMPTZ NOT NULL,
    author TEXT NOT NULL,
    message TEXT NOT NULL,
    operations JSONB NOT NULL -- List of operations
);

-- Audit Logs
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    timestamp TIMESTAMPTZ NOT NULL,
    actor TEXT NOT NULL,
    action TEXT NOT NULL,
    resource TEXT NOT NULL,
    details TEXT
);

-- Indexes
CREATE INDEX idx_commits_tenant ON commits(tenant_id);
CREATE INDEX idx_audit_logs_tenant_timestamp ON audit_logs(tenant_id, timestamp DESC);
