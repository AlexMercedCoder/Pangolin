use super::MemoryStore;
use anyhow::Result;
use uuid::Uuid;
use pangolin_core::token::*;
use async_trait::async_trait;
use chrono::Utc;

impl MemoryStore {
    pub(crate) async fn store_token_internal(&self, token: pangolin_core::token::TokenInfo) -> Result<()> {
            self.active_tokens.insert(token.id, token);
            Ok(())
        }
    pub(crate) async fn list_active_tokens_internal(&self, tenant_id: Uuid, user_id: Uuid) -> Result<Vec<pangolin_core::token::TokenInfo>> {
            let mut tokens = Vec::new();
            // Return tokens that match user and are not revoked/expired
            for entry in self.active_tokens.iter() {
                let token = entry.value();
                if token.tenant_id == tenant_id && token.user_id == user_id {
                    // Check revocation
                    if !self.revoked_tokens.contains_key(&token.id) && token.expires_at > Utc::now() {
                        tokens.push(token.clone());
                    }
                }
            }
            Ok(tokens)
        }

    pub(crate) async fn revoke_token_internal(&self, token_id: Uuid, expires_at: chrono::DateTime<chrono::Utc>, reason: Option<String>) -> Result<()> {
            let revoked = pangolin_core::token::RevokedToken::new(token_id, expires_at, reason);
            self.revoked_tokens.insert(token_id, revoked);
            Ok(())
        }
}
