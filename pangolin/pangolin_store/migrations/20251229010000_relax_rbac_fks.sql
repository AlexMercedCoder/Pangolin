-- Relax RBAC FK constraints to allow Service Users
-- This allows user_roles.user_id and permissions.user_id to reference IDs 
-- that exist in the 'service_users' table (which are not in 'users').

-- Drop FK on user_roles
ALTER TABLE user_roles DROP CONSTRAINT IF EXISTS user_roles_user_id_fkey;

-- Drop FK on permissions
ALTER TABLE permissions DROP CONSTRAINT IF EXISTS permissions_user_id_fkey;

-- Drop FK on active_tokens
ALTER TABLE active_tokens DROP CONSTRAINT IF EXISTS active_tokens_user_id_fkey;

-- Note: We rely on Application-Level validation to ensure the ID exists 
-- in either 'users' or 'service_users'.
