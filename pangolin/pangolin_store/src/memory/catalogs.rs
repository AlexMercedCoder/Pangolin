use super::MemoryStore;
use anyhow::Result;
use uuid::Uuid;
use pangolin_core::model::*;
use async_trait::async_trait;

impl MemoryStore {
    pub(crate) async fn create_catalog_internal(&self, tenant_id: Uuid, catalog: Catalog) -> Result<()> {
            let key = (tenant_id, catalog.name.clone());
            self.catalogs.insert(key, catalog);
            Ok(())
        }
    pub(crate) async fn get_catalog_internal(&self, tenant_id: Uuid, name: String) -> Result<Option<Catalog>> {
            let key = (tenant_id, name);
            if let Some(c) = self.catalogs.get(&key) {
                Ok(Some(c.value().clone()))
            } else {
                Ok(None)
            }
        }

    pub(crate) async fn search_catalogs_internal(&self, tenant_id: Uuid, query: &str) -> Result<Vec<Catalog>> {
        let query = query.to_lowercase();
        let results = self.catalogs.iter()
            .filter(|entry| {
                entry.key().0 == tenant_id && 
                entry.value().name.to_lowercase().contains(&query)
            })
            .map(|entry| entry.value().clone())
            .collect();
        Ok(results)
    }
    pub(crate) async fn list_catalogs_internal(&self, tenant_id: Uuid, pagination: Option<crate::PaginationParams>) -> Result<Vec<Catalog>> {
            let mut catalogs: Vec<Catalog> = self.catalogs.iter()
                .filter(|entry| entry.key().0 == tenant_id)
                .map(|entry| entry.value().clone())
                .collect();
                
            catalogs.sort_by(|a, b| a.name.cmp(&b.name));
            
            if let Some(p) = pagination {
                let offset = p.offset.unwrap_or(0);
                let limit = p.limit.unwrap_or(usize::MAX);
                
                if offset >= catalogs.len() {
                    return Ok(Vec::new());
                }
                
                let end = std::cmp::min(offset + limit, catalogs.len());
                Ok(catalogs[offset..end].to_vec())
            } else {
                Ok(catalogs)
            }
        }
    pub(crate) async fn update_catalog_internal(&self, tenant_id: Uuid, name: String, updates: pangolin_core::model::CatalogUpdate) -> Result<Catalog> {
            let key = (tenant_id, name.clone());
            if let Some(mut catalog) = self.catalogs.get_mut(&key) {
                if let Some(warehouse_name) = updates.warehouse_name {
                    catalog.warehouse_name = Some(warehouse_name);
                }
                if let Some(storage_location) = updates.storage_location {
                    catalog.storage_location = Some(storage_location);
                }
                if let Some(properties) = updates.properties {
                    catalog.properties.extend(properties);
                }
                Ok(catalog.clone())
            } else {
                Err(anyhow::anyhow!("Catalog '{}' not found", name))
            }
        }
    pub(crate) async fn delete_catalog_internal(&self, tenant_id: Uuid, name: String) -> Result<()> {
            let key = (tenant_id, name.clone());
            if self.catalogs.remove(&key).is_some() {
                // Cascade delete: Remove all associated resources
                // Note: In a real database, this would be handled by foreign keys.
                // In MemoryStore, we must manually iterate and remove.
            
                // Remove Namespaces
                self.namespaces.retain(|k, _| !(k.0 == tenant_id && k.1 == name));
            
                // Remove Assets
                self.assets.retain(|k, _| !(k.0 == tenant_id && k.1 == name));
            
                // Remove Branches
                self.branches.retain(|k, _| !(k.0 == tenant_id && k.1 == name));
            
                // Remove Tags
                self.tags.retain(|k, _| !(k.0 == tenant_id && k.1 == name));

                // Clean up assets_by_id index
                // This is expensive O(N) since we have to scan the whole index
                // But deletion is rare.
                self.assets_by_id.retain(|_, v| v.0 != name);

                Ok(())
            } else {
                Err(anyhow::anyhow!("Catalog not found"))
            }
        }
}
