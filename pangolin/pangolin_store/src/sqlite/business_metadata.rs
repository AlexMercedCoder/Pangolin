/// Business Metadata and Search operations for SqliteStore
use super::SqliteStore;
use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;
use chrono::{DateTime, Utc, TimeZone};
use pangolin_core::model::{Asset, AssetType, Catalog, CatalogType, Namespace, Branch, BranchType};
use pangolin_core::business_metadata::BusinessMetadata;
use std::collections::HashMap;

impl SqliteStore {
    pub async fn upsert_business_metadata(&self, metadata: BusinessMetadata) -> Result<()> {
        sqlx::query(
            "INSERT INTO business_metadata (id, asset_id, description, tags, properties, discoverable, created_by, created_at, updated_by, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(asset_id) DO UPDATE SET
             description=excluded.description, tags=excluded.tags, properties=excluded.properties, discoverable=excluded.discoverable, updated_by=excluded.updated_by, updated_at=excluded.updated_at"
        )
        .bind(metadata.id.to_string())
        .bind(metadata.asset_id.to_string())
        .bind(metadata.description)
        .bind(serde_json::to_string(&metadata.tags)?)
        .bind(serde_json::to_string(&metadata.properties)?)
        .bind(if metadata.discoverable { 1 } else { 0 })
        .bind(metadata.created_by.to_string())
        .bind(metadata.created_at.timestamp_millis())
        .bind(metadata.updated_by.to_string())
        .bind(metadata.updated_at.timestamp_millis())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_business_metadata(&self, asset_id: Uuid) -> Result<Option<BusinessMetadata>> {
        let row = sqlx::query("SELECT * FROM business_metadata WHERE asset_id = ?")
            .bind(asset_id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
             Ok(Some(BusinessMetadata {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                asset_id: Uuid::parse_str(&row.get::<String, _>("asset_id"))?,
                description: row.get("description"),
                tags: serde_json::from_str(&row.get::<String, _>("tags"))?,
                properties: serde_json::from_str(&row.get::<String, _>("properties"))?,
                discoverable: row.get::<i32, _>("discoverable") != 0,
                created_by: Uuid::parse_str(&row.get::<String, _>("created_by"))?,
                created_at: Utc.timestamp_millis_opt(row.get("created_at")).single().unwrap_or_default(),
                updated_by: Uuid::parse_str(&row.get::<String, _>("updated_by"))?,
                updated_at: Utc.timestamp_millis_opt(row.get("updated_at")).single().unwrap_or_default(),
             }))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_business_metadata(&self, asset_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM business_metadata WHERE asset_id = ?")
            .bind(asset_id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn search_assets(&self, tenant_id: Uuid, query: &str, tags: Option<Vec<String>>) -> Result<Vec<(Asset, Option<BusinessMetadata>, String, Vec<String>)>> {
        let mut sql = String::from(
            "SELECT 
                a.id, a.tenant_id, a.catalog_name, a.namespace_path, a.name, a.metadata_location, a.properties as asset_properties, a.branch_name, a.asset_type,
                m.id as meta_id, m.description, m.tags, m.properties as meta_properties, m.discoverable, m.created_by as meta_created_by, 
                m.created_at as meta_created_at, m.updated_by as meta_updated_by, m.updated_at as meta_updated_at
            FROM assets a
            LEFT JOIN business_metadata m ON a.id = m.asset_id
            WHERE a.tenant_id = ? AND (a.name LIKE ? OR m.description LIKE ?)"
        );

        let query_pattern = format!("%{}%", query);
        
        if let Some(ref tag_list) = tags {
            if !tag_list.is_empty() {
                sql.push_str(" AND EXISTS (SELECT 1 FROM json_each(m.tags) WHERE value IN (");
                for (i, _) in tag_list.iter().enumerate() {
                    if i > 0 { sql.push_str(", "); }
                    sql.push_str("?");
                }
                sql.push_str("))");
            }
        }

        let mut query_builder = sqlx::query(&sql)
            .bind(tenant_id.to_string())
            .bind(&query_pattern)
            .bind(&query_pattern);

        if let Some(ref tag_list) = tags {
            if !tag_list.is_empty() {
                for tag in tag_list {
                    query_builder = query_builder.bind(tag);
                }
            }
        }

        let rows = query_builder.fetch_all(&self.pool).await?;
        let mut results = Vec::new();

        for row in rows {
            let asset_type_str: String = row.get("asset_type");
            let kind = match asset_type_str.as_str() {
                "IcebergTable" => AssetType::IcebergTable,
                "View" => AssetType::View,
                _ => AssetType::IcebergTable,
            };

            let asset = Asset {
                id: Uuid::parse_str(row.get("id"))?,
                name: row.get("name"),
                kind,
                location: row.get::<Option<String>, _>("metadata_location").unwrap_or_default(),
                properties: serde_json::from_str(row.get("asset_properties")).unwrap_or_default(),
            };

            let meta_id_str: Option<String> = row.get("meta_id");
            let metadata = if let Some(id_str) = meta_id_str {
                Some(BusinessMetadata {
                    id: Uuid::parse_str(&id_str)?,
                    asset_id: asset.id,
                    description: row.get("description"),
                    tags: serde_json::from_str(row.get("tags")).unwrap_or_default(),
                    properties: serde_json::from_str(row.get("meta_properties")).unwrap_or_default(),
                    discoverable: row.get::<i32, _>("discoverable") != 0,
                    created_by: Uuid::parse_str(row.get("meta_created_by"))?,
                    created_at: Utc.timestamp_nanos(row.get("meta_created_at")),
                    updated_by: Uuid::parse_str(row.get("meta_updated_by"))?,
                    updated_at: Utc.timestamp_nanos(row.get("meta_updated_at")),
                })
            } else {
                None
            };
            
            let catalog_name: String = row.get("catalog_name");
            let namespace_path: String = row.get("namespace_path");
            let namespace: Vec<String> = namespace_path.split('\x1F').map(String::from).collect();

            results.push((asset, metadata, catalog_name, namespace));
        }

        Ok(results)
    }

    pub async fn search_catalogs(&self, tenant_id: Uuid, query: &str) -> Result<Vec<Catalog>> {
        let query_pattern = format!("%{}%", query);
        let rows = sqlx::query("SELECT id, name, catalog_type, warehouse_name, storage_location, federated_config, properties FROM catalogs WHERE tenant_id = ? AND name LIKE ?")
            .bind(tenant_id.to_string())
            .bind(&query_pattern)
            .fetch_all(&self.pool)
            .await?;

        let mut catalogs = Vec::new();
        for row in rows {
            catalogs.push(Catalog {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                name: row.get("name"),
                catalog_type: serde_json::from_str(&format!("\"{}\"", row.get::<String, _>("catalog_type"))).unwrap_or(CatalogType::Local),
                warehouse_name: row.get("warehouse_name"),
                storage_location: row.get("storage_location"),
                federated_config: row.get::<Option<String>, _>("federated_config").and_then(|s| serde_json::from_str(&s).ok()),
                properties: serde_json::from_str(&row.get::<String, _>("properties")).unwrap_or_default(),
            });
        }
        Ok(catalogs)
    }

    pub async fn search_namespaces(&self, tenant_id: Uuid, query: &str) -> Result<Vec<(Namespace, String)>> {
        let query_pattern = format!("%{}%", query);
        let rows = sqlx::query("SELECT catalog_name, namespace_path, properties FROM namespaces WHERE tenant_id = ? AND namespace_path LIKE ?")
            .bind(tenant_id.to_string())
            .bind(&query_pattern)
            .fetch_all(&self.pool)
            .await?;

        let mut namespaces = Vec::new();
        for row in rows {
            let catalog_name: String = row.get("catalog_name");
            let namespace_json: String = row.get("namespace_path");
            let namespace_path: Vec<String> = serde_json::from_str(&namespace_json).unwrap_or_default();
            
            namespaces.push((Namespace {
                name: namespace_path,
                properties: serde_json::from_str(&row.get::<String, _>("properties")).unwrap_or_default(),
            }, catalog_name));
        }
        Ok(namespaces)
    }

    pub async fn search_branches(&self, tenant_id: Uuid, query: &str) -> Result<Vec<(Branch, String)>> {
        let query_pattern = format!("%{}%", query);
        let rows = sqlx::query("SELECT catalog_name, name, head_commit_id, branch_type, assets FROM branches WHERE tenant_id = ? AND name LIKE ?")
            .bind(tenant_id.to_string())
            .bind(&query_pattern)
            .fetch_all(&self.pool)
            .await?;

        let mut branches = Vec::new();
        for row in rows {
            let catalog_name: String = row.get("catalog_name");
            let branch_type_str: String = row.get("branch_type");
            let branch_type = match branch_type_str.as_str() {
                "Ingest" => BranchType::Ingest,
                _ => BranchType::Experimental,
            };

            branches.push((Branch {
                name: row.get("name"),
                head_commit_id: row.get::<Option<String>, _>("head_commit_id").map(|s| Uuid::parse_str(&s)).transpose()?,
                branch_type,
                assets: serde_json::from_str(&row.get::<String, _>("assets")).unwrap_or_default(),
            }, catalog_name));
        }
        Ok(branches)
    }
}
