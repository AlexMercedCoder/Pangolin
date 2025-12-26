use super::PostgresStore;
use anyhow::Result;
use uuid::Uuid;
use sqlx::Row;
use pangolin_core::user::ServiceUser;
use chrono::{DateTime, Utc};
use pangolin_core::user::UserRole;

impl PostgresStore {
    // Service User Operations
    pub async fn create_service_user(&self, service_user: ServiceUser) -> Result<()> {
        let role_str = match service_user.role {
            UserRole::Root => "Root",
            UserRole::TenantAdmin => "TenantAdmin",
            _ => "TenantUser",
        };

        sqlx::query(
            "INSERT INTO service_users (id, name, description, tenant_id, api_key_hash, role, created_at, created_by, last_used, expires_at, active) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
        )
        .bind(service_user.id)
        .bind(service_user.name)
        .bind(service_user.description)
        .bind(service_user.tenant_id)
        .bind(service_user.api_key_hash)
        .bind(role_str)
        .bind(service_user.created_at)
        .bind(service_user.created_by)
        .bind(service_user.last_used)
        .bind(service_user.expires_at)
        .bind(service_user.active)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_service_user(&self, id: Uuid) -> Result<Option<ServiceUser>> {
        let row = sqlx::query(
            "SELECT id, name, description, tenant_id, api_key_hash, role, created_at, created_by, last_used, expires_at, active 
             FROM service_users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            self.row_to_service_user(row)
        } else {
            Ok(None)
        }
    }

    pub async fn get_service_user_by_api_key_hash(&self, api_key_hash: &str) -> Result<Option<ServiceUser>> {
        let row = sqlx::query(
            "SELECT id, name, description, tenant_id, api_key_hash, role, created_at, created_by, last_used, expires_at, active 
             FROM service_users WHERE api_key_hash = $1 AND active = true"
        )
        .bind(api_key_hash)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            self.row_to_service_user(row)
        } else {
            Ok(None)
        }
    }

    pub async fn list_service_users(&self, tenant_id: Uuid) -> Result<Vec<ServiceUser>> {
        let rows = sqlx::query(
            "SELECT id, name, description, tenant_id, api_key_hash, role, created_at, created_by, last_used, expires_at, active 
             FROM service_users WHERE tenant_id = $1 ORDER BY created_at DESC"
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        let mut service_users = Vec::new();
        for row in rows {
            if let Some(su) = self.row_to_service_user(row)? {
                service_users.push(su);
            }
        }

        Ok(service_users)
    }

    pub async fn update_service_user(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        active: Option<bool>,
    ) -> Result<()> {
        let mut updates = Vec::new();
        let mut bind_idx = 2; // $1 is id

        if name.is_some() {
            updates.push(format!("name = ${}", bind_idx));
            bind_idx += 1;
        }
        if description.is_some() {
            updates.push(format!("description = ${}", bind_idx));
            bind_idx += 1;
        }
        if active.is_some() {
            updates.push(format!("active = ${}", bind_idx));
        }

        if updates.is_empty() {
            return Ok(());
        }

        let query_str = format!(
            "UPDATE service_users SET {} WHERE id = $1",
            updates.join(", ")
        );

        let mut query = sqlx::query(&query_str).bind(id);

        if let Some(n) = name {
            query = query.bind(n);
        }
        if let Some(d) = description {
            query = query.bind(d);
        }
        if let Some(a) = active {
            query = query.bind(a);
        }

        query.execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_service_user(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM service_users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_service_user_last_used(&self, id: Uuid, timestamp: DateTime<Utc>) -> Result<()> {
        sqlx::query("UPDATE service_users SET last_used = $1 WHERE id = $2")
            .bind(timestamp)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // This helper is used internally by the service user methods
    pub(crate) fn row_to_service_user(&self, row: sqlx::postgres::PgRow) -> Result<Option<ServiceUser>> {
        let role_str: String = row.get("role");
        let role = match role_str.as_str() {
            "Root" => UserRole::Root,
            "TenantAdmin" => UserRole::TenantAdmin,
            _ => UserRole::TenantUser,
        };

        Ok(Some(ServiceUser {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            tenant_id: row.get("tenant_id"),
            api_key_hash: row.get("api_key_hash"),
            role,
            created_at: row.get("created_at"),
            created_by: row.get("created_by"),
            last_used: row.get("last_used"),
            expires_at: row.get("expires_at"),
            active: row.get("active"),
        }))
    }
}
