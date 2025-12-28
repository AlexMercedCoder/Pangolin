use super::MemoryStore;
use anyhow::Result;
use uuid::Uuid;
use crate::signer::Credentials;
use pangolin_core::model::VendingStrategy;
use pangolin_core::model::Warehouse;
use chrono::{Utc, DateTime};
use async_trait::async_trait;
use std::collections::HashMap;
use object_store::aws::AmazonS3Builder;

impl MemoryStore {
    pub(crate) async fn get_table_credentials_internal(&self, location: &str) -> Result<Credentials> {
            // 1. Find the warehouse that owns this location
            // Iterate over all warehouses
            let warehouses_map: Vec<Warehouse> = self.warehouses
                .iter()
                .map(|entry| entry.value().clone())
                .collect();

            // Simple prefix match. In real world, we might want more robust matching.
            let mut target_warehouse = None;
        
            for warehouse in warehouses_map {
                // Check AWS S3
                if let Some(bucket) = warehouse.storage_config.get("s3.bucket") {
                    if location.contains(bucket) {
                        target_warehouse = Some(warehouse);
                        break;
                    }
                }
                // Check Azure
                if let Some(container) = warehouse.storage_config.get("azure.container") {
                    if location.contains(container) {
                         target_warehouse = Some(warehouse);
                         break;
                    }
                }
                // Check GCP
                if let Some(bucket) = warehouse.storage_config.get("gcp.bucket") {
                    if location.contains(bucket) {
                        target_warehouse = Some(warehouse);
                        break;
                    }
                }
            }

            let warehouse = target_warehouse.ok_or_else(|| anyhow::anyhow!("No warehouse found for location: {}", location))?;
        
            // 2. Check Vending Strategy
            match &warehouse.vending_strategy {
                 Some(VendingStrategy::AwsSts { role_arn, external_id }) => {
                     let client = crate::aws_utils::create_sts_client(&warehouse.storage_config).await?;
                     crate::aws_utils::assume_role(
                         &client, 
                         Some(role_arn), 
                         external_id.as_deref(), 
                         "pangolin-memory-sts"
                     ).await
                 }
                 Some(VendingStrategy::AwsStatic { access_key_id, secret_access_key }) => {
                     Ok(Credentials::Aws {
                         access_key_id: access_key_id.clone(),
                         secret_access_key: secret_access_key.clone(),
                         session_token: None,
                         expiration: None,
                     })
                 }
                 Some(VendingStrategy::AzureSas { account_name, account_key }) => {
                     #[cfg(feature = "azure")]
                     {
                         let signer = crate::azure_signer::AzureSigner::new(account_name.clone(), account_key.clone());
                         let sas_token = signer.generate_sas_token(location).await?;
                         Ok(Credentials::Azure {
                             sas_token,
                             account_name: account_name.clone(),
                             expiration: chrono::Utc::now() + chrono::Duration::hours(1),
                         })
                     }
                     #[cfg(not(feature = "azure"))]
                     Err(anyhow::anyhow!("Azure vending requires 'azure' feature"))
                 }
                 Some(VendingStrategy::GcpDownscoped { service_account_email, private_key }) => {
                     #[cfg(feature = "gcp")]
                     {
                         let signer = crate::gcp_signer::GcpSigner::new(service_account_email.clone(), private_key.clone());
                         let access_token = signer.generate_downscoped_token(location).await?;
                         Ok(Credentials::Gcp {
                             access_token,
                             expiration: chrono::Utc::now() + chrono::Duration::hours(1),
                         })
                     }
                     #[cfg(not(feature = "gcp"))]
                     Err(anyhow::anyhow!("GCP vending requires 'gcp' feature"))
                 }
                 Some(VendingStrategy::None) => Err(anyhow::anyhow!("Vending disabled")),
                 None => {
                     // Backward compatibility logic
                     if warehouse.use_sts {
                        let client = crate::aws_utils::create_sts_client(&warehouse.storage_config).await?;
                        let role_arn = warehouse.storage_config.get("s3.role-arn").map(|s| s.as_str());
                        
                        crate::aws_utils::assume_role(
                            &client,
                            role_arn,
                            None,
                            "pangolin-memory-legacy"
                        ).await
                     } else {
                         let access_key = warehouse.storage_config.get("s3.access-key-id")
                            .ok_or_else(|| anyhow::anyhow!("Missing s3.access-key-id"))?;
                         let secret_key = warehouse.storage_config.get("s3.secret-access-key")
                            .ok_or_else(|| anyhow::anyhow!("Missing s3.secret-access-key"))?;
                            
                         Ok(Credentials::Aws {
                             access_key_id: access_key.clone(),
                             secret_access_key: secret_key.clone(),
                             session_token: None,
                             expiration: None,
                         })
                     }
                 }
            }
        }
    pub(crate) async fn presign_get_internal(&self, location: &str) -> Result<String> {
        // 1. Find the warehouse (reuse logic or refactor, duplicating for now to avoid borrow checker pain with self)
        let warehouses_map: Vec<Warehouse> = self.warehouses
            .iter()
            .map(|entry| entry.value().clone())
            .collect();
            
        let mut target_warehouse = None;
        for warehouse in warehouses_map {
             if let Some(bucket) = warehouse.storage_config.get("s3.bucket") {
                if location.contains(bucket) {
                    target_warehouse = Some(warehouse);
                    break;
                }
            }
        }
        
        let warehouse = target_warehouse.ok_or_else(|| anyhow::anyhow!("No warehouse found for location: {}", location))?;
        
        // 2. Build S3 Client from Warehouse Config
        // Note: Using object_store specific builder, separate from SDK builder
        let bucket = warehouse.storage_config.get("s3.bucket")
            .ok_or_else(|| anyhow::anyhow!("Missing s3.bucket"))?;
            
        let mut builder = AmazonS3Builder::new()
            .with_bucket_name(bucket)
            .with_allow_http(true); // Allow HTTP for local dev
            
        if let Some(ep) = warehouse.storage_config.get("s3.endpoint") {
            builder = builder.with_endpoint(ep);
        }
        if let Some(region) = warehouse.storage_config.get("s3.region") {
            builder = builder.with_region(region);
        }
        if let Some(ak) = warehouse.storage_config.get("s3.access-key-id") {
            builder = builder.with_access_key_id(ak);
        }
        if let Some(sk) = warehouse.storage_config.get("s3.secret-access-key") {
            builder = builder.with_secret_access_key(sk);
        }
        
        // Attempt to extract key from location
        // location = s3://bucket/path/to/file
        let prefix = format!("s3://{}/", bucket);
        if !location.starts_with(&prefix) {
             return Err(anyhow::anyhow!("Location {} does not match bucket {}", location, bucket));
        }
        let key = &location[prefix.len()..];
        
        // This requires the 'aws' feature on object_store crate
        let store = builder.build()?;
        
        // Presign
        // object_store 0.10+ supports presign_get
        // Current ObjectStore trait in pangolin_store dependencies?
        // Assuming it's available.
        
        // Note: object_store::aws::AmazonS3 implements presign through an extension or method?
        // Actually, internal AmazonS3 struct has it. Box<dyn ObjectStore> doesn't expose it directly 
        // in all versions without a specific trait bound or downcast.
        // However, we just built a concrete AmazonS3 struct.
        
        // Checking object_store docs: get_opts -> GetResult. Not direct presign URL.
        // Wait, AmazonS3 struct has `sign_request` but maybe not high level `presign_get`.
        // Let's check imports. `object_store::aws::AmazonS3`? 
        // If not available, we might need a direct `aws_sdk_s3` presigning client.
        // Using `aws_sdk_s3` is probably safer given we already rely on it for STS.
        
        // Switch to `presign_using_sdk` logic
        Self::presign_using_sdk(&warehouse.storage_config, bucket, key).await
    }
    
    async fn presign_using_sdk(config: &HashMap<String, String>, bucket: &str, key: &str) -> Result<String> {
        let region = config.get("s3.region").map(|s| aws_config::Region::new(s.to_string())).unwrap_or(aws_config::Region::new("us-east-1"));
        let access_key = config.get("s3.access-key-id").unwrap_or(&String::new()).to_string();
        let secret_key = config.get("s3.secret-access-key").unwrap_or(&String::new()).to_string();
        
         let creds = aws_credential_types::Credentials::new(
            access_key,
            secret_key,
            None,
            None,
            "presigner"
        );
        
        let config_loader = aws_config::from_env()
            .region(region)
            .credentials_provider(creds);
            
         let sdk_config = if let Some(ep) = config.get("s3.endpoint") {
            config_loader.endpoint_url(ep).load().await
        } else {
            config_loader.load().await
        };
        
        let client = aws_sdk_s3::Client::new(&sdk_config);
        let presigning_config = aws_sdk_s3::presigning::PresigningConfig::expires_in(std::time::Duration::from_secs(3600))?;
        
        let presigned_req = client.get_object()
            .bucket(bucket)
            .key(key)
            .presigned(presigning_config)
            .await?;
            
        Ok(presigned_req.uri().to_string())
    }
}
