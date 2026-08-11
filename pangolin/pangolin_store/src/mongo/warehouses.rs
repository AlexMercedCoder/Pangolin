use super::main::to_bson_uuid;
use super::MongoStore;
use crate::secrets;
use anyhow::Result;
use futures::stream::TryStreamExt;
use mongodb::bson::doc;
use pangolin_core::model::{Warehouse, WarehouseUpdate};
use uuid::Uuid;

impl MongoStore {
    pub async fn create_warehouse(&self, _tenant_id: Uuid, mut warehouse: Warehouse) -> Result<()> {
        // C-11: see the module note in `crate::secrets`.
        secrets::seal(&mut warehouse.storage_config)?;
        self.warehouses().insert_one(warehouse).await?;
        Ok(())
    }

    pub async fn get_warehouse(&self, tenant_id: Uuid, name: String) -> Result<Option<Warehouse>> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id), "name": name };
        let warehouse = self.warehouses().find_one(filter).await?;
        match warehouse {
            Some(mut w) => {
                secrets::open(&mut w.storage_config)?;
                Ok(Some(w))
            }
            None => Ok(None),
        }
    }

    pub async fn list_warehouses(
        &self,
        tenant_id: Uuid,
        pagination: Option<crate::PaginationParams>,
    ) -> Result<Vec<Warehouse>> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id) };

        let collection = self.warehouses();
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
        let mut warehouses: Vec<Warehouse> = cursor.try_collect().await?;
        for warehouse in &mut warehouses {
            secrets::open(&mut warehouse.storage_config)?;
        }
        Ok(warehouses)
    }

    pub async fn update_warehouse(
        &self,
        tenant_id: Uuid,
        name: String,
        updates: WarehouseUpdate,
    ) -> Result<Warehouse> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id), "name": &name };
        let mut update_doc = doc! {};

        if let Some(new_name) = &updates.name {
            update_doc.insert("name", new_name);
        }
        if let Some(config) = &updates.storage_config {
            let mut sealed = config.clone();
            secrets::seal(&mut sealed)?;
            update_doc.insert("storage_config", mongodb::bson::to_bson(&sealed)?);
        }
        if let Some(use_sts) = updates.use_sts {
            update_doc.insert("use_sts", use_sts);
        }
        if let Some(vending_strategy) = updates.vending_strategy {
            update_doc.insert(
                "vending_strategy",
                mongodb::bson::to_bson(&vending_strategy)?,
            );
        }

        if update_doc.is_empty() {
            return self
                .get_warehouse(tenant_id, name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Warehouse not found"));
        }

        let update = doc! { "$set": update_doc };
        self.warehouses().update_one(filter, update).await?;

        let new_name = updates.name.unwrap_or(name);
        self.get_warehouse(tenant_id, new_name)
            .await?
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
