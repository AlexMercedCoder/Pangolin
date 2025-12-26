use super::MongoStore;
use anyhow::Result;
use mongodb::bson::{doc};
use pangolin_core::model::Warehouse;
use std::collections::HashMap;
use std::sync::Arc;
use object_store::aws::AmazonS3Builder;
use object_store::ObjectStore;
use object_store::path::Path as ObjPath;
use futures::stream::TryStreamExt;

impl MongoStore {
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

    pub async fn read_file_uncached(&self, path: &str) -> Result<Vec<u8>> {
        // Try to look up warehouse credentials first
        if let Some(store) = self.get_object_store(path).await? {
            // Extract key relative to bucket
            let key = if let Some(rest) = path.strip_prefix("s3://").or_else(|| path.strip_prefix("az://")).or_else(|| path.strip_prefix("gs://")) {
                 rest.split_once('/').map(|(_, k)| k).unwrap_or(rest)
            } else {
                 path
            };
            
            match store.get(&ObjPath::from(key)).await {
                Ok(result) => return Ok(result.bytes().await?.to_vec()),
                Err(e) => {
                     tracing::warn!("Failed to read from warehouse-configured store for {}, falling back to global env: {}", path, e);
                }
            }
        }

        if let Some(rest) = path.strip_prefix("s3://") {
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
             let location = ObjPath::from(key);
             let result = store.get(&location).await?;
             let bytes = result.bytes().await?;
             Ok(bytes.to_vec())
        } else {
             Err(anyhow::anyhow!("Only s3:// paths are supported in Mongo store"))
        }
    }

    pub async fn write_file(&self, path: &str, data: Vec<u8>) -> Result<()> {
        // Invalidate metadata cache on write
        if path.ends_with("metadata.json") || path.ends_with(".metadata.json") {
            self.metadata_cache.invalidate(path).await;
        }

        if let Some(store) = self.get_object_store(path).await? {
            // Extract key relative to bucket
            let key = if let Some(rest) = path.strip_prefix("s3://").or_else(|| path.strip_prefix("az://")).or_else(|| path.strip_prefix("gs://")) {
                 rest.split_once('/').map(|(_, k)| k).unwrap_or(rest)
            } else {
                 path
            };
            
            store.put(&ObjPath::from(key), data.into()).await?;
            return Ok(());
        }

        // Fallback to S3 global env
        if let Some(rest) = path.strip_prefix("s3://") {
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
             store.put(&ObjPath::from(key), data.into()).await?;
             Ok(())
        } else {
             Err(anyhow::anyhow!("Only s3:// paths are supported for write in Mongo store"))
        }
    }

    pub async fn get_object_store(&self, path: &str) -> Result<Option<Arc<dyn ObjectStore>>> {
        if let Some(warehouse) = self.get_warehouse_for_location(path).await? {
            if path.starts_with("s3://") || path.starts_with("az://") || path.starts_with("gs://") {
                let cache_key = self.get_object_store_cache_key(&warehouse.storage_config, path);
                let store = self.object_store_cache.try_get_or_insert::<_, anyhow::Error>(cache_key, || {
                    let os: Box<dyn ObjectStore> = crate::object_store_factory::create_object_store(&warehouse.storage_config, path)
                        .map_err(|e| anyhow::anyhow!("Failed to create object store for warehouse: {}", e))?;
                    let arc_os: Arc<dyn ObjectStore> = Arc::from(os);
                    Ok(arc_os)
                })?;
                return Ok(Some(store));
            }
        }
        Ok(None)
    }

    pub(crate) async fn get_warehouse_for_location(&self, location: &str) -> Result<Option<Warehouse>> {
         let cursor = self.warehouses().find(doc! {}).await.map_err(|e| anyhow::anyhow!(e))?;
         let warehouses: Vec<Warehouse> = cursor.try_collect().await.map_err(|e| anyhow::anyhow!(e))?;

         for warehouse in warehouses {
             let s3_match = warehouse.storage_config.get("s3.bucket").or_else(|| warehouse.storage_config.get("bucket")).map(|b| location.contains(b)).unwrap_or(false);
             let azure_match = warehouse.storage_config.get("azure.container").map(|c| location.contains(c)).unwrap_or(false);
             let gcp_match = warehouse.storage_config.get("gcp.bucket").map(|b| location.contains(b)).unwrap_or(false);
             
             if s3_match || azure_match || gcp_match {
                 return Ok(Some(warehouse));
             }
         }
         
         Ok(None)
    }

    pub(crate) fn get_object_store_cache_key(&self, config: &HashMap<String, String>, location: &str) -> String {
        let endpoint = config.get("s3.endpoint").or_else(|| config.get("endpoint")).or_else(|| config.get("azure.endpoint")).or_else(|| config.get("gcp.endpoint")).map(|s| s.as_str()).unwrap_or("");
        let bucket = config.get("s3.bucket").or_else(|| config.get("bucket")).or_else(|| config.get("azure.container")).or_else(|| config.get("gcp.bucket")).map(|s| s.as_str()).unwrap_or_else(|| {
            location.strip_prefix("s3://").or_else(|| location.strip_prefix("az://")).or_else(|| location.strip_prefix("gs://")).and_then(|s| s.split('/').next()).unwrap_or("")
        });
        let access_key = config.get("s3.access-key-id").or_else(|| config.get("access_key_id")).or_else(|| config.get("azure.account-name")).or_else(|| config.get("gcp.service-account-key")).map(|s| s.as_str()).unwrap_or("");
        let region = config.get("s3.region").or_else(|| config.get("region")).or_else(|| config.get("azure.region")).or_else(|| config.get("gcp.region")).map(|s| s.as_str()).unwrap_or("");
        crate::ObjectStoreCache::cache_key(endpoint, &bucket, access_key, region)
    }
}
