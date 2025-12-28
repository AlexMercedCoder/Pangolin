-- Add performance indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_commits_parent_id ON commits(parent_id);
CREATE INDEX IF NOT EXISTS idx_active_tokens_user_id ON active_tokens(user_id);
