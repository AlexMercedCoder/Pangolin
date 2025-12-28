use super::PostgresStore;
use anyhow::Result;
use pangolin_core::model::Namespace;
use uuid::Uuid;
use sqlx::Row;

impl PostgresStore {
    // Namespace Operations
    pub async fn create_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Namespace) -> Result<()> {
        sqlx::query("INSERT INTO namespaces (id, tenant_id, catalog_name, namespace_path, properties) VALUES ($1, $2, $3, $4, $5)")
            .bind(Uuid::new_v4())
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(&namespace.name)
            .bind(serde_json::to_value(&namespace.properties)?)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<Option<Namespace>> {
        let row: Option<sqlx::postgres::PgRow> = sqlx::query("SELECT namespace_path, properties FROM namespaces WHERE tenant_id = $1 AND catalog_name = $2 AND namespace_path = $3")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(&namespace)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(Namespace {
                name: row.get("namespace_path"),
                properties: serde_json::from_value(row.get("properties"))?,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_namespaces(&self, tenant_id: Uuid, catalog_name: &str, _parent: Option<String>, pagination: Option<crate::PaginationParams>) -> Result<Vec<Namespace>> {
        let limit = pagination.map(|p| p.limit.unwrap_or(i64::MAX as usize) as i64).unwrap_or(i64::MAX);
        let offset = pagination.map(|p| p.offset.unwrap_or(0) as i64).unwrap_or(0);

        let rows = sqlx::query("SELECT namespace_path, properties FROM namespaces WHERE tenant_id = $1 AND catalog_name = $2 LIMIT $3 OFFSET $4")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let mut namespaces = Vec::new();
        for row in rows {
            namespaces.push(Namespace {
                name: row.get("namespace_path"),
                properties: serde_json::from_value(row.get("properties"))?,
            });
        }
        Ok(namespaces)
    }

    pub async fn delete_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<()> {
        sqlx::query("DELETE FROM namespaces WHERE tenant_id = $1 AND catalog_name = $2 AND namespace_path = $3")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(&namespace)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_namespace_properties(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>, properties: std::collections::HashMap<String, String>) -> Result<()> {
        let ns = self.get_namespace(tenant_id, catalog_name, namespace.clone()).await?;
        if let Some(mut n) = ns {
            n.properties.extend(properties);
            sqlx::query("UPDATE namespaces SET properties = $1 WHERE tenant_id = $2 AND catalog_name = $3 AND namespace_path = $4")
                .bind(serde_json::to_value(&n.properties)?)
                .bind(tenant_id)
                .bind(catalog_name)
                .bind(&namespace)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    pub async fn count_namespaces(&self, tenant_id: Uuid) -> Result<usize> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM namespaces WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(count as usize)
    }
}
