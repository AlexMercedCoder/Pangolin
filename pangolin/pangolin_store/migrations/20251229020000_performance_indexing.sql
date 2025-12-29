-- Performance Indexing
-- Optimization for list_user_permissions which effectively scans permissions table
-- This is critical for authorization checks.

CREATE INDEX IF NOT EXISTS idx_permissions_user_id ON permissions(user_id);

-- Ensure active_tokens has user_id index (it should, but verify if exists)
-- CREATE INDEX IF NOT EXISTS idx_active_tokens_user ON active_tokens(user_id);
-- (Using IF NOT EXISTS is safe)
CREATE INDEX IF NOT EXISTS idx_active_tokens_user ON active_tokens(user_id);
