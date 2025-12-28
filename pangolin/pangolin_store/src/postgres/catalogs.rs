use super::PostgresStore;
use anyhow::Result;
use pangolin_core::model::{Catalog, CatalogUpdate};
use uuid::Uuid;
use sqlx::Row;

impl PostgresStore {
    // Catalog Operations
    pub async fn create_catalog(&self, tenant_id: Uuid, catalog: Catalog) -> Result<()> {
        sqlx::query("INSERT INTO catalogs (id, tenant_id, name, catalog_type, warehouse_name, storage_location, federated_config, properties) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
            .bind(catalog.id)
            .bind(tenant_id)
            .bind(&catalog.name)
            .bind(format!("{:?}", catalog.catalog_type))
            .bind(&catalog.warehouse_name)
            .bind(&catalog.storage_location)
            .bind(serde_json::to_value(&catalog.federated_config)?)
            .bind(serde_json::to_value(&catalog.properties)?)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_catalog(&self, tenant_id: Uuid, name: String) -> Result<Option<Catalog>> {
        let row: Option<sqlx::postgres::PgRow> = sqlx::query("SELECT id, name, catalog_type, warehouse_name, storage_location, federated_config, properties FROM catalogs WHERE tenant_id = $1 AND name = $2")
            .bind(tenant_id)
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let catalog_type_str: String = row.get("catalog_type");
            let catalog_type = match catalog_type_str.as_str() {
                "Local" => pangolin_core::model::CatalogType::Local,
                "Federated" => pangolin_core::model::CatalogType::Federated,
                _ => pangolin_core::model::CatalogType::Local,
            };
            
            Ok(Some(Catalog {
                id: row.get("id"),
                name: row.get("name"),
                catalog_type,
                warehouse_name: row.get("warehouse_name"),
                storage_location: row.get("storage_location"),
                federated_config: serde_json::from_value(row.get("federated_config")).ok(),
                properties: serde_json::from_value(row.get("properties")).unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_catalogs(&self, tenant_id: Uuid, pagination: Option<crate::PaginationParams>) -> Result<Vec<Catalog>> {
        let limit = pagination.map(|p| p.limit.unwrap_or(i64::MAX as usize) as i64).unwrap_or(i64::MAX);
        let offset = pagination.map(|p| p.offset.unwrap_or(0) as i64).unwrap_or(0);

        let rows = sqlx::query("SELECT id, name, warehouse_name, storage_location, properties FROM catalogs WHERE tenant_id = $1 LIMIT $2 OFFSET $3")
            .bind(tenant_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let mut catalogs = Vec::new();
        for row in rows {
            catalogs.push(Catalog {
                id: row.get("id"),
                name: row.get("name"),
                catalog_type: pangolin_core::model::CatalogType::Local,
                warehouse_name: row.get("warehouse_name"),
                storage_location: row.get("storage_location"),
                federated_config: None,
                properties: serde_json::from_value(row.get("properties")).unwrap_or_default(),
            });
        }
        Ok(catalogs)
    }

    pub async fn update_catalog(&self, tenant_id: Uuid, name: String, updates: CatalogUpdate) -> Result<Catalog> {
        let mut query = String::from("UPDATE catalogs SET ");
        let mut set_clauses = Vec::new();
        let mut bind_count = 1;

        if updates.warehouse_name.is_some() {
            set_clauses.push(format!("warehouse_name = ${}", bind_count));
            bind_count += 1;
        }
        if updates.storage_location.is_some() {
            set_clauses.push(format!("storage_location = ${}", bind_count));
            bind_count += 1;
        }
        if updates.properties.is_some() {
            set_clauses.push(format!("properties = ${}", bind_count));
            bind_count += 1;
        }

        if set_clauses.is_empty() {
            return self.get_catalog(tenant_id, name).await?
                .ok_or_else(|| anyhow::anyhow!("Catalog not found"));
        }

        query.push_str(&set_clauses.join(", "));
        query.push_str(&format!(" WHERE tenant_id = ${} AND name = ${} RETURNING id, name, catalog_type, warehouse_name, storage_location, federated_config, properties", bind_count, bind_count + 1));

        let mut q = sqlx::query(&query);
        if let Some(warehouse_name) = &updates.warehouse_name {
            q = q.bind(warehouse_name);
        }
        if let Some(storage_location) = &updates.storage_location {
            q = q.bind(storage_location);
        }
        if let Some(properties) = &updates.properties {
            q = q.bind(serde_json::to_value(properties)?);
        }
        q = q.bind(tenant_id).bind(&name);

        let row: sqlx::postgres::PgRow = q.fetch_one(&self.pool).await?;
        let catalog_type_str: String = row.get("catalog_type");
        let catalog_type = match catalog_type_str.as_str() {
            "Local" => pangolin_core::model::CatalogType::Local,
            "Federated" => pangolin_core::model::CatalogType::Federated,
            _ => pangolin_core::model::CatalogType::Local,
        };

        Ok(Catalog {
            id: row.get("id"),
            name: row.get("name"),
            catalog_type,
            warehouse_name: row.get("warehouse_name"),
            storage_location: row.get("storage_location"),
            federated_config: serde_json::from_value(row.get("federated_config")).ok(),
            properties: serde_json::from_value(row.get("properties")).unwrap_or_default(),
        })
    }

    pub async fn delete_catalog(&self, tenant_id: Uuid, name: String) -> Result<()> {
        // Manually cascade delete dependent resources
        // 1. Tags
        sqlx::query("DELETE FROM tags WHERE tenant_id = $1 AND catalog_name = $2")
            .bind(tenant_id)
            .bind(&name)
            .execute(&self.pool)
            .await?;

        // 2. Branches
        sqlx::query("DELETE FROM branches WHERE tenant_id = $1 AND catalog_name = $2")
            .bind(tenant_id)
            .bind(&name)
            .execute(&self.pool)
            .await?;

        // 3. Assets
        sqlx::query("DELETE FROM assets WHERE tenant_id = $1 AND catalog_name = $2")
            .bind(tenant_id)
            .bind(&name)
            .execute(&self.pool)
            .await?;

        // 4. Namespaces
        sqlx::query("DELETE FROM namespaces WHERE tenant_id = $1 AND catalog_name = $2")
            .bind(tenant_id)
            .bind(&name)
            .execute(&self.pool)
            .await?;

        // 5. Catalog
        let result: sqlx::postgres::PgQueryResult = sqlx::query("DELETE FROM catalogs WHERE tenant_id = $1 AND name = $2")
            .bind(tenant_id)
            .bind(&name)
            .execute(&self.pool)
            .await?;
        
        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Catalog '{}' not found", name));
        }
        Ok(())
    }
}
