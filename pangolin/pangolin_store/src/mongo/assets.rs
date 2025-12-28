use super::MongoStore;
use super::main::{to_bson_uuid, from_bson_uuid};
use anyhow::Result;
use mongodb::bson::{doc, Document};
use mongodb::options::ReplaceOptions;
use pangolin_core::model::{Asset, AssetType};
use uuid::Uuid;
use futures::stream::TryStreamExt;
use std::collections::HashMap;

impl MongoStore {
    pub async fn create_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, asset: Asset) -> Result<()> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "branch": &branch_name,
            "namespace": &namespace,
            "name": &asset.name
        };
        
        let doc = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "branch": &branch_name,
            "namespace": namespace,
            "id": to_bson_uuid(asset.id),
            "name": &asset.name,
            "kind": format!("{:?}", asset.kind),
            "location": &asset.location,
            "properties": mongodb::bson::to_bson(&asset.properties)?
        };
        
        let options = ReplaceOptions::builder().upsert(true).build();
        self.db.collection::<Document>("assets").replace_one(filter, doc).with_options(options).await?;
        Ok(())
    }

    pub async fn get_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<Option<Asset>> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "branch": branch_name,
            "namespace": namespace,
            "name": name
        };
        
        let doc = self.db.collection::<Document>("assets").find_one(filter).await?;
        
        if let Some(d) = doc {
            let kind_str = d.get_str("kind")?;
            let kind = match kind_str {
                "IcebergTable" => AssetType::IcebergTable,
                "View" => AssetType::View,
                _ => AssetType::IcebergTable,
            };
            
            let properties: HashMap<String, String> = mongodb::bson::from_bson(d.get("properties").unwrap().clone())?;
            let id_bson = d.get("id").ok_or(anyhow::anyhow!("Missing id"))?;
            let id = from_bson_uuid(id_bson)?;

            Ok(Some(Asset {
                id,
                name: d.get_str("name")?.to_string(),
                kind,
                location: d.get_str("location").unwrap_or("").to_string(),
                properties,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_assets(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, pagination: Option<crate::PaginationParams>) -> Result<Vec<Asset>> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "branch": branch_name,
            "namespace": namespace
        };

        let collection = self.db.collection::<Document>("assets");
        let mut find = collection.find(filter);
        if let Some(p) = pagination {
            if let Some(l) = p.limit {
                find = find.limit(l as i64);
            }
            if let Some(o) = p.offset {
                find = find.skip(o as u64);
            }
        }

        let cursor = find.await?;
        let docs: Vec<Document> = cursor.try_collect().await?;
        
        let mut assets = Vec::new();
        for d in docs {
            let kind_str = d.get_str("kind")?;
            let kind = match kind_str {
                "IcebergTable" => AssetType::IcebergTable,
                "View" => AssetType::View,
                _ => AssetType::IcebergTable,
            };
            
            let properties: HashMap<String, String> = mongodb::bson::from_bson(d.get("properties").unwrap().clone())?;
            let id = if let Ok(i) = d.get("id").ok_or(anyhow::anyhow!("Missing id")).and_then(|b| from_bson_uuid(b)) {
                i
            } else {
                Uuid::new_v4()
            };

            assets.push(Asset {
                id,
                name: d.get_str("name")?.to_string(),
                kind,
                location: d.get_str("location").unwrap_or("").to_string(),
                properties,
            });
        }
        Ok(assets)
    }

    pub async fn get_asset_by_id(&self, tenant_id: Uuid, asset_id: Uuid) -> Result<Option<(Asset, String, Vec<String>)>> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "id": to_bson_uuid(asset_id)
        };
        let d = self.db.collection::<Document>("assets").find_one(filter).await?;
        
        if let Some(d) = d {
            let catalog_name = d.get_str("catalog_name")?.to_string();
            let namespace = d.get_array("namespace")?.iter().map(|v| v.as_str().unwrap().to_string()).collect(); 
            let kind_str = d.get_str("kind")?;
            let kind = match kind_str {
                "IcebergTable" => AssetType::IcebergTable,
                "View" => AssetType::View,
                _ => AssetType::IcebergTable,
            };
            
            let properties: HashMap<String, String> = mongodb::bson::from_bson(d.get("properties").unwrap().clone())?;

            let asset = Asset {
                id: asset_id,
                name: d.get_str("name")?.to_string(),
                kind,
                location: d.get_str("location").unwrap_or("").to_string(),
                properties,
            };
            
            Ok(Some((asset, catalog_name, namespace)))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<()> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "branch": branch_name,
            "namespace": namespace,
            "name": name
        };
        self.db.collection::<Document>("assets").delete_one(filter).await?;
        Ok(())
    }

    pub async fn rename_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, source_namespace: Vec<String>, source_name: String, dest_namespace: Vec<String>, dest_name: String) -> Result<()> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "branch": &branch_name,
            "namespace": source_namespace,
            "name": source_name
        };
        
        let update = doc! {
            "$set": {
                "namespace": dest_namespace,
                "name": dest_name
            }
        };
        
        self.db.collection::<Document>("assets").update_one(filter, update).await?;
        Ok(())
    }

    pub async fn count_assets(&self, tenant_id: Uuid) -> Result<usize> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id) };
        let count = self.assets().count_documents(filter).await?;
        Ok(count as usize)
    }

    pub async fn copy_assets_bulk(&self, tenant_id: Uuid, catalog_name: &str, src_branch: &str, dest_branch: &str, namespace_filter: Option<String>) -> Result<usize> {
        let mut match_stage = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "branch": src_branch
        };

        if let Some(ns_str) = namespace_filter {
            let ns_vec: Vec<String> = ns_str.split('.').map(|s| s.to_string()).collect();
            match_stage.insert("namespace", ns_vec);
        }

        let count_filter = match_stage.clone(); // Clone before move


        // Mongo's $merge doesn't return count directly, so we run match count first if fine, 
        // OR we can just rely on a second query to count dest assets.
        // Actually, $merge output is not standard cursor.
        // Let's optimize: Count source first for return value, then Execute.
        
        let count = self.assets().count_documents(count_filter.clone()).await?;
        
        // However, generating new UUIDs via $function (JS) requires server-side JS enabled, which might be restricted.
        // Alternative: Fetch then BulkWrite.
        // Let's use the Fetch-BulkWrite approach for better compatibility and UUID control in Rust layer 
        // until we are sure about Mongo JS support.
        // It is still O(1) round trips if we use bulk_write but O(N) data transfer.
        // Wait, the goal is O(1) DB interaction.
        // For Mongo, true bulk copy inside DB without JS is hard for UUID generation.
        // Let's fall back to Fetch-BulkWrite for Safety/Compatibility unless we are sure.
        // Actually, let's implement the Fetch -> Bulk Insert. It's efficient enough for typical loads compared to API loop.
        
        // Attempting Fetch-BulkWrite Implementation:
        let src_docs: Vec<Document> = self.db.collection::<Document>("assets").find(count_filter).await?.try_collect().await?;
        if src_docs.is_empty() {
             return Ok(0);
        }

        let mut new_docs = Vec::new();
        for mut d in src_docs {
            d.remove("_id"); // Let Mongo generate new ObjectId
            d.insert("branch", dest_branch);
            d.insert("id", to_bson_uuid(Uuid::new_v4())); // New UUID
            new_docs.push(d);
        }
        
        if !new_docs.is_empty() {
            self.db.collection::<Document>("assets").insert_many(new_docs).await?;
        }
        
        Ok(count as usize)
    }
}
