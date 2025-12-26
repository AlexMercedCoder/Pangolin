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

#[async_trait]
impl Signer for MongoStore {
    async fn get_table_credentials(&self, location: &str) -> Result<Credentials> {
         // Attempt to extract bucket/container from location
         let container = if location.starts_with("s3://") {
             location[5..].split('/').next().unwrap_or("").to_string()
         } else if location.starts_with("az://") {
             location[5..].split('/').next().unwrap_or("").to_string()
         } else if location.starts_with("gs://") {
             location[5..].split('/').next().unwrap_or("").to_string()
         } else if location.starts_with("abfs://") {
             location[7..].split('/').next().unwrap_or("").split('@').next().unwrap_or("").to_string()
         } else {
             String::new() 
         };

         // Find warehouse matching this container
         let filter = doc! {
             "$or": [
                 { "storage_config.s3.bucket": &container },
                 { "storage_config.azure.container": &container },
                 { "storage_config.gcp.bucket": &container }
             ]
         };
         
         let warehouse = self.warehouses().find_one(filter).await?
             .ok_or_else(|| anyhow::anyhow!("No warehouse found for location: {}", location))?;

         match &warehouse.vending_strategy {
             Some(VendingStrategy::AwsSts { role_arn: _, external_id: _ }) => {
                 Err(anyhow::anyhow!("AWS STS vending not implemented yet via VendingStrategy in MongoStore"))
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
                 let signer = crate::azure_signer::AzureSigner::new(account_name.clone(), account_key.clone());
                 let sas_token = signer.generate_sas_token(location).await?;
                 Ok(Credentials::Azure {
                     sas_token,
                     account_name: account_name.clone(),
                     expiration: chrono::Utc::now() + chrono::Duration::hours(1),
                 })
             }
             Some(VendingStrategy::GcpDownscoped { service_account_email, private_key }) => {
                 let signer = crate::gcp_signer::GcpSigner::new(service_account_email.clone(), private_key.clone());
                 let access_token = signer.generate_downscoped_token(location).await?;
                 Ok(Credentials::Gcp {
                     access_token,
                     expiration: chrono::Utc::now() + chrono::Duration::hours(1),
                 })
             }
             Some(VendingStrategy::None) => Err(anyhow::anyhow!("Vending disabled")),
             None => {
                 // Backward compatibility logic
                 let access_key = warehouse.storage_config.get("s3.access-key-id")
                    .ok_or_else(|| anyhow::anyhow!("Missing s3.access-key-id"))?;
                 let secret_key = warehouse.storage_config.get("s3.secret-access-key")
                    .ok_or_else(|| anyhow::anyhow!("Missing s3.secret-access-key"))?;
                    
                 if warehouse.use_sts {
                      let region = warehouse.storage_config.get("s3.region")
                          .map(|s| s.as_str())
                          .unwrap_or("us-east-1");
                          
                      let endpoint = warehouse.storage_config.get("s3.endpoint")
                          .map(|s| s.as_str());
      
                      let creds = aws_credential_types::Credentials::new(
                          access_key.to_string(),
                          secret_key.to_string(),
                          None,
                          None,
                          "legacy_provider"
                      );
                      
                      let config_loader = aws_config::from_env()
                          .region(aws_config::Region::new(region.to_string()))
                          .credentials_provider(creds);
                          
                      let config = if let Some(ep) = endpoint {
                           config_loader.endpoint_url(ep).load().await
                      } else {
                           config_loader.load().await
                      };
                      
                      let client = aws_sdk_sts::Client::new(&config);
                      
                      let role_arn = warehouse.storage_config.get("s3.role-arn").map(|s| s.as_str());
                  
                      if let Some(arn) = role_arn {
                           let resp = client.assume_role()
                               .role_arn(arn)
                               .role_session_name("pangolin-mongo-legacy")
                               .send()
                               .await
                               .map_err(|e| anyhow::anyhow!("STS AssumeRole failed: {}", e))?;
                               
                           let c = resp.credentials.ok_or_else(|| anyhow::anyhow!("No credentials in AssumeRole response"))?;
                           Ok(Credentials::Aws {
                               access_key_id: c.access_key_id,
                               secret_access_key: c.secret_access_key,
                               session_token: Some(c.session_token),
                               expiration: chrono::DateTime::from_timestamp(c.expiration.secs(), c.expiration.subsec_nanos()),
                           })
                      } else {
                           let resp = client.get_session_token()
                               .send()
                               .await
                               .map_err(|e| anyhow::anyhow!("STS GetSessionToken failed: {}", e))?;
                               
                           let c = resp.credentials.ok_or_else(|| anyhow::anyhow!("No credentials in GetSessionToken response"))?;
                           Ok(Credentials::Aws {
                               access_key_id: c.access_key_id,
                               secret_access_key: c.secret_access_key,
                               session_token: Some(c.session_token),
                               expiration: chrono::DateTime::from_timestamp(c.expiration.secs(), c.expiration.subsec_nanos()),
                           })
                      }
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
    }

    async fn presign_get(&self, _location: &str) -> Result<String> {
        Err(anyhow::anyhow!("MongoStore does not support presigning yet"))
    }
}
