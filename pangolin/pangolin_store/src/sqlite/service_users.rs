/// Service Users implementation for SqliteStore
use async_trait::async_trait;
use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;
use chrono::Utc;
use pangolin_core::user::{ServiceUser, UserRole};
use std::str::FromStr;

use super::SqliteStore;

impl SqliteStore {
    pub async fn create_service_user(&self, service_user: ServiceUser) -> Result<()> {
        sqlx::query(
            "INSERT INTO service_users (id, name, description, tenant_id, api_key_hash, role, created_at, created_by, last_used, expires_at, active)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(service_user.id.to_string())
        .bind(service_user.name)
        .bind(service_user.description)
        .bind(service_user.tenant_id.to_string())
        .bind(service_user.api_key_hash)
        .bind(format!("{:?}", service_user.role))
        .bind(service_user.created_at.timestamp())
        .bind(service_user.created_by.to_string())
        .bind(service_user.last_used.map(|t| t.timestamp()))
        .bind(service_user.expires_at.map(|t| t.timestamp()))
        .bind(service_user.active as i32)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn get_service_user(&self, id: Uuid) -> Result<Option<ServiceUser>> {
        let row = sqlx::query(
            "SELECT id, name, description, tenant_id, api_key_hash, role, created_at, created_by, last_used, expires_at, active
             FROM service_users WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let role_str: String = row.get("role");
                let role = match role_str.as_str() {
                    "Root" => UserRole::Root,
                    "TenantAdmin" => UserRole::TenantAdmin,
                    "TenantUser" => UserRole::TenantUser,
                    _ => UserRole::TenantUser,
                };
                
                Ok(Some(ServiceUser {
                    id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                    name: row.get("name"),
                    description: row.get("description"),
                    tenant_id: Uuid::parse_str(&row.get::<String, _>("tenant_id"))?,
                    api_key_hash: row.get("api_key_hash"),
                    role,
                    created_at: chrono::DateTime::from_timestamp(row.get("created_at"), 0).unwrap(),
                    created_by: Uuid::parse_str(&row.get::<String, _>("created_by"))?,
                    last_used: row.get::<Option<i64>, _>("last_used").and_then(|t| chrono::DateTime::from_timestamp(t, 0)),
                    expires_at: row.get::<Option<i64>, _>("expires_at").and_then(|t| chrono::DateTime::from_timestamp(t, 0)),
                    active: row.get::<i32, _>("active") != 0,
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn get_service_user_by_api_key_hash(&self, api_key_hash: &str) -> Result<Option<ServiceUser>> {
        let row = sqlx::query(
            "SELECT id, name, description, tenant_id, api_key_hash, role, created_at, created_by, last_used, expires_at, active
             FROM service_users WHERE api_key_hash = ?"
        )
        .bind(api_key_hash)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let role_str: String = row.get("role");
                let role = match role_str.as_str() {
                    "Root" => UserRole::Root,
                    "TenantAdmin" => UserRole::TenantAdmin,
                    "TenantUser" => UserRole::TenantUser,
                    _ => UserRole::TenantUser,
                };
                
                Ok(Some(ServiceUser {
                    id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                    name: row.get("name"),
                    description: row.get("description"),
                    tenant_id: Uuid::parse_str(&row.get::<String, _>("tenant_id"))?,
                    api_key_hash: row.get("api_key_hash"),
                    role,
                    created_at: chrono::DateTime::from_timestamp(row.get("created_at"), 0).unwrap(),
                    created_by: Uuid::parse_str(&row.get::<String, _>("created_by"))?,
                    last_used: row.get::<Option<i64>, _>("last_used").and_then(|t| chrono::DateTime::from_timestamp(t, 0)),
                    expires_at: row.get::<Option<i64>, _>("expires_at").and_then(|t| chrono::DateTime::from_timestamp(t, 0)),
                    active: row.get::<i32, _>("active") != 0,
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn list_service_users(&self, tenant_id: Uuid) -> Result<Vec<ServiceUser>> {
        let rows = sqlx::query(
            "SELECT id, name, description, tenant_id, api_key_hash, role, created_at, created_by, last_used, expires_at, active
             FROM service_users WHERE tenant_id = ?"
        )
        .bind(tenant_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        let mut users = Vec::new();
        for row in rows {
            let role_str: String = row.get("role");
            let role = match role_str.as_str() {
                "Root" => UserRole::Root,
                "TenantAdmin" => UserRole::TenantAdmin,
                "TenantUser" => UserRole::TenantUser,
                _ => UserRole::TenantUser,
            };
            
            users.push(ServiceUser {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                name: row.get("name"),
                description: row.get("description"),
                tenant_id: Uuid::parse_str(&row.get::<String, _>("tenant_id"))?,
                api_key_hash: row.get("api_key_hash"),
                role,
                created_at: chrono::DateTime::from_timestamp(row.get("created_at"), 0).unwrap(),
                created_by: Uuid::parse_str(&row.get::<String, _>("created_by"))?,
                last_used: row.get::<Option<i64>, _>("last_used").and_then(|t| chrono::DateTime::from_timestamp(t, 0)),
                expires_at: row.get::<Option<i64>, _>("expires_at").and_then(|t| chrono::DateTime::from_timestamp(t, 0)),
                active: row.get::<i32, _>("active") != 0,
            });
        }

        Ok(users)
    }

    pub async fn update_service_user(&self, id: Uuid, name: Option<String>, description: Option<String>, active: Option<bool>) -> Result<()> {
        let mut query = String::from("UPDATE service_users SET ");
        let mut updates = Vec::new();
        let mut params: Vec<String> = Vec::new();

        if let Some(n) = name {
            updates.push("name = ?");
            params.push(n);
        }
        if let Some(d) = description {
            updates.push("description = ?");
            params.push(d);
        }
        if let Some(a) = active {
            updates.push("active = ?");
            params.push((a as i32).to_string());
        }

        if updates.is_empty() {
            return Ok(());
        }

        query.push_str(&updates.join(", "));
        query.push_str(" WHERE id = ?");
        params.push(id.to_string());

        let mut q = sqlx::query(&query);
        for param in params {
            q = q.bind(param);
        }

        q.execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_service_user(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM service_users WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_service_user_last_used(&self, id: Uuid, timestamp: chrono::DateTime<Utc>) -> Result<()> {
        sqlx::query("UPDATE service_users SET last_used = ? WHERE id = ?")
            .bind(timestamp.timestamp())
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
