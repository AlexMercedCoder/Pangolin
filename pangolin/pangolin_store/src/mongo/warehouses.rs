use super::MongoStore;
use super::main::to_bson_uuid;
use anyhow::Result;
use mongodb::bson::{doc};
use pangolin_core::model::{Warehouse, WarehouseUpdate};
use uuid::Uuid;
use futures::stream::TryStreamExt;

impl MongoStore {
    pub async fn create_warehouse(&self, _tenant_id: Uuid, warehouse: Warehouse) -> Result<()> {
        self.warehouses().insert_one(warehouse).await?;
        Ok(())
    }

    pub async fn get_warehouse(&self, tenant_id: Uuid, name: String) -> Result<Option<Warehouse>> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id), "name": name };
        let warehouse = self.warehouses().find_one(filter).await?;
        Ok(warehouse)
    }

    pub async fn list_warehouses(&self, tenant_id: Uuid) -> Result<Vec<Warehouse>> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id) };
        let cursor = self.warehouses().find(filter).await?;
        let warehouses: Vec<Warehouse> = cursor.try_collect().await?;
        Ok(warehouses)
    }

    pub async fn update_warehouse(&self, tenant_id: Uuid, name: String, updates: WarehouseUpdate) -> Result<Warehouse> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id), "name": &name };
        let mut update_doc = doc! {};
        
        if let Some(new_name) = &updates.name {
            update_doc.insert("name", new_name);
        }
        if let Some(config) = &updates.storage_config {
            update_doc.insert("storage_config", mongodb::bson::to_bson(config)?);
        }
        if let Some(use_sts) = updates.use_sts {
            update_doc.insert("use_sts", use_sts);
        }
        if let Some(vending_strategy) = updates.vending_strategy {
            update_doc.insert("vending_strategy", mongodb::bson::to_bson(&vending_strategy)?);
        }
        
        if update_doc.is_empty() {
            return self.get_warehouse(tenant_id, name).await?
                .ok_or_else(|| anyhow::anyhow!("Warehouse not found"));
        }
        
        let update = doc! { "$set": update_doc };
        self.warehouses().update_one(filter, update).await?;
        
        let new_name = updates.name.unwrap_or(name);
        self.get_warehouse(tenant_id, new_name).await?
            .ok_or_else(|| anyhow::anyhow!("Warehouse not found"))
    }

    pub async fn delete_warehouse(&self, tenant_id: Uuid, name: String) -> Result<()> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id), "name": &name };
        let result = self.warehouses().delete_one(filter).await?;
        
        if result.deleted_count == 0 {
            return Err(anyhow::anyhow!("Warehouse '{}' not found", name));
        }
        Ok(())
    }
}
