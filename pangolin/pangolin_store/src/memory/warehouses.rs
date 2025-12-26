use super::MemoryStore;
use anyhow::Result;
use uuid::Uuid;
use pangolin_core::model::*;
use async_trait::async_trait;

impl MemoryStore {
    pub(crate) async fn create_warehouse_internal(&self, tenant_id: Uuid, warehouse: Warehouse) -> Result<()> {
            let key = (tenant_id, warehouse.name.clone());
            self.warehouses.insert(key, warehouse);
            Ok(())
        }
    pub(crate) async fn get_warehouse_internal(&self, tenant_id: Uuid, name: String) -> Result<Option<Warehouse>> {
            let key = (tenant_id, name);
            if let Some(w) = self.warehouses.get(&key) {
                Ok(Some(w.value().clone()))
            } else {
                Ok(None)
            }
        }
    pub(crate) async fn list_warehouses_internal(&self, tenant_id: Uuid) -> Result<Vec<Warehouse>> {
            let warehouses = self.warehouses.iter()
                .filter(|r| r.key().0 == tenant_id)
                .map(|r| r.value().clone())
                .collect();
            Ok(warehouses)
        }
    pub(crate) async fn update_warehouse_internal(&self, tenant_id: Uuid, name: String, updates: pangolin_core::model::WarehouseUpdate) -> Result<Warehouse> {
            let key = (tenant_id, name.clone());
            if let Some(mut warehouse) = self.warehouses.get_mut(&key) {
                if let Some(new_name) = updates.name {
                    // If name is changing, we need to remove old key and insert with new key
                    let mut w = warehouse.clone();
                    w.name = new_name.clone();
                    drop(warehouse); // Release the mutable reference
                    self.warehouses.remove(&key);
                    let new_key = (tenant_id, new_name);
                    self.warehouses.insert(new_key, w.clone());
                    return Ok(w);
                }
                if let Some(config) = updates.storage_config {
                    warehouse.storage_config.extend(config);
                }
                if let Some(use_sts) = updates.use_sts {
                    warehouse.use_sts = use_sts;
                }
                Ok(warehouse.clone())
            } else {
                Err(anyhow::anyhow!("Warehouse '{}' not found", name))
            }
        }
    pub(crate) async fn delete_warehouse_internal(&self, tenant_id: Uuid, name: String) -> Result<()> {
            let key = (tenant_id, name.clone());
        
            if self.warehouses.remove(&key).is_some() {
                Ok(())
            } else {
                Err(anyhow::anyhow!("Warehouse '{}' not found", name))
            }
        }
}
