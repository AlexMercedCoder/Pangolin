use super::MongoStore;
use super::main::{to_bson_uuid, from_bson_uuid};
use anyhow::Result;
use mongodb::bson::{doc, Document};
use pangolin_core::token::TokenInfo;
use uuid::Uuid;
use futures::stream::TryStreamExt;

impl MongoStore {
    pub async fn list_active_tokens(&self, tenant_id: Uuid, user_id: Option<Uuid>, pagination: Option<crate::PaginationParams>) -> Result<Vec<TokenInfo>> {

        let mut filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id)
        };
        if let Some(uid) = user_id {
            filter.insert("user_id", to_bson_uuid(uid));
        }

        let collection = self.active_tokens();
        let mut find = collection.find(filter);
        if let Some(p) = pagination {
            if let Some(l) = p.limit {
                find = find.limit(l as i64);
            }
            if let Some(o) = p.offset {
                find = find.skip(o as u64);
            }
        }
        let cursor = find.await?;
        let docs: Vec<Document> = cursor.try_collect().await?;
        
        let mut tokens = Vec::new();
        for d in docs {
            tokens.push(TokenInfo {
                id: from_bson_uuid(d.get("token_id").ok_or(anyhow::anyhow!("Missing token_id"))?)?,
                tenant_id: from_bson_uuid(d.get("tenant_id").ok_or(anyhow::anyhow!("Missing tenant_id"))?)?,
                user_id: from_bson_uuid(d.get("user_id").ok_or(anyhow::anyhow!("Missing user_id"))?)?,
                username: d.get_str("username").unwrap_or("unknown").to_string(),
                token: d.get_str("token").ok().map(|s| s.to_string()),
                expires_at: mongodb::bson::from_bson(d.get("expires_at").unwrap().clone())?,
                created_at: mongodb::bson::from_bson(d.get("created_at").unwrap_or(&mongodb::bson::Bson::Null).clone()).unwrap_or_else(|_| chrono::Utc::now()),
                is_valid: true,
            });
        }
        Ok(tokens)
    }

    pub async fn store_token(&self, token_info: TokenInfo) -> Result<()> {
        let doc = doc! {
            "token_id": to_bson_uuid(token_info.id),
            "tenant_id": to_bson_uuid(token_info.tenant_id),
            "user_id": to_bson_uuid(token_info.user_id),
            "username": token_info.username,
            "token": token_info.token.unwrap_or_default(),
            "expires_at": token_info.expires_at,
            "created_at": token_info.created_at
        };
        self.active_tokens().insert_one(doc).await?;
        Ok(())
    }

    pub async fn revoke_token(&self, token_id: Uuid, expires_at: chrono::DateTime<chrono::Utc>, reason: Option<String>) -> Result<()> {
        let revoked = pangolin_core::token::RevokedToken::new(token_id, expires_at, reason);
        self.db.collection("revoked_tokens").insert_one(revoked).await?;
        Ok(())
    }

    pub async fn is_token_revoked(&self, token_id: Uuid) -> Result<bool> {
        let filter = doc! { "token_id": to_bson_uuid(token_id) };
        let result = self.db.collection::<pangolin_core::token::RevokedToken>("revoked_tokens")
            .find_one(filter)
            .await?;
        Ok(result.is_some())
    }

    pub async fn cleanup_expired_tokens(&self) -> Result<usize> {
        let now = chrono::Utc::now();
        let filter = doc! { "expires_at": { "$lt": now } };
        let result = self.db.collection::<pangolin_core::token::RevokedToken>("revoked_tokens")
            .delete_many(filter)
            .await?;
        Ok(result.deleted_count as usize)
    }

    pub async fn validate_token(&self, _token: &str) -> Result<Option<TokenInfo>> {
        // This usually involves JWT validation which is handled in the API layer,
        // but the store can provide a way to check if it's revoked or look up metadata.
        Ok(None)
    }
}
