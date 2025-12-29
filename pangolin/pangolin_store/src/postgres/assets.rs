use super::PostgresStore;
use anyhow::Result;
use pangolin_core::model::Asset;
use uuid::Uuid;
use sqlx::Row;

impl PostgresStore {
    // Asset Operations
    pub async fn create_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, asset: Asset) -> Result<()> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        sqlx::query("INSERT INTO assets (id, tenant_id, catalog_name, branch_name, namespace_path, name, asset_type, metadata_location, properties) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)")
            .bind(asset.id)
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(&branch_name)
            .bind(&namespace)
            .bind(&asset.name)
            .bind(format!("{:?}", asset.kind))
            .bind(asset.properties.get("metadata_location").unwrap_or(&asset.location))
            .bind(serde_json::to_value(&asset.properties)?)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<Option<Asset>> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        let row: Option<sqlx::postgres::PgRow> = sqlx::query("SELECT id, name, asset_type, metadata_location, properties FROM assets WHERE tenant_id = $1 AND catalog_name = $2 AND branch_name = $3 AND namespace_path = $4 AND name = $5")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(&branch_name)
            .bind(&namespace)
            .bind(&name)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let asset_type_str: String = row.get("asset_type");
            let kind = match asset_type_str.as_str() {
                "IcebergTable" => pangolin_core::model::AssetType::IcebergTable,
                "View" => pangolin_core::model::AssetType::View,
                _ => pangolin_core::model::AssetType::IcebergTable,
            };
            
            let a = Asset {
                id: row.get("id"),
                name: row.get("name"),
                kind,
                location: row.get::<Option<String>, _>("metadata_location").unwrap_or_default(),
                properties: serde_json::from_value(row.get("properties")).unwrap_or_default(),
            };
            Ok(Some(a))
        } else {
            Ok(None)
        }
    }

    pub async fn get_asset_by_id(&self, tenant_id: Uuid, asset_id: Uuid) -> Result<Option<(Asset, String, Vec<String>)>> {
        let row: Option<sqlx::postgres::PgRow> = sqlx::query("SELECT id, name, catalog_name, namespace_path, asset_type, metadata_location, properties FROM assets WHERE tenant_id = $1 AND id = $2")
            .bind(tenant_id)
            .bind(asset_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let catalog_name: String = row.get("catalog_name");
            let namespace_path: Vec<String> = row.get("namespace_path");
            let asset_type_str: String = row.get("asset_type");
            let kind = match asset_type_str.as_str() {
                "IcebergTable" => pangolin_core::model::AssetType::IcebergTable,
                "View" => pangolin_core::model::AssetType::View,
                _ => pangolin_core::model::AssetType::IcebergTable,
            };
            
            let asset = Asset {
                id: row.get("id"),
                name: row.get("name"),
                kind,
                location: row.get::<Option<String>, _>("metadata_location").unwrap_or_default(),
                properties: serde_json::from_value(row.get("properties")).unwrap_or_default(),
            };
            
            Ok(Some((asset, catalog_name, namespace_path)))
        } else {
            Ok(None)
        }
    }

    pub async fn list_assets(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, pagination: Option<crate::PaginationParams>) -> Result<Vec<Asset>> {
        let limit = pagination.map(|p| p.limit.unwrap_or(i64::MAX as usize) as i64).unwrap_or(i64::MAX);
        let offset = pagination.map(|p| p.offset.unwrap_or(0) as i64).unwrap_or(0);
        let branch_name = branch.unwrap_or_else(|| "main".to_string());

        let rows = sqlx::query("SELECT id, name, asset_type, metadata_location, properties FROM assets WHERE tenant_id = $1 AND catalog_name = $2 AND branch_name = $3 AND namespace_path = $4 ORDER BY name LIMIT $5 OFFSET $6")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(&branch_name)
            .bind(&namespace)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let mut assets = Vec::new();
        for row in rows {
            let asset_type_str: String = row.get("asset_type");
            let kind = match asset_type_str.as_str() {
                "IcebergTable" => pangolin_core::model::AssetType::IcebergTable,
                "View" => pangolin_core::model::AssetType::View,
                _ => pangolin_core::model::AssetType::IcebergTable,
            };
            
            let a = Asset {
                id: row.get("id"),
                name: row.get("name"),
                kind,
                location: row.get::<Option<String>, _>("metadata_location").unwrap_or_default(),
                properties: serde_json::from_value(row.get("properties")).unwrap_or_default(),
            };
            assets.push(a);
        }
        Ok(assets)
    }

    pub async fn delete_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<()> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        sqlx::query("DELETE FROM assets WHERE tenant_id = $1 AND catalog_name = $2 AND branch_name = $3 AND namespace_path = $4 AND name = $5")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(&branch_name)
            .bind(&namespace)
            .bind(&name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn rename_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, source_namespace: Vec<String>, source_name: String, dest_namespace: Vec<String>, dest_name: String) -> Result<()> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        sqlx::query("UPDATE assets SET namespace_path = $1, name = $2 WHERE tenant_id = $3 AND catalog_name = $4 AND branch_name = $5 AND namespace_path = $6 AND name = $7")
            .bind(&dest_namespace)
            .bind(&dest_name)
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(&branch_name)
            .bind(&source_namespace)
            .bind(&source_name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn count_assets(&self, tenant_id: Uuid) -> Result<usize> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM assets WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(count as usize)
    }

    pub async fn copy_assets_bulk(
        &self,
        tenant_id: Uuid,
        catalog_name: &str,
        src_branch: &str,
        dest_branch: &str,
        namespace_filter: Option<String>,
    ) -> Result<usize> {
        // 1. Select the assets to copy
        // Postgres: namespace_path is TEXT[], so we map to Vec<String>.
        // Filters handled in memory for simplicity unless performance dictates otherwise.
        
        let rows = sqlx::query("SELECT namespace_path, name, asset_type, metadata_location, properties FROM assets WHERE tenant_id = $1 AND catalog_name = $2 AND branch_name = $3")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(src_branch)
            .fetch_all(&self.pool)
            .await?;

        let mut assets_to_copy = Vec::new();
        for row in rows {
            let ns_path: Vec<String> = row.get("namespace_path"); 
            let name: String = row.get("name");
            
            // Filter
            if let Some(ref filter_ns) = namespace_filter {
                // If filter_ns is expected to be JSON string of array, parse it.
                // Assuming strict equality for now based on MemoryStore behavior.
                let ns_json = serde_json::to_string(&ns_path).unwrap_or_default();
                if ns_json != *filter_ns { continue; }
            }

            let atype: String = row.get("asset_type");
            let floc: Option<String> = row.get("metadata_location");
            let props: serde_json::Value = row.get("properties"); 
            
            assets_to_copy.push((ns_path, name, atype, floc, props));
        }

        if assets_to_copy.is_empty() {
             return Ok(0);
        }

        // 2. Bulk Insert
        let mut query_builder: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
            "INSERT INTO assets (id, tenant_id, catalog_name, namespace_path, name, branch_name, asset_type, metadata_location, properties) "
        );

        query_builder.push_values(assets_to_copy, |mut b, (ns, name, atype, floc, props)| {
            b.push_bind(Uuid::new_v4())
             .push_bind(tenant_id)
             .push_bind(catalog_name)
             .push_bind(ns)
             .push_bind(name)
             .push_bind(dest_branch)
             .push_bind(atype)
             .push_bind(floc)
             .push_bind(props);
        });

        let result = query_builder.build().execute(&self.pool).await?;
        Ok(result.rows_affected() as usize)
    }
}
