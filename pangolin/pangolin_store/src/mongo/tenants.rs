use super::MongoStore;
use super::main::to_bson_uuid;
use anyhow::Result;
use mongodb::bson::{doc};
use pangolin_core::model::{Tenant, TenantUpdate};
use uuid::Uuid;
use futures::stream::TryStreamExt;

impl MongoStore {
    pub async fn create_tenant_internal(&self, tenant: Tenant) -> Result<()> {
        tracing::info!("DEBUG_MONGO: create_tenant_internal: {:?}", tenant.id);
        match self.tenants().insert_one(tenant).await {
            Ok(_) => {
                tracing::info!("DEBUG_MONGO: Tenant created successfully");
                Ok(())
            },
            Err(e) => {
                tracing::error!("DEBUG_MONGO: Failed to create tenant: {:?}", e);
                Err(anyhow::anyhow!("Mongo Error: {:?}", e))
            }
        }
    }

    pub async fn get_tenant(&self, id: Uuid) -> Result<Option<Tenant>> {
        let filter = doc! { "id": to_bson_uuid(id) };
        let tenant = self.tenants().find_one(filter).await?;
        Ok(tenant)
    }

    pub async fn list_tenants(&self) -> Result<Vec<Tenant>> {
        let cursor = self.tenants().find(doc! {}).await?;
        let tenants: Vec<Tenant> = cursor.try_collect().await?;
        Ok(tenants)
    }

    pub async fn update_tenant(&self, tenant_id: Uuid, updates: TenantUpdate) -> Result<Tenant> {
        let filter = doc! { "id": to_bson_uuid(tenant_id) };
        let mut update_doc = doc! {};
        
        if let Some(name) = updates.name {
            update_doc.insert("name", name);
        }
        if let Some(properties) = updates.properties {
            update_doc.insert("properties", mongodb::bson::to_bson(&properties)?);
        }
        
        if update_doc.is_empty() {
            return self.get_tenant(tenant_id).await?
                .ok_or_else(|| anyhow::anyhow!("Tenant not found"));
        }
        
        let update = doc! { "$set": update_doc };
        self.tenants().update_one(filter.clone(), update).await?;
        
        self.get_tenant(tenant_id).await?
            .ok_or_else(|| anyhow::anyhow!("Tenant not found"))
    }

    pub async fn delete_tenant(&self, tenant_id: Uuid) -> Result<()> {
        let filter = doc! { "id": to_bson_uuid(tenant_id) };
        let result = self.tenants().delete_one(filter).await?;
        
        if result.deleted_count == 0 {
            return Err(anyhow::anyhow!("Tenant not found"));
        }
        Ok(())
    }
}
