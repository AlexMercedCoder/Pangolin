/// Token operations for SqliteStore
use super::SqliteStore;
use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;
use chrono::Utc;
use pangolin_core::token::TokenInfo;

impl SqliteStore {
    pub async fn store_token(&self, token_info: TokenInfo) -> Result<()> {
        sqlx::query("INSERT INTO active_tokens (token_id, user_id, tenant_id, token, expires_at, created_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(&token_info.id.to_string())
            .bind(token_info.user_id.to_string())
            .bind(token_info.tenant_id.to_string())
            .bind(token_info.token.unwrap_or_default())
            .bind(token_info.expires_at.timestamp())
            .bind(Utc::now().timestamp())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn validate_token(&self, token: &str) -> Result<Option<TokenInfo>> {
        // 1. Check if it's in active_tokens and not expired
        let row = sqlx::query("SELECT token_id, user_id, tenant_id, token, expires_at FROM active_tokens WHERE token = ? AND expires_at > ?")
            .bind(token)
            .bind(Utc::now().timestamp())
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
             let token_id_str: String = row.get("token_id");
             let token_id = Uuid::parse_str(&token_id_str)?;

             // 2. Check if it's in revoked_tokens
             let is_revoked: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM revoked_tokens WHERE token_id = ?")
                .bind(&token_id_str)
                .fetch_one(&self.pool)
                .await?;
            
             if is_revoked > 0 {
                 return Ok(None);
             }

            let tenant_id_str: String = row.get("tenant_id");
            let tenant_id = Uuid::parse_str(&tenant_id_str).unwrap_or_default();

            Ok(Some(TokenInfo {
                id: token_id,
                tenant_id, 
                user_id: Uuid::parse_str(&row.get::<String, _>("user_id"))?,
                username: "unknown".to_string(),
                token: Some(row.get("token")),
                expires_at: chrono::DateTime::from_timestamp(row.get("expires_at"), 0).unwrap_or_default(),
                created_at: Utc::now(),
                is_valid: true,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_active_tokens(&self, tenant_id: Uuid, user_id: Option<Uuid>, pagination: Option<crate::PaginationParams>) -> Result<Vec<TokenInfo>> {
        let limit = pagination.map(|p| p.limit.unwrap_or(i64::MAX as usize) as i64).unwrap_or(-1);
        let offset = pagination.map(|p| p.offset.unwrap_or(0) as i64).unwrap_or(0);

        let rows = if let Some(uid) = user_id {
            sqlx::query("SELECT token_id, user_id, tenant_id, token, expires_at FROM active_tokens WHERE tenant_id = ? AND user_id = ? AND expires_at > ? LIMIT ? OFFSET ?")
                .bind(tenant_id.to_string())
                .bind(uid.to_string())
                .bind(Utc::now().timestamp())
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query("SELECT token_id, user_id, tenant_id, token, expires_at FROM active_tokens WHERE tenant_id = ? AND expires_at > ? LIMIT ? OFFSET ?")
                .bind(tenant_id.to_string())
                .bind(Utc::now().timestamp())
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
        };

        let mut tokens = Vec::new();
        for row in rows {
             let token_id_str: String = row.get("token_id");
             
             // Check revocation
             let is_revoked: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM revoked_tokens WHERE token_id = ?")
                .bind(&token_id_str)
                .fetch_one(&self.pool)
                .await?;
                
             if is_revoked == 0 {
                let tenant_id_str: String = row.get("tenant_id");
                let t_id = Uuid::parse_str(&tenant_id_str).unwrap_or_default();
                
                tokens.push(TokenInfo {
                    id: Uuid::parse_str(&token_id_str)?,
                    tenant_id: t_id,
                    user_id: Uuid::parse_str(&row.get::<String, _>("user_id"))?,
                    username: "unknown".to_string(), // Inefficient to join username unless needed
                    token: Some(row.get("token")),
                    expires_at: chrono::DateTime::from_timestamp(row.get("expires_at"), 0).unwrap_or_default(),
                    created_at: Utc::now(),
                    is_valid: true,
                });
             }
        }
        Ok(tokens)
    }
}
