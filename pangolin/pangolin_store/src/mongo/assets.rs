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

    pub async fn list_assets(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>) -> Result<Vec<Asset>> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "branch": branch_name,
            "namespace": namespace
        };
        let cursor = self.db.collection::<Document>("assets").find(filter).await?;
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
}
