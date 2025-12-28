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
    pub(crate) async fn list_active_tokens_internal(&self, tenant_id: Uuid, user_id: Option<Uuid>, pagination: Option<crate::PaginationParams>) -> Result<Vec<pangolin_core::token::TokenInfo>> {
            let iter = self.active_tokens.iter()
                .filter(|entry| {
                    let token = entry.value();
                    token.tenant_id == tenant_id && 
                    match user_id {
                        Some(uid) => token.user_id == uid,
                        None => true
                    } &&
                    !self.revoked_tokens.contains_key(&token.id) && 
                    token.expires_at > Utc::now()
                })
                .map(|entry| entry.value().clone());

            let tokens = if let Some(p) = pagination {
                iter.skip(p.offset.unwrap_or(0)).take(p.limit.unwrap_or(usize::MAX)).collect()
            } else {
                iter.collect()
            };
            Ok(tokens)
        }

    pub(crate) async fn revoke_token_internal(&self, token_id: Uuid, expires_at: chrono::DateTime<chrono::Utc>, reason: Option<String>) -> Result<()> {
            let revoked = pangolin_core::token::RevokedToken::new(token_id, expires_at, reason);
            self.revoked_tokens.insert(token_id, revoked);
            Ok(())
        }

    pub(crate) async fn is_token_revoked_internal(&self, token_id: Uuid) -> Result<bool> {
        Ok(self.revoked_tokens.contains_key(&token_id))
    }

    pub(crate) async fn cleanup_expired_tokens_internal(&self) -> Result<usize> {
        let now = Utc::now();
        let mut count = 0;
        
        // Cleanup expired active tokens
        let mut expired_active = Vec::new();
        for entry in self.active_tokens.iter() {
            if entry.value().expires_at < now {
                 expired_active.push(entry.key().clone());
            }
        }
        
        for id in expired_active {
            self.active_tokens.remove(&id);
            count += 1;
        }
        
        // Cleanup expired revoked tokens (blacklist)
        // If a token is expired, we don't need to keep it in blacklist anymore
        let mut expired_revoked = Vec::new();
        for entry in self.revoked_tokens.iter() {
            if entry.value().expires_at < now {
                expired_revoked.push(entry.key().clone());
            }
        }

        for id in expired_revoked {
            self.revoked_tokens.remove(&id);
            count += 1;
        }

        Ok(count)
    }
}
