/// User operations for SqliteStore
use super::SqliteStore;
use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;
use chrono::Utc;
use pangolin_core::user::User;

impl SqliteStore {
    pub async fn create_user(&self, user: User) -> Result<()> {
        sqlx::query("INSERT INTO users (id, username, email, password_hash, oauth_provider, oauth_subject, tenant_id, role, created_at, updated_at, last_login, active) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(user.id.to_string())
            .bind(user.username)
            .bind(user.email)
            .bind(user.password_hash)
            .bind(user.oauth_provider.map(|p| format!("{:?}", p).to_lowercase()))
            .bind(user.oauth_subject)
            .bind(user.tenant_id.map(|u| u.to_string()))
            .bind(format!("{:?}", user.role))
            .bind(user.created_at.timestamp_millis())
            .bind(user.updated_at.timestamp_millis())
            .bind(user.last_login.map(|t| t.timestamp_millis()))
            .bind(if user.active { 1 } else { 0 })
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_user(&self, user_id: Uuid) -> Result<Option<User>> {
        let row = sqlx::query("SELECT id, username, email, password_hash, oauth_provider, oauth_subject, tenant_id, role, created_at, updated_at, last_login, active FROM users WHERE id = ?")
            .bind(user_id.to_string())
            .fetch_optional(&self.pool)
            .await?;
            
        self.row_to_user(row)
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let row = sqlx::query("SELECT id, username, email, password_hash, oauth_provider, oauth_subject, tenant_id, role, created_at, updated_at, last_login, active FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;
            
        self.row_to_user(row)
    }

    pub async fn list_users(&self, tenant_id: Option<Uuid>, pagination: Option<crate::PaginationParams>) -> Result<Vec<User>> {
        let limit = pagination.map(|p| p.limit.unwrap_or(i64::MAX as usize) as i64).unwrap_or(-1);
        let offset = pagination.map(|p| p.offset.unwrap_or(0) as i64).unwrap_or(0);

        let rows = if let Some(tid) = tenant_id {
            sqlx::query("SELECT id, username, email, password_hash, oauth_provider, oauth_subject, tenant_id, role, created_at, updated_at, last_login, active FROM users WHERE tenant_id = ? LIMIT ? OFFSET ?")
                .bind(tid.to_string())
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query("SELECT id, username, email, password_hash, oauth_provider, oauth_subject, tenant_id, role, created_at, updated_at, last_login, active FROM users LIMIT ? OFFSET ?")
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
        };
        
        let mut users = Vec::new();
        for row in rows {
            if let Some(user) = self.row_to_user(Some(row))? {
                users.push(user);
            }
        }
        Ok(users)
    }

    pub async fn update_user(&self, user: User) -> Result<()> {
        sqlx::query("UPDATE users SET username = ?, email = ?, password_hash = ?, role = ?, active = ?, updated_at = ?, last_login = ? WHERE id = ?")
            .bind(user.username)
            .bind(user.email)
            .bind(user.password_hash)
            .bind(format!("{:?}", user.role))
            .bind(if user.active { 1 } else { 0 })
            .bind(Utc::now().timestamp_millis())
            .bind(user.last_login.map(|t| t.timestamp_millis()))
            .bind(user.id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_user(&self, user_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = ?").bind(user_id.to_string()).execute(&self.pool).await?;
        Ok(())
    }
}
