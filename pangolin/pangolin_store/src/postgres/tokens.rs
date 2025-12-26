use super::PostgresStore;
use anyhow::Result;
use uuid::Uuid;

impl PostgresStore {
    // Token Revocation Operations
    pub async fn revoke_token(&self, token_id: Uuid, expires_at: chrono::DateTime<chrono::Utc>, reason: Option<String>) -> Result<()> {
        sqlx::query(
            "INSERT INTO revoked_tokens (token_id, expires_at, reason) VALUES ($1, $2, $3)"
        )
        .bind(token_id)
        .bind(expires_at)
        .bind(reason)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn is_token_revoked(&self, token_id: Uuid) -> Result<bool> {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM revoked_tokens WHERE token_id = $1)"
        )
        .bind(token_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(exists)
    }

    pub async fn cleanup_expired_tokens(&self) -> Result<usize> {
        let rows_affected = sqlx::query(
            "DELETE FROM revoked_tokens WHERE expires_at < NOW()"
        )
        .execute(&self.pool)
        .await?
        .rows_affected();
        Ok(rows_affected as usize)
    }
}
