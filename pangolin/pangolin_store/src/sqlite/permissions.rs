/// Permission operations for SqliteStore
use super::SqliteStore;
use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;
use chrono::Utc;
use pangolin_core::permission::{Permission, PermissionGrant};

impl SqliteStore {
    pub async fn create_permission(&self, permission: Permission) -> Result<()> {
        sqlx::query("INSERT INTO permissions (id, user_id, scope, actions, granted_by, granted_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(permission.id.to_string())
            .bind(permission.user_id.to_string())
            .bind(serde_json::to_string(&permission.scope)?)
            .bind(serde_json::to_string(&permission.actions)?)
            .bind(permission.granted_by.to_string())
            .bind(permission.granted_at.timestamp_millis())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn revoke_permission(&self, permission_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM permissions WHERE id = ?").bind(permission_id.to_string()).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn list_user_permissions(&self, user_id: Uuid) -> Result<Vec<Permission>> {
        // 1. Fetch direct permissions
        let rows = sqlx::query("SELECT id, user_id, scope, actions, granted_by, granted_at FROM permissions WHERE user_id = ?")
            .bind(user_id.to_string())
            .fetch_all(&self.pool)
            .await?;
            
        let mut perms = Vec::new();
        for row in rows {
            perms.push(Permission {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                user_id: Uuid::parse_str(&row.get::<String, _>("user_id"))?,
                scope: serde_json::from_str(&row.get::<String, _>("scope"))?,
                actions: serde_json::from_str(&row.get::<String, _>("actions"))?,
                granted_by: Uuid::parse_str(&row.get::<String, _>("granted_by"))?,
                granted_at: chrono::DateTime::from_timestamp_millis(row.get("granted_at")).unwrap_or_default(),
            });
        }

        // 2. Fetch role-based permissions
        let role_rows = sqlx::query(
            "SELECT r.permissions, r.created_by, r.created_at FROM roles r \
             JOIN user_roles ur ON r.id = ur.role_id \
             WHERE ur.user_id = ?"
        )
        .bind(user_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        for row in role_rows {
            let grants_json: String = row.get("permissions");
            let grants: Vec<PermissionGrant> = serde_json::from_str(&grants_json)?;
            let created_by = Uuid::parse_str(&row.get::<String, _>("created_by"))?;
            let created_at = chrono::DateTime::from_timestamp_millis(row.get("created_at")).unwrap_or_default();

            for grant in grants {
                perms.push(Permission {
                    id: Uuid::new_v4(), // Synthesized ID
                    user_id,
                    scope: grant.scope,
                    actions: grant.actions,
                    granted_by: created_by,
                    granted_at: created_at,
                });
            }
        }

        Ok(perms)
    }

    pub async fn list_permissions(&self, tenant_id: Uuid) -> Result<Vec<Permission>> {
        let rows = sqlx::query(
            "SELECT p.id, p.user_id, p.scope, p.actions, p.granted_by, p.granted_at 
             FROM permissions p
             JOIN users u ON p.user_id = u.id
             WHERE u.tenant_id = ?"
        )
        .bind(tenant_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        let mut perms = Vec::new();
        for row in rows {
            perms.push(Permission {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                user_id: Uuid::parse_str(&row.get::<String, _>("user_id"))?,
                scope: serde_json::from_str(&row.get::<String, _>("scope"))?,
                actions: serde_json::from_str(&row.get::<String, _>("actions"))?,
                granted_by: Uuid::parse_str(&row.get::<String, _>("granted_by"))?,
                granted_at: chrono::DateTime::from_timestamp_millis(row.get("granted_at")).unwrap_or_default(),
            });
        }
        Ok(perms)
    }
}
