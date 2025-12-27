use super::MongoStore;
use super::main::to_bson_uuid;
use anyhow::Result;
use mongodb::bson::{doc, Document};
use pangolin_core::model::Branch;
use uuid::Uuid;
use futures::stream::TryStreamExt;

impl MongoStore {
    pub async fn create_branch(&self, tenant_id: Uuid, catalog_name: &str, branch: Branch) -> Result<()> {
        let doc = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "name": &branch.name,
            "head_commit_id": branch.head_commit_id,
            "branch_type": format!("{:?}", branch.branch_type),
            "assets": &branch.assets
        };
        self.db.collection::<Document>("branches").insert_one(doc).await?;
        Ok(())
    }

    pub async fn get_branch(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Branch>> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "name": name
        };
        let doc = self.db.collection::<Document>("branches").find_one(filter).await?;
        
        if let Some(d) = doc {
             let type_str = d.get_str("branch_type")?;
             let branch_type = match type_str {
                 "Ingest" => pangolin_core::model::BranchType::Ingest,
                 "Experimental" => pangolin_core::model::BranchType::Experimental,
                 _ => pangolin_core::model::BranchType::Experimental,
             };
             
             Ok(Some(Branch {
                 name: d.get_str("name")?.to_string(),
                 head_commit_id: mongodb::bson::from_bson(d.get("head_commit_id").unwrap().clone())?,
                 branch_type,
                 assets: mongodb::bson::from_bson(d.get("assets").unwrap().clone())?,
             }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_branches(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<Branch>> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name
        };
        let cursor = self.db.collection::<Document>("branches").find(filter).await?;
        let docs: Vec<Document> = cursor.try_collect().await?;
        
        let mut branches = Vec::new();
        for d in docs {
             let type_str = d.get_str("branch_type")?;
             let branch_type = match type_str {
                 "Ingest" => pangolin_core::model::BranchType::Ingest,
                 "Experimental" => pangolin_core::model::BranchType::Experimental,
                 _ => pangolin_core::model::BranchType::Experimental,
             };
             
             branches.push(Branch {
                 name: d.get_str("name")?.to_string(),
                 head_commit_id: mongodb::bson::from_bson(d.get("head_commit_id").unwrap().clone())?,
                 branch_type,
                 assets: mongodb::bson::from_bson(d.get("assets").unwrap().clone())?,
             });
        }
        Ok(branches)
    }

    pub async fn delete_branch(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<()> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "name": &name
        };
        let result = self.db.collection::<Document>("branches").delete_one(filter).await?;
        if result.deleted_count == 0 {
            return Err(anyhow::anyhow!("Branch '{}' not found", name));
        }
        
        // Also delete assets associated with this branch
        let asset_filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "branch": name
        };
        self.db.collection::<Document>("assets").delete_many(asset_filter).await?;
        
        Ok(())
    }

    pub async fn merge_branch(&self, tenant_id: Uuid, catalog_name: &str, target_branch: String, source_branch: String) -> Result<()> {

        let source = self.get_branch(tenant_id, catalog_name, source_branch.clone()).await?
            .ok_or_else(|| anyhow::anyhow!("Source branch not found"))?;
            
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "name": &target_branch
        };
        
        let update = doc! {
            "$set": {
                "head_commit_id": source.head_commit_id
            }
        };
        
        self.db.collection::<Document>("branches").update_one(filter, update).await?;

        // Sync assets from source to target (Simulating Fast-Forward Merge)
        // Iterate known namespaces to find assets
        let namespaces = self.list_namespaces(tenant_id, catalog_name, None).await?;
        for ns in namespaces {
             let assets = self.list_assets(tenant_id, catalog_name, Some(source_branch.clone()), ns.name.clone()).await?;
             for asset in assets {
                 // Upsert asset into target branch
                 self.create_asset(tenant_id, catalog_name, Some(target_branch.clone()), ns.name.clone(), asset).await?;
             }
        }

        Ok(())
    }
}
