use super::MongoStore;
use super::main::{to_bson_uuid, from_bson_uuid};
use crate::signer::{Signer, Credentials};
use anyhow::Result;
use async_trait::async_trait;
use mongodb::bson::{doc};
use pangolin_core::model::{Warehouse, VendingStrategy};
use std::collections::HashMap;
use std::sync::Arc;
use object_store::aws::AmazonS3Builder;
use object_store::ObjectStore;
use object_store::path::Path as ObjPath;
use futures::stream::TryStreamExt;
use crate::aws_utils; // Added for new STS logic
use crate::azure_signer; // Added for Azure signer
use crate::gcp_signer; // Added for GCP signer
use aws_sdk_s3; // Added for S3 presigning
use aws_credential_types; // Added for S3 presigning
use aws_config; // Added for S3 presigning
use std::time::Duration; // Added for S3 presigning

#[async_trait]
impl Signer for MongoStore {
    async fn get_table_credentials(&self, location: &str) -> Result<Credentials> {
        let collection = self.db.collection::<Warehouse>("warehouses");
        let mut cursor = collection.find(doc!{}).await?;
        
        let mut target_warehouse = None;
        
        while let Some(warehouse) = cursor.try_next().await? {
             // Check matches
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
        if let Some(strategy) = warehouse.vending_strategy {
             match strategy {
                 VendingStrategy::AwsSts { role_arn, external_id } => {
                     let client = crate::aws_utils::create_sts_client(&warehouse.storage_config).await?;
                     crate::aws_utils::assume_role(
                         &client, 
                         Some(&role_arn), 
                         external_id.as_deref(), 
                         "pangolin-mongo-sts"
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
            let access_key = warehouse.storage_config.get("s3.access-key-id")
                .ok_or_else(|| anyhow::anyhow!("Missing s3.access-key-id"))?;
            let secret_key = warehouse.storage_config.get("s3.secret-access-key")
                .ok_or_else(|| anyhow::anyhow!("Missing s3.secret-access-key"))?;
                
             if warehouse.use_sts {
                  // Existing STS Logic restored for backward compatibility
                  let client = crate::aws_utils::create_sts_client(&warehouse.storage_config).await?;
                  let role_arn = warehouse.storage_config.get("s3.role-arn").map(|s| s.as_str());
                  
                  crate::aws_utils::assume_role(
                     &client, 
                     role_arn, 
                     None, 
                     "pangolin-mongo-legacy"
                  ).await
             } else {
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
        let collection = self.db.collection::<Warehouse>("warehouses");
        let mut cursor = collection.find(doc!{}).await?;
        
        let mut target_config = None;
        
        while let Some(warehouse) = cursor.try_next().await? {
             let bucket_opt = warehouse.storage_config.get("s3.bucket").map(|s| s.clone());
             
             if let Some(bucket) = bucket_opt {
                if location.contains(&bucket) {
                    target_config = Some((warehouse.storage_config, bucket));
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
