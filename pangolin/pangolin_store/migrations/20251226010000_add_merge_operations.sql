-- Add Merge Operations and Merge Conflicts tables for advanced branching features

-- Merge Operations table
CREATE TABLE IF NOT EXISTS merge_operations (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    catalog_name TEXT NOT NULL,
    source_branch TEXT NOT NULL,
    target_branch TEXT NOT NULL,
    base_commit_id UUID,
    status TEXT NOT NULL,
    conflicts UUID[] NOT NULL DEFAULT '{}',
    initiated_by UUID NOT NULL,
    initiated_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    result_commit_id UUID
);

CREATE INDEX IF NOT EXISTS idx_merge_operations_tenant_catalog ON merge_operations(tenant_id, catalog_name);
CREATE INDEX IF NOT EXISTS idx_merge_operations_status ON merge_operations(status);
CREATE INDEX IF NOT EXISTS idx_merge_operations_initiated_by ON merge_operations(initiated_by);

-- Merge Conflicts table
CREATE TABLE IF NOT EXISTS merge_conflicts (
    id UUID PRIMARY KEY,
    merge_operation_id UUID NOT NULL REFERENCES merge_operations(id) ON DELETE CASCADE,
    conflict_type TEXT NOT NULL,
    asset_id UUID,
    description TEXT NOT NULL,
    resolution JSONB,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_merge_conflicts_operation ON merge_conflicts(merge_operation_id);
CREATE INDEX IF NOT EXISTS idx_merge_conflicts_asset ON merge_conflicts(asset_id);
