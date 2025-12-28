/// Namespace operations for SqliteStore
use super::SqliteStore;
use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;
use pangolin_core::model::Namespace;
use std::collections::HashMap;

impl SqliteStore {
    pub async fn create_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Namespace) -> Result<()> {
        let namespace_path = serde_json::to_string(&namespace.name)?;
        sqlx::query("INSERT INTO namespaces (id, tenant_id, catalog_name, namespace_path, properties) VALUES (?, ?, ?, ?, ?)")
            .bind(Uuid::new_v4().to_string())
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(&namespace_path)
            .bind(serde_json::to_string(&namespace.properties)?)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<Option<Namespace>> {
        let namespace_path = serde_json::to_string(&namespace)?;
        let row = sqlx::query("SELECT namespace_path, properties FROM namespaces WHERE tenant_id = ? AND catalog_name = ? AND namespace_path = ?")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(&namespace_path)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(Namespace {
                name: serde_json::from_str(&row.get::<String, _>("namespace_path"))?,
                properties: serde_json::from_str(&row.get::<String, _>("properties"))?,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_namespaces(&self, tenant_id: Uuid, catalog_name: &str, _parent: Option<String>, pagination: Option<crate::PaginationParams>) -> Result<Vec<Namespace>> {
        let limit = pagination.map(|p| p.limit.unwrap_or(i64::MAX as usize) as i64).unwrap_or(-1);
        let offset = pagination.map(|p| p.offset.unwrap_or(0) as i64).unwrap_or(0);

        let rows = sqlx::query("SELECT namespace_path, properties FROM namespaces WHERE tenant_id = ? AND catalog_name = ? LIMIT ? OFFSET ?")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let mut namespaces = Vec::new();
        for row in rows {
            namespaces.push(Namespace {
                name: serde_json::from_str(&row.get::<String, _>("namespace_path"))?,
                properties: serde_json::from_str(&row.get::<String, _>("properties"))?,
            });
        }
        Ok(namespaces)
    }

    pub async fn delete_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<()> {
        let namespace_path = serde_json::to_string(&namespace)?;
        sqlx::query("DELETE FROM namespaces WHERE tenant_id = ? AND catalog_name = ? AND namespace_path = ?")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(&namespace_path)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_namespace_properties(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>, properties: HashMap<String, String>) -> Result<()> {
        let ns = self.get_namespace(tenant_id, catalog_name, namespace.clone()).await?;
        if let Some(mut n) = ns {
            n.properties.extend(properties);
            let namespace_path = serde_json::to_string(&namespace)?;
            sqlx::query("UPDATE namespaces SET properties = ? WHERE tenant_id = ? AND catalog_name = ? AND namespace_path = ?")
                .bind(serde_json::to_string(&n.properties)?)
                .bind(tenant_id.to_string())
                .bind(catalog_name)
                .bind(&namespace_path)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }
}
