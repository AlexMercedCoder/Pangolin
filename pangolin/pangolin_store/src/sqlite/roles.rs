/// Role operations for SqliteStore
use super::SqliteStore;
use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;
use chrono::Utc;
use pangolin_core::permission::{Role, UserRole as UserRoleAssignment};

impl SqliteStore {
    pub async fn create_role(&self, role: Role) -> Result<()> {
        sqlx::query("INSERT INTO roles (id, tenant_id, name, description, permissions, created_by, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(role.id.to_string())
            .bind(role.tenant_id.to_string())
            .bind(role.name)
            .bind(role.description)
            .bind(serde_json::to_string(&role.permissions)?)
            .bind(role.created_by.to_string())
            .bind(role.created_at.timestamp_millis())
            .bind(role.updated_at.timestamp_millis())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_role(&self, role_id: Uuid) -> Result<Option<Role>> {
        let row = sqlx::query("SELECT id, tenant_id, name, description, permissions, created_by, created_at, updated_at FROM roles WHERE id = ?")
            .bind(role_id.to_string())
            .fetch_optional(&self.pool)
            .await?;
        
        if let Some(row) = row {
            Ok(Some(Role {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                tenant_id: Uuid::parse_str(&row.get::<String, _>("tenant_id"))?,
                name: row.get("name"),
                description: row.get("description"),
                permissions: serde_json::from_str(&row.get::<String, _>("permissions"))?,
                created_by: Uuid::parse_str(&row.get::<String, _>("created_by"))?,
                created_at: chrono::DateTime::from_timestamp_millis(row.get("created_at")).unwrap_or_default(),
                updated_at: chrono::DateTime::from_timestamp_millis(row.get("updated_at")).unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_roles(&self, tenant_id: Uuid, pagination: Option<crate::PaginationParams>) -> Result<Vec<Role>> {
        let limit = pagination.map(|p| p.limit.unwrap_or(i64::MAX as usize) as i64).unwrap_or(-1);
        let offset = pagination.map(|p| p.offset.unwrap_or(0) as i64).unwrap_or(0);

        let rows = sqlx::query("SELECT id, tenant_id, name, description, permissions, created_by, created_at, updated_at FROM roles WHERE tenant_id = ? LIMIT ? OFFSET ?")
            .bind(tenant_id.to_string())
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;
            
        let mut roles = Vec::new();
        for row in rows {
            roles.push(Role {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                tenant_id: Uuid::parse_str(&row.get::<String, _>("tenant_id"))?,
                name: row.get("name"),
                description: row.get("description"),
                permissions: serde_json::from_str(&row.get::<String, _>("permissions"))?,
                created_by: Uuid::parse_str(&row.get::<String, _>("created_by"))?,
                created_at: chrono::DateTime::from_timestamp_millis(row.get("created_at")).unwrap_or_default(),
                updated_at: chrono::DateTime::from_timestamp_millis(row.get("updated_at")).unwrap_or_default(),
            });
        }
        Ok(roles)
    }

    pub async fn delete_role(&self, role_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM roles WHERE id = ?").bind(role_id.to_string()).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn update_role(&self, role: Role) -> Result<()> {
        sqlx::query("UPDATE roles SET name = ?, description = ?, permissions = ?, updated_at = ? WHERE id = ?")
            .bind(role.name)
            .bind(role.description)
            .bind(serde_json::to_string(&role.permissions)?)
            .bind(Utc::now().timestamp_millis())
            .bind(role.id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn assign_role(&self, user_role: UserRoleAssignment) -> Result<()> {
        sqlx::query("INSERT INTO user_roles (user_id, role_id, assigned_by, assigned_at) VALUES (?, ?, ?, ?)")
            .bind(user_role.user_id.to_string())
            .bind(user_role.role_id.to_string())
            .bind(user_role.assigned_by.to_string())
            .bind(user_role.assigned_at.timestamp_millis())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn revoke_role(&self, user_id: Uuid, role_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM user_roles WHERE user_id = ? AND role_id = ?")
            .bind(user_id.to_string())
            .bind(role_id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<UserRoleAssignment>> {
        let rows = sqlx::query("SELECT user_id, role_id, assigned_by, assigned_at FROM user_roles WHERE user_id = ?")
            .bind(user_id.to_string())
            .fetch_all(&self.pool)
            .await?;
        
        let mut roles = Vec::new();
        for row in rows {
            roles.push(UserRoleAssignment {
                user_id: Uuid::parse_str(&row.get::<String, _>("user_id"))?,
                role_id: Uuid::parse_str(&row.get::<String, _>("role_id"))?,
                assigned_by: Uuid::parse_str(&row.get::<String, _>("assigned_by"))?,
                assigned_at: chrono::DateTime::from_timestamp_millis(row.get("assigned_at")).unwrap_or_default(),
            });
        }
        Ok(roles)
    }
}
