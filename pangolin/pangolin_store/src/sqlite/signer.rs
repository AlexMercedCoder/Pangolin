use super::SqliteStore;
use anyhow::Result;
use async_trait::async_trait;
use sqlx::Row;
use std::collections::HashMap;
use crate::signer::{Signer, Credentials};
use pangolin_core::model::VendingStrategy;
use object_store::aws::AmazonS3Builder;

#[async_trait]
impl Signer for SqliteStore {
    async fn get_table_credentials(&self, location: &str) -> Result<Credentials> {
        // 1. Find the warehouse that owns this location by querying all warehouses
        let warehouses = sqlx::query("SELECT id, name, storage_config, use_sts, vending_strategy FROM warehouses")
            .fetch_all(&self.pool)
            .await?;
        
        let mut target_warehouse = None;
        
        for row in warehouses {
            let storage_config_str: String = row.get("storage_config");
            let storage_config: HashMap<String, String> = serde_json::from_str(&storage_config_str)?;
            
            // Check matches
             // Check AWS S3
            if let Some(bucket) = storage_config.get("s3.bucket") {
                if location.contains(bucket) {
                    target_warehouse = Some(row);
                    break;
                }
            }
            // Check Azure
            if let Some(container) = storage_config.get("azure.container") {
                if location.contains(container) {
                     target_warehouse = Some(row);
                     break;
                }
            }
            // Check GCP
            if let Some(bucket) = storage_config.get("gcp.bucket") {
                if location.contains(bucket) {
                    target_warehouse = Some(row);
                    break;
                }
            }
        }
        
        let row = target_warehouse.ok_or_else(|| anyhow::anyhow!("No warehouse found for location: {}", location))?;
        
        let storage_config_str: String = row.get("storage_config");
        let storage_config: HashMap<String, String> = serde_json::from_str(&storage_config_str)?;
        let use_sts: bool = row.try_get("use_sts").unwrap_or(false);
        let vending_strategy_str: Option<String> = row.try_get("vending_strategy").unwrap_or(None);
        let vending_strategy: Option<VendingStrategy> = vending_strategy_str
            .and_then(|s| serde_json::from_str(&s).ok());
            
        // 2. Check Vending Strategy
        if let Some(strategy) = vending_strategy {
             match strategy {
                 VendingStrategy::AwsSts { role_arn, external_id } => {
                     let client = crate::aws_utils::create_sts_client(&storage_config).await?;
                     crate::aws_utils::assume_role(
                         &client, 
                         Some(&role_arn), 
                         external_id.as_deref(), 
                         "pangolin-sqlite-sts"
                     ).await
                 }
                 VendingStrategy::AwsStatic { access_key_id, secret_access_key } => {
                     Ok(Credentials::Aws {
                         access_key_id,
                         secret_access_key,
                         session_token: None,
                         expiration: None,
                     })
                 }
                 VendingStrategy::AzureSas { account_name, account_key } => {
                     let signer = crate::azure_signer::AzureSigner::new(account_name.clone(), account_key);
                     let sas_token = signer.generate_sas_token(location).await?;
                     Ok(Credentials::Azure {
                         sas_token,
                         account_name,
                         expiration: chrono::Utc::now() + chrono::Duration::hours(1),
                     })
                 }
                 VendingStrategy::GcpDownscoped { service_account_email, private_key } => {
                     let signer = crate::gcp_signer::GcpSigner::new(service_account_email, private_key);
                     let access_token = signer.generate_downscoped_token(location).await?;
                     Ok(Credentials::Gcp {
                         access_token,
                         expiration: chrono::Utc::now() + chrono::Duration::hours(1),
                     })
                 }
                 VendingStrategy::None => Err(anyhow::anyhow!("Vending disabled")),
             }
        } else {
            // Legacy Logic
            if use_sts {
                 let client = crate::aws_utils::create_sts_client(&storage_config).await?;
                 let role_arn = storage_config.get("s3.role-arn").map(|s| s.as_str());
                 
                 crate::aws_utils::assume_role(
                     &client, 
                     role_arn, 
                     None, 
                     "pangolin-sqlite-legacy"
                 ).await
            } else {
                let access_key = storage_config.get("s3.access-key-id")
                    .ok_or_else(|| anyhow::anyhow!("Missing s3.access-key-id"))?;
                let secret_key = storage_config.get("s3.secret-access-key")
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

    async fn presign_get(&self, location: &str) -> Result<String> {
        // 1. Find the warehouse
        let warehouses = sqlx::query("SELECT id, name, storage_config FROM warehouses")
            .fetch_all(&self.pool)
            .await?;
            
        let mut target_config = None;
        
        for row in warehouses {
             let storage_config_str: String = row.get("storage_config");
             let storage_config: HashMap<String, String> = serde_json::from_str(&storage_config_str)?;
             
             let bucket_opt = storage_config.get("s3.bucket").map(|s| s.clone());
             
             if let Some(bucket) = bucket_opt {
                if location.contains(&bucket) {
                    target_config = Some((storage_config, bucket));
                    break;
                }
             }
        }
        
        let (config, bucket) = target_config.ok_or_else(|| anyhow::anyhow!("No warehouse found for location: {}", location))?;
        
        let prefix = format!("s3://{}/", bucket);
        if !location.starts_with(&prefix) {
             return Err(anyhow::anyhow!("Location {} does not match bucket {}", location, bucket));
        }
        let key = &location[prefix.len()..];
        
        // Use logic similar to MemoryStore's private helper, but inline here or move to common?
        // Let's duplicate inline for now to avoid modifying trait signature or extensive refactoring.
        // It's small enough.
        
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
