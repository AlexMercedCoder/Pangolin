-- Migration: Fix Asset Branching
-- Rename active_branch to branch_name and include it in Primary Key

ALTER TABLE assets RENAME COLUMN active_branch TO branch_name;
ALTER TABLE assets ALTER COLUMN branch_name SET DEFAULT 'main';
UPDATE assets SET branch_name = 'main' WHERE branch_name IS NULL;
ALTER TABLE assets ALTER COLUMN branch_name SET NOT NULL;

-- Drop and recreate PK to include branch_name
ALTER TABLE assets DROP CONSTRAINT assets_pkey;
ALTER TABLE assets ADD PRIMARY KEY (tenant_id, catalog_name, branch_name, namespace_path, name);
