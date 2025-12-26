use super::MongoStore;
use super::main::to_bson_uuid;
use anyhow::Result;
use mongodb::bson::{doc};
use pangolin_core::model::SystemSettings;
use uuid::Uuid;

impl MongoStore {
    pub async fn get_system_settings(&self, tenant_id: Uuid) -> Result<SystemSettings> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id) };
        let doc = self.system_settings().find_one(filter).await?;
        
        if let Some(d) = doc {
            Ok(mongodb::bson::from_bson(d.get("settings").unwrap().clone())?)
        } else {
            Ok(SystemSettings {
                allow_public_signup: None,
                default_warehouse_bucket: None,
                default_retention_days: None,
                smtp_host: None,
                smtp_port: None,
                smtp_user: None,
                smtp_password: None,
            })
        }
    }

    pub async fn update_system_settings(&self, tenant_id: Uuid, settings: SystemSettings) -> Result<SystemSettings> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id) };
        let update = doc! {
            "$set": {
                "settings": mongodb::bson::to_bson(&settings)?
            }
        };
        
        let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
        self.system_settings().update_one(filter, update).with_options(options).await?;
        Ok(settings)
    }
}
