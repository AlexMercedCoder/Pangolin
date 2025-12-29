use super::PostgresStore;
use anyhow::Result;
use pangolin_core::permission::{Permission, PermissionGrant, PermissionScope, Action};
use uuid::Uuid;
use sqlx::Row;
use chrono::{DateTime, Utc};
use std::collections::HashSet;

impl PostgresStore {
    // Direct Permission Operations
    pub async fn create_permission(&self, permission: Permission) -> Result<()> {
        sqlx::query("INSERT INTO permissions (id, user_id, tenant_id, scope, actions, granted_by, granted_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(permission.id)
            .bind(permission.user_id)
            .bind(permission.tenant_id)
            .bind(serde_json::to_value(&permission.scope)?)
            .bind(serde_json::to_value(&permission.actions)?)
            .bind(permission.granted_by)
            .bind(permission.granted_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn revoke_permission(&self, permission_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM permissions WHERE id = $1")
            .bind(permission_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list_user_permissions(&self, user_id: Uuid, pagination: Option<crate::PaginationParams>) -> Result<Vec<Permission>> {
        // 1. Fetch direct permissions
        let rows = sqlx::query("SELECT id, user_id, tenant_id, scope, actions, granted_by, granted_at FROM permissions WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
        
        let mut perms = Vec::new();
        for row in rows {
            perms.push(Permission {
                id: row.get("id"),
                user_id: row.get("user_id"),
                tenant_id: row.get("tenant_id"),
                scope: serde_json::from_value::<PermissionScope>(row.get("scope"))?,
                actions: serde_json::from_value::<HashSet<Action>>(row.get("actions"))?,
                granted_by: row.get("granted_by"),
                granted_at: row.get("granted_at"),
            });
        }

        // 2. Fetch role-based permissions
        let role_rows = sqlx::query(
            "SELECT r.permissions, r.created_by, r.created_at, r.tenant_id FROM roles r \
             JOIN user_roles ur ON r.id = ur.role_id \
             WHERE ur.user_id = $1"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        for row in role_rows {
            let grants: Vec<PermissionGrant> = serde_json::from_value(row.get("permissions"))?;
            let created_by: Uuid = row.get("created_by");
            let created_at: DateTime<Utc> = row.get("created_at");
            let tenant_id: Uuid = row.get("tenant_id");

            for grant in grants {
                perms.push(Permission {
                    id: Uuid::new_v4(), // Synthesized ID
                    user_id,
                    tenant_id,
                    scope: grant.scope,
                    actions: grant.actions,
                    granted_by: created_by,
                    granted_at: created_at,
                });
            }
        }

        // In-memory pagination
        if let Some(p) = pagination {
             let limit = p.limit.unwrap_or(usize::MAX);
             let offset = p.offset.unwrap_or(0);
             if offset >= perms.len() {
                 return Ok(Vec::new());
             }
             let end = std::cmp::min(offset + limit, perms.len());
             Ok(perms[offset..end].to_vec())
        } else {
             Ok(perms)
        }
    }

    pub async fn list_permissions(&self, tenant_id: Uuid, pagination: Option<crate::PaginationParams>) -> Result<Vec<Permission>> {
        let limit = pagination.map(|p| p.limit.unwrap_or(i64::MAX as usize) as i64).unwrap_or(i64::MAX);
        let offset = pagination.map(|p| p.offset.unwrap_or(0) as i64).unwrap_or(0);

        let rows = sqlx::query(
            "SELECT id, user_id, tenant_id, scope, actions, granted_by, granted_at 
             FROM permissions 
             WHERE tenant_id = $1
             LIMIT $2 OFFSET $3"
        )
            .bind(tenant_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let mut perms = Vec::new();
        for row in rows {
            perms.push(Permission {
                id: row.get("id"),
                user_id: row.get("user_id"),
                tenant_id: row.get("tenant_id"),
                scope: serde_json::from_value::<PermissionScope>(row.get("scope"))?,
                actions: serde_json::from_value::<HashSet<Action>>(row.get("actions"))?,
                granted_by: row.get("granted_by"),
                granted_at: row.get("granted_at"),
            });
        }
        Ok(perms)
    }
}
