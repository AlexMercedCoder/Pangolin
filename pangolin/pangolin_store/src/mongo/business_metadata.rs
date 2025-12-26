use super::MongoStore;
use super::main::{to_bson_uuid, from_bson_uuid};
use anyhow::Result;
use mongodb::bson::{doc, Document};
use pangolin_core::business_metadata::BusinessMetadata;
use pangolin_core::model::{Asset, AssetType, Catalog, Namespace, Branch};
use uuid::Uuid;
use futures::stream::TryStreamExt;
use std::collections::HashMap;

impl MongoStore {
    pub async fn upsert_business_metadata(&self, metadata: BusinessMetadata) -> Result<()> {
        let filter = doc! { "asset-id": to_bson_uuid(metadata.asset_id) };
        let mut doc = mongodb::bson::to_document(&metadata)?;
        doc.insert("asset-id", to_bson_uuid(metadata.asset_id));
        
        let options = mongodb::options::ReplaceOptions::builder().upsert(true).build();
        self.db.collection::<Document>("business_metadata")
            .replace_one(filter, doc).with_options(options)
            .await?;
        Ok(())
    }

    pub async fn get_business_metadata(&self, asset_id: Uuid) -> Result<Option<BusinessMetadata>> {
        let filter = doc! { "asset-id": to_bson_uuid(asset_id) };
        let meta = self.business_metadata().find_one(filter).await?;
        Ok(meta)
    }

    pub async fn delete_business_metadata(&self, asset_id: Uuid) -> Result<()> {
        let filter = doc! { "asset-id": to_bson_uuid(asset_id) };
        self.business_metadata().delete_one(filter).await?;
        Ok(())
    }

    pub async fn search_assets(&self, tenant_id: Uuid, query: &str, tags: Option<Vec<String>>) -> Result<Vec<(Asset, Option<BusinessMetadata>, String, Vec<String>)>> {
        let query_regex = mongodb::bson::Regex {
             pattern: format!(".*{}.*", regex::escape(query)),
             options: "i".to_string(),
        };

        let mut pipeline = vec![
            doc! { "$match": { "tenant_id": to_bson_uuid(tenant_id) } },
            doc! { 
                "$lookup": {
                    "from": "business_metadata",
                    "localField": "id",
                    "foreignField": "asset-id",
                    "as": "metadata"
                }
            },
            doc! {
                "$unwind": {
                    "path": "$metadata",
                    "preserveNullAndEmptyArrays": true
                }
            },
            doc! {
                "$match": {
                    "$or": [
                        { "name": query_regex.clone() },
                        { "metadata.description": query_regex }
                    ]
                }
            }
        ];

        if let Some(tag_list) = tags {
            if !tag_list.is_empty() {
                pipeline.push(doc! {
                    "$match": {
                        "metadata.tags": { "$all": tag_list }
                    }
                });
            }
        }

        let cursor = self.db.collection::<Document>("assets").aggregate(pipeline).await?;
        let docs: Vec<Document> = cursor.try_collect().await?;
        
        let mut results = Vec::new();
        for d in docs {
            let kind_str = d.get_str("kind")?;
            let kind = match kind_str {
                "IcebergTable" => AssetType::IcebergTable,
                "View" => AssetType::View,
                _ => AssetType::IcebergTable,
            };
            
            let properties: HashMap<String, String> = mongodb::bson::from_bson(d.get("properties").unwrap().clone())?;
            let asset = Asset {
                id: from_bson_uuid(d.get("id").ok_or(anyhow::anyhow!("Missing id"))?)?,
                name: d.get_str("name")?.to_string(),
                kind,
                location: d.get_str("location").unwrap_or("").to_string(),
                properties,
            };
            
            let metadata = if let Ok(m) = d.get_document("metadata") {
                Some(mongodb::bson::from_document(m.clone())?)
            } else {
                None
            };
            
            let catalog_name = d.get_str("catalog_name")?.to_string();
            let namespace = d.get_array("namespace")?.iter().map(|v| v.as_str().unwrap().to_string()).collect();

            results.push((asset, metadata, catalog_name, namespace));
        }
        Ok(results)
    }

    pub async fn search_catalogs(&self, tenant_id: Uuid, query: &str) -> Result<Vec<Catalog>> {
        let query_regex = mongodb::bson::Regex {
             pattern: format!(".*{}.*", regex::escape(query)),
             options: "i".to_string(),
        };
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "name": query_regex
        };
        let cursor = self.catalogs().find(filter).await?;
        let catalogs: Vec<Catalog> = cursor.try_collect().await?;
        Ok(catalogs)
    }

    pub async fn search_namespaces(&self, tenant_id: Uuid, query: &str) -> Result<Vec<(Namespace, String)>> {
        let query_regex = mongodb::bson::Regex {
             pattern: format!(".*{}.*", regex::escape(query)),
             options: "i".to_string(),
        };
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "name": query_regex
        };
        let cursor = self.db.collection::<Document>("namespaces").find(filter).await?;
        let docs: Vec<Document> = cursor.try_collect().await?;
        
        let mut results = Vec::new();
        for d in docs {
            results.push((
                Namespace {
                    name: d.get_array("name")?.iter().map(|v| v.as_str().unwrap().to_string()).collect(),
                    properties: mongodb::bson::from_bson(d.get("properties").unwrap().clone())?,
                },
                d.get_str("catalog_name")?.to_string()
            ));
        }
        Ok(results)
    }

    pub async fn search_branches(&self, tenant_id: Uuid, query: &str) -> Result<Vec<(Branch, String)>> {
        let query_regex = mongodb::bson::Regex {
             pattern: format!(".*{}.*", regex::escape(query)),
             options: "i".to_string(),
        };
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "name": query_regex
        };
        let cursor = self.db.collection::<Document>("branches").find(filter).await?;
        let docs: Vec<Document> = cursor.try_collect().await?;
        
        let mut results = Vec::new();
        for d in docs {
             let type_str = d.get_str("branch_type")?;
             let branch_type = match type_str {
                 "Ingest" => pangolin_core::model::BranchType::Ingest,
                 "Experimental" => pangolin_core::model::BranchType::Experimental,
                 _ => pangolin_core::model::BranchType::Experimental,
             };
             
             results.push((
                 Branch {
                     name: d.get_str("name")?.to_string(),
                     head_commit_id: mongodb::bson::from_bson(d.get("head_commit_id").unwrap().clone())?,
                     branch_type,
                     assets: mongodb::bson::from_bson(d.get("assets").unwrap().clone())?,
                 },
                 d.get_str("catalog_name")?.to_string()
             ));
        }
        Ok(results)
    }
}
