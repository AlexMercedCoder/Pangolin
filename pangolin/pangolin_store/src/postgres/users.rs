use super::PostgresStore;
use anyhow::Result;
use pangolin_core::user::{User, UserRole, OAuthProvider};
use uuid::Uuid;
use sqlx::Row;

impl PostgresStore {
    // Helper to Convert Row to User
    fn row_to_user(&self, row: sqlx::postgres::PgRow) -> Result<Option<User>> {
        let role_str: String = row.get("role");
        let role = match role_str.as_str() {
            "Root" => UserRole::Root,
            "TenantAdmin" => UserRole::TenantAdmin,
            _ => UserRole::TenantUser,
        };

        let oauth_provider_str: Option<String> = row.get("oauth_provider");
        let oauth_provider = match oauth_provider_str.as_deref() {
            Some("google") => Some(OAuthProvider::Google),
            Some("microsoft") => Some(OAuthProvider::Microsoft),
            Some("github") => Some(OAuthProvider::GitHub),
            Some("okta") => Some(OAuthProvider::Okta),
            _ => None,
        };

        Ok(Some(User {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            oauth_provider,
            oauth_subject: row.get("oauth_subject"),
            tenant_id: row.get("tenant_id"),
            role,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            last_login: row.get("last_login"),
            active: row.get("active"),
        }))
    }

    // User Operations
    pub async fn create_user(&self, user: User) -> Result<()> {
        sqlx::query("INSERT INTO users (id, username, email, password_hash, oauth_provider, oauth_subject, tenant_id, role, active, created_at, updated_at, last_login) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)")
            .bind(user.id)
            .bind(user.username)
            .bind(user.email)
            .bind(user.password_hash)
            .bind(user.oauth_provider.map(|p| format!("{:?}", p).to_lowercase()))
            .bind(user.oauth_subject)
            .bind(user.tenant_id)
            .bind(format!("{:?}", user.role))
            .bind(user.active)
            .bind(user.created_at)
            .bind(user.updated_at)
            .bind(user.last_login)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_user(&self, user_id: Uuid) -> Result<Option<User>> {
        let row: Option<sqlx::postgres::PgRow> = sqlx::query("SELECT id, username, email, password_hash, oauth_provider, oauth_subject, tenant_id, role, active, created_at, updated_at, last_login FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;
        
        if let Some(row) = row {
            self.row_to_user(row)
        } else {
            Ok(None)
        }
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let row: Option<sqlx::postgres::PgRow> = sqlx::query("SELECT id, username, email, password_hash, oauth_provider, oauth_subject, tenant_id, role, active, created_at, updated_at, last_login FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;
        
        if let Some(row) = row {
            self.row_to_user(row)
        } else {
            Ok(None)
        }
    }

    pub async fn list_users(&self, tenant_id: Option<Uuid>, pagination: Option<crate::PaginationParams>) -> Result<Vec<User>> {
        let limit = pagination.map(|p| p.limit.unwrap_or(i64::MAX as usize) as i64).unwrap_or(i64::MAX);
        let offset = pagination.map(|p| p.offset.unwrap_or(0) as i64).unwrap_or(0);

        let rows = if let Some(tid) = tenant_id {
            sqlx::query("SELECT id, username, email, password_hash, oauth_provider, oauth_subject, tenant_id, role, active, created_at, updated_at, last_login FROM users WHERE tenant_id = $1 LIMIT $2 OFFSET $3")
                .bind(tid)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query("SELECT id, username, email, password_hash, oauth_provider, oauth_subject, tenant_id, role, active, created_at, updated_at, last_login FROM users LIMIT $1 OFFSET $2")
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
        };

        let mut users = Vec::new();
        for row in rows {
            if let Ok(Some(user)) = self.row_to_user(row) {
                users.push(user);
            }
        }
        Ok(users)
    }

    pub async fn update_user(&self, user: User) -> Result<()> {
        sqlx::query("UPDATE users SET username = $1, email = $2, password_hash = $3, role = $4, active = $5, updated_at = $6, last_login = $7 WHERE id = $8")
            .bind(user.username)
            .bind(user.email)
            .bind(user.password_hash)
            .bind(format!("{:?}", user.role))
            .bind(user.active)
            .bind(chrono::Utc::now())
            .bind(user.last_login)
            .bind(user.id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_user(&self, user_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = $1").bind(user_id).execute(&self.pool).await?;
        Ok(())
    }
}
