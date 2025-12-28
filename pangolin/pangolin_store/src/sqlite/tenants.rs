/// Tenant operations for SqliteStore
use super::SqliteStore;
use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;
use pangolin_core::model::{Tenant, TenantUpdate};

impl SqliteStore {
    pub async fn create_tenant(&self, tenant: Tenant) -> Result<()> {
        sqlx::query("INSERT INTO tenants (id, name, properties) VALUES (?, ?, ?)")
            .bind(tenant.id.to_string())
            .bind(&tenant.name)
            .bind(serde_json::to_string(&tenant.properties)?)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_tenant(&self, id: Uuid) -> Result<Option<Tenant>> {
        let row = sqlx::query("SELECT id, name, properties FROM tenants WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(Tenant {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                name: row.get("name"),
                properties: serde_json::from_str(&row.get::<String, _>("properties"))?,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_tenants(&self, pagination: Option<crate::PaginationParams>) -> Result<Vec<Tenant>> {
        let limit = pagination.map(|p| p.limit.unwrap_or(i64::MAX as usize) as i64).unwrap_or(-1);
        let offset = pagination.map(|p| p.offset.unwrap_or(0) as i64).unwrap_or(0);

        let rows = sqlx::query("SELECT id, name, properties FROM tenants LIMIT ? OFFSET ?")
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let mut tenants = Vec::new();
        for row in rows {
            tenants.push(Tenant {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                name: row.get("name"),
                properties: serde_json::from_str(&row.get::<String, _>("properties"))?,
            });
        }
        Ok(tenants)
    }

    pub async fn update_tenant(&self, tenant_id: Uuid, updates: TenantUpdate) -> Result<Tenant> {
        let mut query = String::from("UPDATE tenants SET ");
        let mut set_clauses = Vec::new();
        
        if updates.name.is_some() {
            set_clauses.push("name = ?");
        }
        if updates.properties.is_some() {
            set_clauses.push("properties = ?");
        }
        
        if set_clauses.is_empty() {
            return self.get_tenant(tenant_id).await?
                .ok_or_else(|| anyhow::anyhow!("Tenant not found"));
        }
        
        query.push_str(&set_clauses.join(", "));
        query.push_str(" WHERE id = ?");
        
        let mut q = sqlx::query(&query);
        if let Some(name) = &updates.name {
            q = q.bind(name);
        }
        if let Some(properties) = &updates.properties {
            q = q.bind(serde_json::to_string(properties)?);
        }
        q = q.bind(tenant_id.to_string());
        
        q.execute(&self.pool).await?;
        
        self.get_tenant(tenant_id).await?
            .ok_or_else(|| anyhow::anyhow!("Tenant not found"))
    }

    pub async fn delete_tenant(&self, tenant_id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM tenants WHERE id = ?")
            .bind(tenant_id.to_string())
            .execute(&self.pool)
            .await?;
        
        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Tenant not found"));
        }
        Ok(())
    }
}
