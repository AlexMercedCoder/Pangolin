-- Tenants
CREATE TABLE tenants (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    properties JSONB NOT NULL
);

-- Warehouses
CREATE TABLE warehouses (
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name TEXT NOT NULL,
    storage_config JSONB NOT NULL,
    PRIMARY KEY (tenant_id, name)
);

-- Catalogs
CREATE TABLE catalogs (
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name TEXT NOT NULL,
    properties JSONB NOT NULL,
    PRIMARY KEY (tenant_id, name)
);

-- Namespaces
CREATE TABLE namespaces (
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    catalog_name TEXT NOT NULL,
    name TEXT NOT NULL,
    properties JSONB NOT NULL,
    PRIMARY KEY (tenant_id, catalog_name, name),
    FOREIGN KEY (tenant_id, catalog_name) REFERENCES catalogs(tenant_id, name)
);

-- Assets (Tables, Views, etc.)
CREATE TABLE assets (
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    catalog_name TEXT NOT NULL,
    namespace_name TEXT NOT NULL,
    name TEXT NOT NULL,
    kind TEXT NOT NULL, -- 'IcebergTable', 'View', etc.
    location TEXT NOT NULL,
    properties JSONB NOT NULL,
    active_branch TEXT, -- Optional: if asset is tied to a specific branch (though usually assets are branch-agnostic in definition, but their state is branch-dependent)
    -- Actually, assets are global in a namespace, but their *state* (metadata pointer) is tracked by branches/commits.
    -- However, for simple listing, we need to know they exist.
    PRIMARY KEY (tenant_id, catalog_name, namespace_name, name),
    FOREIGN KEY (tenant_id, catalog_name, namespace_name) REFERENCES namespaces(tenant_id, catalog_name, name)
);

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
