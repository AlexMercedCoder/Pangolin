use super::MemoryStore;
use anyhow::Result;
use uuid::Uuid;
use crate::signer::Credentials;
use pangolin_core::model::VendingStrategy;
use pangolin_core::model::Warehouse;
use chrono::{Utc, DateTime};
use async_trait::async_trait;

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
                 Some(VendingStrategy::AwsSts { role_arn: _, external_id: _ }) => {
                     Err(anyhow::anyhow!("AWS STS vending not implemented yet via VendingStrategy in MemoryStore"))
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
                     let access_key = warehouse.storage_config.get("s3.access-key-id")
                        .ok_or_else(|| anyhow::anyhow!("Missing s3.access-key-id"))?;
                     let secret_key = warehouse.storage_config.get("s3.secret-access-key")
                        .ok_or_else(|| anyhow::anyhow!("Missing s3.secret-access-key"))?;
                    
                 if warehouse.use_sts {
                      // Existing STS Logic restored for backward compatibility
                      // MemoryStore doesn't usually make external calls, but to parity other stores:
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
                  
                      // For testing purposes, if we are in a test env without AWS creds, 
                      // this client.get_session_token().send().await will likely fail.
                      // This failure is what we expect in the regression test "execution attempt".
                  
                      let role_arn = warehouse.storage_config.get("s3.role-arn").map(|s| s.as_str());
              
                      if let Some(arn) = role_arn {
                           let resp = client.assume_role()
                              .role_arn(arn)
                              .role_session_name("pangolin-memory-legacy")
                              .send()
                              .await
                              .map_err(|e| anyhow::anyhow!("STS AssumeRole failed: {}", e))?;
                          
                           let c = resp.credentials.ok_or_else(|| anyhow::anyhow!("No credentials in AssumeRole response"))?;
                           Ok(Credentials::Aws {
                               access_key_id: c.access_key_id,
                               secret_access_key: c.secret_access_key,
                               session_token: Some(c.session_token),
                               expiration: chrono::DateTime::from_timestamp(c.expiration.secs(), c.expiration.subsec_nanos()),
                               // Note: MemoryStore doesn't return Credentials struct in same way?
                               // Actually MemoryStore::get_table_credentials returns Credentials struct.
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
    pub(crate) async fn presign_get_internal(&self, _location: &str) -> Result<String> {
        // Stub: Presigning requires keeping an ObjectStore client around or rebuilding it.
        // For this task, we focus on table credentials.
        Err(anyhow::anyhow!("MemoryStore does not support presigning yet"))
    }
}
