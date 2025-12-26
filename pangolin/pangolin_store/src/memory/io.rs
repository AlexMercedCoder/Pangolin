use super::MemoryStore;
use anyhow::Result;
use uuid::Uuid;
use std::sync::Arc;
use async_trait::async_trait;

impl MemoryStore {
    pub(crate) async fn get_metadata_location_internal(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String) -> Result<Option<String>> {
            let branch_name = branch.unwrap_or_else(|| "main".to_string());
            let key = (tenant_id, catalog_name.to_string(), branch_name, namespace.join("\x1F"), table);
        
            if let Some(asset) = self.assets.get(&key) {
                let loc = asset.properties.get("metadata_location").cloned().unwrap_or(asset.location.clone());
                Ok(Some(loc))
            } else {
                Ok(None)
            }
        }
    pub(crate) async fn update_metadata_location_internal(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String, expected_location: Option<String>, new_location: String) -> Result<()> {
            let branch_name = branch.unwrap_or_else(|| "main".to_string());
            let key = (tenant_id, catalog_name.to_string(), branch_name, namespace.join("\x1F"), table);
        
            if let Some(mut asset) = self.assets.get_mut(&key) {
                let current_loc = asset.properties.get("metadata_location").cloned().unwrap_or(asset.location.clone());
            
                // CAS Check
                if let Some(expected) = expected_location {
                    if current_loc != expected {
                        return Err(anyhow::anyhow!("CAS failure: expected {} but found {}", expected, current_loc));
                    }
                }
            
                asset.location = new_location.clone();
                asset.properties.insert("metadata_location".to_string(), new_location);
                Ok(())
            } else {
                Err(anyhow::anyhow!("Table not found"))
            }
        }
    pub(crate) async fn read_file_internal(&self, location: &str) -> Result<Vec<u8>> {
            // Use metadata cache for metadata.json files
            if location.ends_with("metadata.json") || location.ends_with(".metadata.json") {
                return self.metadata_cache.get_or_fetch(location, || async {
                    self.read_file_uncached_internal(location).await
                }).await;
            }
        
            // Non-metadata files bypass cache
            self.read_file_uncached_internal(location).await
        }
    pub(crate) async fn write_file_internal(&self, location: &str, content: Vec<u8>) -> Result<()> {
            // Invalidate metadata cache on write
            if location.ends_with("metadata.json") || location.ends_with(".metadata.json") {
                self.metadata_cache.invalidate(location).await;
            }
        
            // Dual write: Memory + Object Store
            self.files.insert(location.to_string(), content.clone());
        
            if let Some(warehouse) = self.get_warehouse_for_location(location) {
                 if location.starts_with("s3://") || location.starts_with("az://") || location.starts_with("gs://") {
                     // Use object store cache
                     let cache_key = self.get_object_store_cache_key(&warehouse.storage_config, location);
                     let store = self.object_store_cache.get_or_insert(cache_key, || {
                         Arc::new(crate::object_store_factory::create_object_store(&warehouse.storage_config, location)
                             .expect("Failed to create object store"))
                     });
                 
                     let path = self.extract_object_store_path(location);
                     store.put(&path, content.into()).await?;
                 }
            }
        
            Ok(())
        }
    pub(crate) async fn expire_snapshots_internal(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _namespace: Vec<String>, _table: String, _retention_ms: i64) -> Result<()> {
            tracing::info!("MemoryStore: Expiring snapshots (placeholder)");
            Ok(())
        }
    pub(crate) async fn remove_orphan_files_internal(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _namespace: Vec<String>, _table: String, _older_than_ms: i64) -> Result<()> {
            tracing::info!("MemoryStore: Removing orphan files (placeholder)");
            Ok(())
        }
    pub(crate) async fn read_file_uncached_internal(&self, location: &str) -> Result<Vec<u8>> {
            // Try to read from object store first if configured
            if let Some(warehouse) = self.get_warehouse_for_location(location) {
                 // Basic heuristic to skip memory-only locations if any
                 if location.starts_with("s3://") || location.starts_with("az://") || location.starts_with("gs://") {
                     // Use object store cache
                     let cache_key = self.get_object_store_cache_key(&warehouse.storage_config, location);
                     let store = self.object_store_cache.get_or_insert(cache_key, || {
                         Arc::new(crate::object_store_factory::create_object_store(&warehouse.storage_config, location)
                             .expect("Failed to create object store"))
                     });
                 
                     let path = self.extract_object_store_path(location);
                 
                     match store.get(&path).await {
                         Ok(result) => return Ok(result.bytes().await?.to_vec()),
                         Err(e) => {
                             tracing::warn!("Failed to read from object store for {}, falling back to memory: {}", location, e);
                         }
                     }
                 }
            }

            if let Some(data) = self.files.get(location) {
                Ok(data.value().clone())
            } else {
                Err(anyhow::anyhow!("File not found: {}", location))
            }
        }
    
    // Helper methods for object store operations
    pub fn get_warehouse_for_location(&self, location: &str) -> Option<pangolin_core::model::Warehouse> {
        let warehouses_map: Vec<pangolin_core::model::Warehouse> = self.warehouses
            .iter()
            .map(|entry| entry.value().clone())
            .collect();
        
        for warehouse in warehouses_map {
            if let Some(bucket) = warehouse.storage_config.get("s3.bucket").or_else(|| warehouse.storage_config.get("bucket")) {
                if location.contains(bucket) {
                    return Some(warehouse);
                }
            }
            if let Some(container) = warehouse.storage_config.get("azure.container") {
                if location.contains(container) {
                     return Some(warehouse);
                }
            }
            if let Some(bucket) = warehouse.storage_config.get("gcp.bucket") {
                if location.contains(bucket) {
                    return Some(warehouse);
                }
            }
        }
        None
    }

    fn get_object_store_cache_key(&self, config: &std::collections::HashMap<String, String>, location: &str) -> String {
        let endpoint = config.get("s3.endpoint").or_else(|| config.get("endpoint")).or_else(|| config.get("azure.endpoint")).or_else(|| config.get("gcp.endpoint")).map(|s| s.as_str()).unwrap_or("");
        let bucket = self.extract_bucket_from_location(location);
        let access_key = config.get("s3.access-key-id").or_else(|| config.get("access_key_id")).or_else(|| config.get("azure.account-name")).or_else(|| config.get("gcp.service-account")).map(|s| s.as_str()).unwrap_or("");
        let region = config.get("s3.region").or_else(|| config.get("region")).map(|s| s.as_str()).unwrap_or("us-east-1");
        
        crate::ObjectStoreCache::cache_key(endpoint, &bucket, access_key, region)
    }

    fn extract_bucket_from_location(&self, location: &str) -> String {
        if let Some(rest) = location.strip_prefix("s3://").or_else(|| location.strip_prefix("az://")).or_else(|| location.strip_prefix("gs://")) {
            if let Some((bucket, _)) = rest.split_once('/') {
                return bucket.to_string();
            }
            return rest.to_string();
        }
        "default".to_string()
    }

    fn extract_object_store_path(&self, location: &str) -> object_store::path::Path {
        if let Some(rest) = location.strip_prefix("s3://").or_else(|| location.strip_prefix("az://")).or_else(|| location.strip_prefix("gs://")) {
            if let Some((_, key)) = rest.split_once('/') {
                return object_store::path::Path::from(key);
            }
            return object_store::path::Path::from(rest);
        }
        object_store::path::Path::from(location)
    }
}
