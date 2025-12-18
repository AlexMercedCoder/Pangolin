-- Add vending_strategy column to warehouses table
ALTER TABLE warehouses 
ADD COLUMN vending_strategy JSONB;

-- Set default for existing warehouses (AWS Static if they have credentials)
-- Note: We assume access_key/secret_key were stored in storage_config?
-- Actually, the Warehouse model has `storage_config` map, and `use_sts` bool.
-- In Postgres, `storage_config` is likely a JSONB column.
-- `use_sts` is a boolean column.
-- BUT, typically credentials aren't stored in columns named `access_key` / `secret_key` in the table itself 
-- unless the schema defined them that way.
-- Let's check `postgres.rs` or previous migrations to see `warehouses` schema.

-- Actually, let's create the migration file with just ADD COLUMN first.
-- Data migration logic depends on schema knowledge.
-- If I can't verify existing schema easily via `psql`, I'll check `postgres.rs`.

-- Assuming `warehouses` has `storage_config` (JSONB) and `use_sts` (BOOL).
-- I'll skip complex data migration SQL for now and handle "backward compatibility" in application code (as implemented in MemoryStore).
-- Just adding the column is required.

-- Migration content:
ALTER TABLE warehouses ADD COLUMN vending_strategy JSONB;
