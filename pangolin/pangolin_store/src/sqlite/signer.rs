use super::SqliteStore;
use anyhow::Result;
use async_trait::async_trait;
use sqlx::Row;
use std::collections::HashMap;
use crate::signer::{Signer, Credentials};
use pangolin_core::model::VendingStrategy;

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
                 VendingStrategy::AwsSts { role_arn: _, external_id: _ } => {
                     Err(anyhow::anyhow!("AWS STS vending not implemented yet via VendingStrategy in SqliteStore"))
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
            let access_key = storage_config.get("s3.access-key-id")
                .ok_or_else(|| anyhow::anyhow!("Missing s3.access-key-id"))?;
            let secret_key = storage_config.get("s3.secret-access-key")
                .ok_or_else(|| anyhow::anyhow!("Missing s3.secret-access-key"))?;
                
            if use_sts {
                  // Existing STS Logic restored for backward compatibility
                  let region = storage_config.get("s3.region")
                      .map(|s| s.as_str())
                      .unwrap_or("us-east-1");
                      
                  let endpoint = storage_config.get("s3.endpoint")
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
                  
                  let role_arn = storage_config.get("s3.role-arn").map(|s| s.as_str());
              
                  if let Some(arn) = role_arn {
                       let resp = client.assume_role()
                          .role_arn(arn)
                          .role_session_name("pangolin-sqlite-legacy")
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

    async fn presign_get(&self, _location: &str) -> Result<String> {
        Err(anyhow::anyhow!("SqliteStore does not support presigning yet"))
    }
}
