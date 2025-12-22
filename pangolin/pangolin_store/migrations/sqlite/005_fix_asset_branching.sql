-- Migration: Fix Asset Branching
-- Adds branch_name to assets and ensures uniqueness per branch

-- 1. Add branch_name column
ALTER TABLE assets ADD COLUMN branch_name TEXT NOT NULL DEFAULT 'main';

-- 2. Create unique index to enforce isolation
CREATE UNIQUE INDEX idx_assets_isolation ON assets(tenant_id, catalog_name, branch_name, namespace_path, name);
