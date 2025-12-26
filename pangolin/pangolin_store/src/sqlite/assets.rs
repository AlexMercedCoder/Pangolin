/// Asset operations for SqliteStore
use super::SqliteStore;
use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;
use pangolin_core::model::{Asset, AssetType};

impl SqliteStore {
    pub async fn create_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, asset: Asset) -> Result<()> {
        let namespace_path = serde_json::to_string(&namespace)?;
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        sqlx::query("INSERT INTO assets (id, tenant_id, catalog_name, namespace_path, name, branch_name, asset_type, metadata_location, properties) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(asset.id.to_string())
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(&namespace_path)
            .bind(&asset.name)
            .bind(&branch_name)
            .bind(format!("{:?}", asset.kind))
            .bind(asset.properties.get("metadata_location").unwrap_or(&asset.location))
            .bind(serde_json::to_string(&asset.properties)?)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<Option<Asset>> {
        let namespace_path = serde_json::to_string(&namespace)?;
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        let row = sqlx::query("SELECT id, name, asset_type, metadata_location, properties FROM assets WHERE tenant_id = ? AND catalog_name = ? AND namespace_path = ? AND name = ? AND branch_name = ?")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(&namespace_path)
            .bind(&name)
            .bind(&branch_name)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let asset_type_str: String = row.get("asset_type");
            let kind = match asset_type_str.as_str() {
                "IcebergTable" => AssetType::IcebergTable,
                "View" => AssetType::View,
                _ => AssetType::IcebergTable,
            };
            
            Ok(Some(Asset {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                name: row.get("name"),
                kind,
                location: row.get::<Option<String>, _>("metadata_location").unwrap_or_default(),
                properties: serde_json::from_str(&row.get::<String, _>("properties")).unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_asset_by_id(&self, tenant_id: Uuid, asset_id: Uuid) -> Result<Option<(Asset, String, Vec<String>)>> {
        let row = sqlx::query("SELECT id, name, catalog_name, namespace_path, asset_type, metadata_location, properties FROM assets WHERE tenant_id = ? AND id = ?")
            .bind(tenant_id.to_string())
            .bind(asset_id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let catalog_name: String = row.get("catalog_name");
            let namespace_json: String = row.get("namespace_path");
            let namespace_path: Vec<String> = serde_json::from_str(&namespace_json).unwrap_or_default();
            
            let asset_type_str: String = row.get("asset_type");
            let kind = match asset_type_str.as_str() {
                "IcebergTable" => AssetType::IcebergTable,
                "View" => AssetType::View,
                _ => AssetType::IcebergTable,
            };
            
            let asset = Asset {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                name: row.get("name"),
                kind,
                location: row.get::<Option<String>, _>("metadata_location").unwrap_or_default(),
                properties: serde_json::from_str(&row.get::<String, _>("properties")).unwrap_or_default(),
            };
            
            Ok(Some((asset, catalog_name, namespace_path)))
        } else {
            Ok(None)
        }
    }

    pub async fn list_assets(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>) -> Result<Vec<Asset>> {
        let namespace_path = serde_json::to_string(&namespace)?;
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        let rows = sqlx::query("SELECT id, name, asset_type, metadata_location, properties FROM assets WHERE tenant_id = ? AND catalog_name = ? AND namespace_path = ? AND branch_name = ?")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(&namespace_path)
            .bind(&branch_name)
            .fetch_all(&self.pool)
            .await?;

        let mut assets = Vec::new();
        for row in rows {
            let asset_type_str: String = row.get("asset_type");
            let kind = match asset_type_str.as_str() {
                "IcebergTable" => AssetType::IcebergTable,
                "View" => AssetType::View,
                _ => AssetType::IcebergTable,
            };
            
            assets.push(Asset {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                name: row.get("name"),
                kind,
                location: row.get::<Option<String>, _>("metadata_location").unwrap_or_default(),
                properties: serde_json::from_str(&row.get::<String, _>("properties")).unwrap_or_default(),
            });
        }
        Ok(assets)
    }

    pub async fn delete_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<()> {
        let namespace_path = serde_json::to_string(&namespace)?;
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        sqlx::query("DELETE FROM assets WHERE tenant_id = ? AND catalog_name = ? AND namespace_path = ? AND name = ? AND branch_name = ?")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(&namespace_path)
            .bind(&name)
            .bind(&branch_name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn rename_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, source_namespace: Vec<String>, source_name: String, dest_namespace: Vec<String>, dest_name: String) -> Result<()> {
        let source_ns_path = serde_json::to_string(&source_namespace)?;
        let dest_ns_path = serde_json::to_string(&dest_namespace)?;
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        
        sqlx::query("UPDATE assets SET namespace_path = ?, name = ? WHERE tenant_id = ? AND catalog_name = ? AND namespace_path = ? AND name = ? AND branch_name = ?")
            .bind(&dest_ns_path)
            .bind(&dest_name)
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(&source_ns_path)
            .bind(&source_name)
            .bind(&branch_name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn count_namespaces(&self, tenant_id: Uuid) -> Result<usize> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM namespaces WHERE tenant_id = ?")
            .bind(tenant_id.to_string())
            .fetch_one(&self.pool)
            .await?;
        Ok(count as usize)
    }

    pub async fn count_assets(&self, tenant_id: Uuid) -> Result<usize> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM assets WHERE tenant_id = ?")
            .bind(tenant_id.to_string())
            .fetch_one(&self.pool)
            .await?;
        Ok(count as usize)
    }

    pub async fn get_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, _branch: Option<String>, namespace: Vec<String>, table: String) -> Result<Option<String>> {
        let namespace_path = serde_json::to_string(&namespace)?;
        let row = sqlx::query("SELECT metadata_location, properties FROM assets WHERE tenant_id = ? AND catalog_name = ? AND namespace_path = ? AND name = ?")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(&namespace_path)
            .bind(&table)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let props_str: String = row.get("properties");
            let props: std::collections::HashMap<String, String> = serde_json::from_str(&props_str).unwrap_or_default();
            // Check property first, then column
            let loc = props.get("metadata_location").cloned().or_else(|| row.get::<Option<String>, _>("metadata_location"));
            Ok(loc)
        } else {
            Ok(None)
        }
    }

    pub async fn update_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String, expected_location: Option<String>, new_location: String) -> Result<()> {
        let namespace_path = serde_json::to_string(&namespace)?;
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        
        // Transaction for CAS
        let mut tx = self.pool.begin().await?;
        
        let row = sqlx::query("SELECT metadata_location, properties FROM assets WHERE tenant_id = ? AND catalog_name = ? AND namespace_path = ? AND name = ? AND branch_name = ?")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(&namespace_path)
            .bind(&table)
            .bind(&branch_name)
            .fetch_optional(&mut *tx)
            .await?;
            
        if let Some(row) = row {
            let props_str: String = row.get("properties");
            let mut props: std::collections::HashMap<String, String> = serde_json::from_str(&props_str).unwrap_or_default();
            
            let column_loc: Option<String> = row.get("metadata_location");
            let current_loc = props.get("metadata_location").cloned().or(column_loc);
            
            if current_loc != expected_location {
                return Err(anyhow::anyhow!("CAS failure: expected {:?} but found {:?}", expected_location, current_loc));
            }
            
            props.insert("metadata_location".to_string(), new_location);
            
            let update_result = sqlx::query("UPDATE assets SET properties = ? WHERE tenant_id = ? AND catalog_name = ? AND namespace_path = ? AND name = ?")
                .bind(serde_json::to_string(&props)?)
                .bind(tenant_id.to_string())
                .bind(catalog_name)
                .bind(&namespace_path)
                .bind(&table)
                .execute(&mut *tx)
                .await?;
                
            if update_result.rows_affected() == 0 {
                return Err(anyhow::anyhow!("Failed to update asset properties"));
            }
            
            tx.commit().await?;
            Ok(())
        } else {
             Err(anyhow::anyhow!("Table not found"))
        }
    }
}
