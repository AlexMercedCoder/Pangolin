use super::MongoStore;
use super::main::to_bson_uuid;
use anyhow::Result;
use mongodb::bson::{doc};
use pangolin_core::model::SyncStats;
use uuid::Uuid;

impl MongoStore {
    pub async fn get_sync_stats(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Option<SyncStats>> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name
        };
        let doc = self.federated_sync_stats().find_one(filter).await?;
        
        if let Some(d) = doc {
            Ok(Some(mongodb::bson::from_bson(d.get("stats").unwrap().clone())?))
        } else {
            Ok(None)
        }
    }

    pub async fn update_sync_stats(&self, tenant_id: Uuid, catalog_name: &str, stats: SyncStats) -> Result<()> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name
        };
        let update = doc! {
            "$set": {
                "stats": mongodb::bson::to_bson(&stats)?
            }
        };
        
        let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
        self.federated_sync_stats().update_one(filter, update).with_options(options).await?;
        Ok(())
    }
}
