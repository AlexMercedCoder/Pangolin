/// Catalog operations for SqliteStore
use super::SqliteStore;
use anyhow::Result;
use pangolin_core::model::{Catalog, CatalogType, CatalogUpdate};
use sqlx::Row;
use uuid::Uuid;

impl SqliteStore {
    pub async fn create_catalog(&self, tenant_id: Uuid, catalog: Catalog) -> Result<()> {
        sqlx::query("INSERT INTO catalogs (id, tenant_id, name, catalog_type, warehouse_name, storage_location, federated_config, properties) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(catalog.id.to_string())
            .bind(tenant_id.to_string())
            .bind(&catalog.name)
            .bind(format!("{:?}", catalog.catalog_type))
            .bind(&catalog.warehouse_name)
            .bind(&catalog.storage_location)
            .bind(serde_json::to_string(&catalog.federated_config)?)
            .bind(serde_json::to_string(&catalog.properties)?)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_catalog(&self, tenant_id: Uuid, name: String) -> Result<Option<Catalog>> {
        let row = sqlx::query("SELECT id, name, catalog_type, warehouse_name, storage_location, federated_config, properties FROM catalogs WHERE tenant_id = ? AND name = ?")
            .bind(tenant_id.to_string())
            .bind(&name)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let catalog_type_str: String = row.get("catalog_type");
            let catalog_type = match catalog_type_str.as_str() {
                "Local" => CatalogType::Local,
                "Federated" => CatalogType::Federated,
                _ => CatalogType::Local,
            };

            Ok(Some(Catalog {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                name: row.get("name"),
                catalog_type,
                warehouse_name: row.get("warehouse_name"),
                storage_location: row.get("storage_location"),
                federated_config: serde_json::from_str(&row.get::<String, _>("federated_config"))
                    .ok(),
                properties: serde_json::from_str(&row.get::<String, _>("properties"))
                    .unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_catalogs(
        &self,
        tenant_id: Uuid,
        pagination: Option<crate::PaginationParams>,
    ) -> Result<Vec<Catalog>> {
        let limit = pagination
            .map(|p| p.limit.unwrap_or(i64::MAX as usize) as i64)
            .unwrap_or(-1);
        let offset = pagination
            .map(|p| p.offset.unwrap_or(0) as i64)
            .unwrap_or(0);

        let rows = sqlx::query("SELECT id, name, catalog_type, warehouse_name, storage_location, federated_config, properties FROM catalogs WHERE tenant_id = ? ORDER BY name LIMIT ? OFFSET ?")
            .bind(tenant_id.to_string())
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let mut catalogs = Vec::new();
        for row in rows {
            let catalog_type_str: String = row.get("catalog_type");
            let catalog_type = match catalog_type_str.as_str() {
                "Local" => CatalogType::Local,
                "Federated" => CatalogType::Federated,
                _ => CatalogType::Local,
            };

            catalogs.push(Catalog {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                name: row.get("name"),
                catalog_type,
                warehouse_name: row.get("warehouse_name"),
                storage_location: row.get("storage_location"),
                federated_config: serde_json::from_str(&row.get::<String, _>("federated_config"))
                    .ok(),
                properties: serde_json::from_str(&row.get::<String, _>("properties"))
                    .unwrap_or_default(),
            });
        }
        Ok(catalogs)
    }

    pub async fn update_catalog(
        &self,
        tenant_id: Uuid,
        name: String,
        updates: CatalogUpdate,
    ) -> Result<Catalog> {
        let mut query = String::from("UPDATE catalogs SET ");
        let mut set_clauses = Vec::new();

        if updates.warehouse_name.is_some() {
            set_clauses.push("warehouse_name = ?");
        }
        if updates.storage_location.is_some() {
            set_clauses.push("storage_location = ?");
        }
        if updates.properties.is_some() {
            set_clauses.push("properties = ?");
        }

        if set_clauses.is_empty() {
            return self
                .get_catalog(tenant_id, name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Catalog not found"));
        }

        query.push_str(&set_clauses.join(", "));
        query.push_str(" WHERE tenant_id = ? AND name = ?");

        let mut q = sqlx::query(&query);
        if let Some(warehouse_name) = &updates.warehouse_name {
            q = q.bind(warehouse_name);
        }
        if let Some(storage_location) = &updates.storage_location {
            q = q.bind(storage_location);
        }
        if let Some(properties) = &updates.properties {
            q = q.bind(serde_json::to_string(properties)?);
        }
        q = q.bind(tenant_id.to_string()).bind(&name);

        q.execute(&self.pool).await?;

        self.get_catalog(tenant_id, name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Catalog not found"))
    }

    /// Delete a catalog and cascade to its children.
    ///
    /// B21: this used to run five sequential deletes with no transaction, and
    /// the "does the catalog exist?" check was the *last* statement. So
    /// `delete_catalog(tenant, "nonexistent")` cheerfully deleted every tag,
    /// branch, asset and namespace whose `catalog_name` matched - and only then
    /// returned "not found", leaving the caller believing nothing had happened.
    /// Any failure part-way through left the same wreckage. Postgres wraps the
    /// identical cascade in a transaction; this now matches it, and checks
    /// existence first.
    pub async fn delete_catalog(&self, tenant_id: Uuid, name: String) -> Result<()> {
        let tid = tenant_id.to_string();

        let mut tx = self.pool.begin().await?;

        // Existence first: nothing is destroyed on behalf of a catalog that is
        // not there.
        let exists: Option<(i64,)> =
            sqlx::query_as("SELECT 1 FROM catalogs WHERE tenant_id = ? AND name = ?")
                .bind(&tid)
                .bind(&name)
                .fetch_optional(&mut *tx)
                .await?;
        if exists.is_none() {
            return Err(anyhow::anyhow!("Catalog '{}' not found", name));
        }

        // Delete cascading children manually (no FK constraints on catalog_name)
        for table in ["tags", "branches", "assets", "namespaces"] {
            let sql = format!("DELETE FROM {table} WHERE tenant_id = ? AND catalog_name = ?");
            sqlx::query(&sql)
                .bind(&tid)
                .bind(&name)
                .execute(&mut *tx)
                .await?;
        }

        sqlx::query("DELETE FROM catalogs WHERE tenant_id = ? AND name = ?")
            .bind(&tid)
            .bind(&name)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }
}
