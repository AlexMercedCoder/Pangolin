/// Token operations for SqliteStore
use super::SqliteStore;
use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;
use chrono::Utc;
use pangolin_core::token::TokenInfo;

impl SqliteStore {
    pub async fn store_token(&self, token_info: TokenInfo) -> Result<()> {
        sqlx::query("INSERT INTO active_tokens (token_id, user_id, token, expires_at, created_at) VALUES (?, ?, ?, ?, ?)")
            .bind(&token_info.id.to_string())
            .bind(token_info.user_id.to_string())
            .bind(token_info.token.unwrap_or_default())
            .bind(token_info.expires_at.timestamp())
            .bind(Utc::now().timestamp())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn validate_token(&self, token: &str) -> Result<Option<TokenInfo>> {
        // 1. Check if it's in active_tokens and not expired
        let row = sqlx::query("SELECT token_id, user_id, token, expires_at FROM active_tokens WHERE token = ? AND expires_at > ?")
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

            Ok(Some(TokenInfo {
                id: token_id,
                tenant_id: Uuid::default(),
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

    pub async fn revoke_token(&self, token_id: Uuid, expires_at: chrono::DateTime<chrono::Utc>, reason: Option<String>) -> Result<()> {
        let token_id_str = token_id.to_string();
        let expires_at_ms = expires_at.timestamp_millis();
        
        sqlx::query(
            "INSERT INTO revoked_tokens (token_id, expires_at, reason) VALUES (?, ?, ?)"
        )
        .bind(&token_id_str)
        .bind(expires_at_ms)
        .bind(reason)
        .execute(&self.pool)
        .await?;
        
        // Also remove from active_tokens to keep things clean
        sqlx::query("DELETE FROM active_tokens WHERE token_id = ?")
            .bind(&token_id_str)
            .execute(&self.pool)
            .await?;
            
        Ok(())
    }

    pub async fn is_token_revoked(&self, token_id: Uuid) -> Result<bool> {
        let token_id_str = token_id.to_string();
        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM revoked_tokens WHERE token_id = ?"
        )
        .bind(&token_id_str)
        .fetch_one(&self.pool)
        .await?;
        Ok(exists > 0)
    }

    pub async fn cleanup_expired_tokens(&self) -> Result<usize> {
        let now_ms = chrono::Utc::now().timestamp_millis();
        // Cleanup revoked tokens
        let rows_affected = sqlx::query(
            "DELETE FROM revoked_tokens WHERE expires_at < ?"
        )
        .bind(now_ms)
        .execute(&self.pool)
        .await?
        .rows_affected();
        
        // Cleanup active tokens
         let _ = sqlx::query(
            "DELETE FROM active_tokens WHERE expires_at < ?"
        )
        .bind(chrono::Utc::now().timestamp())
        .execute(&self.pool)
        .await?;

        Ok(rows_affected as usize)
    }

    pub async fn list_active_tokens(&self, user_id: Uuid) -> Result<Vec<TokenInfo>> {
        let rows = sqlx::query("SELECT token_id, user_id, token, expires_at FROM active_tokens WHERE user_id = ? AND expires_at > ?")
            .bind(user_id.to_string())
            .bind(Utc::now().timestamp())
            .fetch_all(&self.pool)
            .await?;

        let mut tokens = Vec::new();
        for row in rows {
             let token_id_str: String = row.get("token_id");
             
             // Check revocation
             let is_revoked: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM revoked_tokens WHERE token_id = ?")
                .bind(&token_id_str)
                .fetch_one(&self.pool)
                .await?;
                
             if is_revoked == 0 {
                tokens.push(TokenInfo {
                    id: Uuid::parse_str(&token_id_str)?,
                    tenant_id: Uuid::default(),
                    user_id: Uuid::parse_str(&row.get::<String, _>("user_id"))?,
                    username: "unknown".to_string(),
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
