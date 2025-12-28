use super::PostgresStore;
use anyhow::Result;
use pangolin_core::permission::{Role, UserRole as UserRoleAssignment, PermissionGrant};
use uuid::Uuid;
use sqlx::Row;

impl PostgresStore {
    // Role Operations
    pub async fn create_role(&self, role: Role) -> Result<()> {
        sqlx::query("INSERT INTO roles (id, tenant_id, name, description, permissions, created_by, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
            .bind(role.id)
            .bind(role.tenant_id)
            .bind(role.name)
            .bind(role.description)
            .bind(serde_json::to_value(&role.permissions)?)
            .bind(role.created_by)
            .bind(role.created_at)
            .bind(role.updated_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_role(&self, role_id: Uuid) -> Result<Option<Role>> {
        let row: Option<sqlx::postgres::PgRow> = sqlx::query("SELECT id, tenant_id, name, description, permissions, created_by, created_at, updated_at FROM roles WHERE id = $1")
            .bind(role_id)
            .fetch_optional(&self.pool)
            .await?;
        
        if let Some(row) = row {
            Ok(Some(Role {
                id: row.get("id"),
                tenant_id: row.get("tenant_id"),
                name: row.get("name"),
                description: row.get("description"),
                permissions: serde_json::from_value::<Vec<PermissionGrant>>(row.get("permissions"))?,
                created_by: row.get("created_by"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_roles(&self, tenant_id: Uuid, pagination: Option<crate::PaginationParams>) -> Result<Vec<Role>> {
        let limit = pagination.map(|p| p.limit.unwrap_or(i64::MAX as usize) as i64).unwrap_or(i64::MAX);
        let offset = pagination.map(|p| p.offset.unwrap_or(0) as i64).unwrap_or(0);

        let rows = sqlx::query("SELECT id, tenant_id, name, description, permissions, created_by, created_at, updated_at FROM roles WHERE tenant_id = $1 LIMIT $2 OFFSET $3")
            .bind(tenant_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;
        
        let mut roles = Vec::new();
        for row in rows {
            roles.push(Role {
                id: row.get("id"),
                tenant_id: row.get("tenant_id"),
                name: row.get("name"),
                description: row.get("description"),
                permissions: serde_json::from_value::<Vec<PermissionGrant>>(row.get("permissions"))?,
                created_by: row.get("created_by"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        Ok(roles)
    }

    pub async fn delete_role(&self, role_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM roles WHERE id = $1").bind(role_id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn update_role(&self, role: Role) -> Result<()> {
        sqlx::query("UPDATE roles SET name = $1, description = $2, permissions = $3, updated_at = $4 WHERE id = $5")
            .bind(role.name)
            .bind(role.description)
            .bind(serde_json::to_value(&role.permissions)?)
            .bind(chrono::Utc::now())
            .bind(role.id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn assign_role(&self, user_role: UserRoleAssignment) -> Result<()> {
        sqlx::query("INSERT INTO user_roles (user_id, role_id, assigned_by, assigned_at) VALUES ($1, $2, $3, $4)")
            .bind(user_role.user_id)
            .bind(user_role.role_id)
            .bind(user_role.assigned_by)
            .bind(user_role.assigned_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn revoke_role(&self, user_id: Uuid, role_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2")
            .bind(user_id)
            .bind(role_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<UserRoleAssignment>> {
        let rows = sqlx::query("SELECT user_id, role_id, assigned_by, assigned_at FROM user_roles WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
        
        let mut roles = Vec::new();
        for row in rows {
            roles.push(UserRoleAssignment {
                user_id: row.get("user_id"),
                role_id: row.get("role_id"),
                assigned_by: row.get("assigned_by"),
                assigned_at: row.get("assigned_at"),
            });
        }
        Ok(roles)
    }
}
