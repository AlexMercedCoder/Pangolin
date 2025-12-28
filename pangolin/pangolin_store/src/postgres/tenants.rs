use super::PostgresStore;
use anyhow::Result;
use pangolin_core::model::{Tenant, TenantUpdate};
use uuid::Uuid;
use sqlx::Row;

impl PostgresStore {
    // Tenant Operations
    pub async fn create_tenant(&self, tenant: Tenant) -> Result<()> {
        sqlx::query("INSERT INTO tenants (id, name, properties) VALUES ($1, $2, $3)")
            .bind(tenant.id)
            .bind(tenant.name)
            .bind(serde_json::to_value(&tenant.properties)?)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_tenant(&self, id: Uuid) -> Result<Option<Tenant>> {
        let row: Option<sqlx::postgres::PgRow> = sqlx::query("SELECT id, name, properties FROM tenants WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(Tenant {
                id: row.get("id"),
                name: row.get("name"),
                properties: serde_json::from_value(row.get("properties")).unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_tenants(&self, pagination: Option<crate::PaginationParams>) -> Result<Vec<Tenant>> {
        let limit = pagination.map(|p| p.limit.unwrap_or(i64::MAX as usize) as i64).unwrap_or(i64::MAX);
        let offset = pagination.map(|p| p.offset.unwrap_or(0) as i64).unwrap_or(0);

        let rows = sqlx::query("SELECT id, name, properties FROM tenants LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let mut tenants = Vec::new();
        for row in rows {
            tenants.push(Tenant {
                id: row.get("id"),
                name: row.get("name"),
                properties: serde_json::from_value(row.get("properties")).unwrap_or_default(),
            });
        }
        Ok(tenants)
    }

    pub async fn update_tenant(&self, tenant_id: Uuid, updates: TenantUpdate) -> Result<Tenant> {
        let mut query = String::from("UPDATE tenants SET ");
        let mut set_clauses = Vec::new();
        let mut bind_count = 1;

        if updates.name.is_some() {
            set_clauses.push(format!("name = ${}", bind_count));
            bind_count += 1;
        }
        if updates.properties.is_some() {
            set_clauses.push(format!("properties = ${}", bind_count));
            bind_count += 1;
        }

        if set_clauses.is_empty() {
             return self.get_tenant(tenant_id).await?
                .ok_or_else(|| anyhow::anyhow!("Tenant not found"));
        }

        query.push_str(&set_clauses.join(", "));
        query.push_str(&format!(" WHERE id = ${} RETURNING id, name, properties", bind_count));

        let mut q = sqlx::query(&query);
        if let Some(name) = &updates.name {
            q = q.bind(name);
        }
        if let Some(properties) = &updates.properties {
            q = q.bind(serde_json::to_value(properties)?);
        }
        q = q.bind(tenant_id);

        let row: sqlx::postgres::PgRow = q.fetch_one(&self.pool).await?;

        Ok(Tenant {
            id: row.get("id"),
            name: row.get("name"),
            properties: serde_json::from_value(row.get("properties")).unwrap_or_default(),
        })
    }

    pub async fn delete_tenant(&self, tenant_id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM tenants WHERE id = $1")
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;
        
        if result.rows_affected() == 0 {
             return Err(anyhow::anyhow!("Tenant not found"));
        }
        Ok(())
    }
}
