/// IO operations for SqliteStore (File reading/writing)
use super::SqliteStore;
use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;
use std::sync::Arc;
use object_store::ObjectStore;
use object_store::aws::AmazonS3Builder;
use object_store::path::Path as ObjPath;
use pangolin_core::model::Warehouse;

impl SqliteStore {
    pub async fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        // Use metadata cache for metadata.json files
        if path.ends_with("metadata.json") || path.ends_with(".metadata.json") {
            return self.metadata_cache.get_or_fetch(path, || async {
                self.read_file_uncached(path).await
            }).await;
        }
        
        // Non-metadata files bypass cache
        self.read_file_uncached(path).await
    }


    pub async fn write_file(&self, path: &str, data: Vec<u8>) -> Result<()> {
        // Invalidate metadata cache on write
        if path.ends_with("metadata.json") || path.ends_with(".metadata.json") {
            self.metadata_cache.invalidate(path).await;
        }
        
        // Try to look up warehouse credentials first
        if let Some(warehouse) = self.get_warehouse_for_location(path).await? {
            if path.starts_with("s3://") || path.starts_with("az://") || path.starts_with("gs://") {
                // Use cached object store
                let cache_key = self.get_object_store_cache_key(&warehouse.storage_config, path);
                let store = self.object_store_cache.get_or_insert(cache_key, || {
                    Arc::new(crate::object_store_factory::create_object_store(&warehouse.storage_config, path).unwrap())
                });
                // Extract key relative to bucket
                let key = if let Some(rest) = path.strip_prefix("s3://").or_else(|| path.strip_prefix("az://")).or_else(|| path.strip_prefix("gs://")) {
                     rest.split_once('/').map(|(_, k)| k).unwrap_or(rest)
                } else {
                     path
                };
                
                store.put(&object_store::path::Path::from(key), data.into()).await?;
                return Ok(());
            }
        }

        if let Some(rest) = path.strip_prefix("file://") {
             let path = std::path::Path::new(rest);
             if let Some(parent) = path.parent() {
                 tokio::fs::create_dir_all(parent).await?;
             }
             match tokio::fs::write(path, data).await {
                 Ok(_) => Ok(()),
                 Err(e) => Err(anyhow::anyhow!("Failed to write local file {}: {}", rest, e)),
             }
        } else if let Some(rest) = path.strip_prefix("s3://") {
            let (bucket, key) = rest.split_once('/').ok_or_else(|| anyhow::anyhow!("Invalid S3 path"))?;
            
            tracing::info!("write_file: s3 path='{}' bucket='{}' key='{}'", path, bucket, key);
            
            let mut builder = AmazonS3Builder::new()
                .with_bucket_name(bucket)
                .with_allow_http(true);
                
             if let Ok(endpoint) = std::env::var("S3_ENDPOINT") {
                 builder = builder.with_endpoint(endpoint);
             }
             if let Ok(key_id) = std::env::var("AWS_ACCESS_KEY_ID") {
                 builder = builder.with_access_key_id(key_id);
             }
             if let Ok(secret) = std::env::var("AWS_SECRET_ACCESS_KEY") {
                 builder = builder.with_secret_access_key(secret);
             }
             if let Ok(region) = std::env::var("AWS_REGION") {
                 builder = builder.with_region(region);
             }
             
             let store = builder.build()?;
             match store.put(&ObjPath::from(key), data.into()).await {
                 Ok(_) => {
                     tracing::info!("write_file: Successfully wrote to S3: {}/{}", bucket, key);
                     Ok(())
                 },
                 Err(e) => {
                     tracing::error!("write_file: Failed to write to S3: {}", e);
                     Err(anyhow::anyhow!(e))
                 }
             }
        } else {
             Err(anyhow::anyhow!("Only s3:// and file:// paths are supported in SQLite store"))
        }
    }

    pub async fn read_file_uncached(&self, path: &str) -> Result<Vec<u8>> {
        // Try to look up warehouse credentials first
        if let Some(warehouse) = self.get_warehouse_for_location(path).await? {
            if path.starts_with("s3://") || path.starts_with("az://") || path.starts_with("gs://") {
                // Use cached object store
                let cache_key = self.get_object_store_cache_key(&warehouse.storage_config, path);
                let store = self.object_store_cache.get_or_insert(cache_key, || {
                    Arc::new(crate::object_store_factory::create_object_store(&warehouse.storage_config, path).unwrap())
                });
                // Extract key relative to bucket
                let key = if let Some(rest) = path.strip_prefix("s3://").or_else(|| path.strip_prefix("az://")).or_else(|| path.strip_prefix("gs://")) {
                     rest.split_once('/').map(|(_, k)| k).unwrap_or(rest)
                } else {
                     path
                };
                
                match store.get(&object_store::path::Path::from(key)).await {
                    Ok(result) => return Ok(result.bytes().await?.to_vec()),
                    Err(e) => {
                         tracing::warn!("Failed to read from warehouse-configured store for {}, falling back to global env: {}", path, e);
                    }
                }
            }
        }

        if let Some(rest) = path.strip_prefix("file://") {
             let path = std::path::Path::new(rest);
             match tokio::fs::read(path).await {
                 Ok(bytes) => Ok(bytes),
                 Err(e) => Err(anyhow::anyhow!("Failed to read local file {}: {}", rest, e)),
             }
        } else if let Some(rest) = path.strip_prefix("s3://") {
            let (bucket, key) = rest.split_once('/').ok_or_else(|| anyhow::anyhow!("Invalid S3 path"))?;
            
            let mut builder = AmazonS3Builder::new()
                .with_bucket_name(bucket)
                .with_allow_http(true);
                
             if let Ok(endpoint) = std::env::var("S3_ENDPOINT") {
                 builder = builder.with_endpoint(endpoint);
             }
             if let Ok(key_id) = std::env::var("AWS_ACCESS_KEY_ID") {
                 builder = builder.with_access_key_id(key_id);
             }
             if let Ok(secret) = std::env::var("AWS_SECRET_ACCESS_KEY") {
                 builder = builder.with_secret_access_key(secret);
             }
             if let Ok(region) = std::env::var("AWS_REGION") {
                 builder = builder.with_region(region);
             }
             
             let store = builder.build()?;
             let result = store.get(&ObjPath::from(key)).await?;
             let bytes = result.bytes().await?;
             Ok(bytes.to_vec())
        } else {
             Err(anyhow::anyhow!("Only s3:// and file:// paths are supported in SQLite store"))
        }
    }

    pub fn get_object_store_cache_key(&self, config: &std::collections::HashMap<String, String>, location: &str) -> String {
        let endpoint = config.get("s3.endpoint").or_else(|| config.get("endpoint")).or_else(|| config.get("azure.endpoint")).or_else(|| config.get("gcp.endpoint")).map(|s| s.as_str()).unwrap_or("");
        let bucket = config.get("s3.bucket").or_else(|| config.get("bucket")).or_else(|| config.get("azure.container")).or_else(|| config.get("gcp.bucket")).map(|s| s.as_str()).unwrap_or_else(|| {
            location.strip_prefix("s3://").or_else(|| location.strip_prefix("az://")).or_else(|| location.strip_prefix("gs://")).and_then(|s| s.split('/').next()).unwrap_or("")
        });
        let access_key = config.get("s3.access-key-id").or_else(|| config.get("access_key_id")).or_else(|| config.get("azure.account-name")).or_else(|| config.get("gcp.service-account-key")).map(|s| s.as_str()).unwrap_or("");
        let region = config.get("s3.region").or_else(|| config.get("region")).or_else(|| config.get("azure.region")).or_else(|| config.get("gcp.region")).map(|s| s.as_str()).unwrap_or("");
        crate::ObjectStoreCache::cache_key(endpoint, &bucket, access_key, region)
    }

    pub async fn get_warehouse_for_location(&self, location: &str) -> Result<Option<Warehouse>> {
        let rows = sqlx::query("SELECT id, name, tenant_id, use_sts, storage_config, vending_strategy FROM warehouses")
            .fetch_all(&self.pool)
            .await?;
        
        tracing::info!("DEBUG_SQLITE: get_warehouse_for_location checking {} warehouses for location: {}", rows.len(), location);

        for row in rows {
            let config_str: String = row.get("storage_config");
            tracing::info!("DEBUG_SQLITE: Checking warehouse ID: {:?}, config: {}", row.get::<String, _>("id"), config_str);

            if let Ok(config) = serde_json::from_str::<std::collections::HashMap<String, String>>(&config_str) {
                 // Check if location contains the bucket/container name (like MemoryStore does)
                 let s3_match = config.get("s3.bucket").or_else(|| config.get("bucket")).map(|b| location.contains(b)).unwrap_or(false);
                 let azure_match = config.get("azure.container").map(|c| location.contains(c)).unwrap_or(false);
                 let gcp_match = config.get("gcp.bucket").map(|b| location.contains(b)).unwrap_or(false);
                 
                 if s3_match || azure_match || gcp_match {
                      let vending_strategy_str: Option<String> = row.get("vending_strategy");
                      let vending_strategy = vending_strategy_str.and_then(|s| serde_json::from_str(&s).ok());
                      
                      tracing::info!("DEBUG_SQLITE: Match found!");
                      return Ok(Some(Warehouse {
                          id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                          tenant_id: Uuid::parse_str(&row.get::<String, _>("tenant_id"))?,
                          name: row.get("name"),
                          use_sts: row.get::<i32, _>("use_sts") != 0,
                          storage_config: config,
                          vending_strategy,
                      }));
                 } else {
                     tracing::info!("DEBUG_SQLITE: No match. s3_match={}, azure_match={}, gcp_match={}", s3_match, azure_match, gcp_match);
                 }
            } else {
                tracing::info!("DEBUG_SQLITE: Failed to parse storage_config json");
            }
        }
        Ok(None)
    }
}
