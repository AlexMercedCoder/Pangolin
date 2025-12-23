use crate::credential_signers::{
    CredentialSigner,
    azure_signer::AzureSasSigner,
    gcp_signer::GcpTokenSigner,
    s3_signer::S3Signer,
};
use pangolin_core::model::Warehouse;
use anyhow::{Result, anyhow};
use std::sync::Arc;
use chrono::Duration;

/// Factory function to create a credential signer based on warehouse configuration
pub fn create_signer_from_warehouse(warehouse: &Warehouse) -> Result<Box<dyn CredentialSigner>> {
    let storage_type = warehouse.storage_config.get("type")
        .map(|s| s.as_str())
        .unwrap_or("s3");
    
    match storage_type {
        "azure" => {
            let account_name = warehouse.storage_config.get("account_name")
                .cloned()
                .ok_or_else(|| anyhow!("Azure warehouse missing account_name"))?;
            
            let container = warehouse.storage_config.get("container")
                .cloned()
                .ok_or_else(|| anyhow!("Azure warehouse missing container"))?;
            
            let account_key = warehouse.storage_config.get("account_key").cloned();
            let tenant_id = warehouse.storage_config.get("tenant_id").cloned();
            let client_id = warehouse.storage_config.get("client_id").cloned();
            let client_secret = warehouse.storage_config.get("client_secret").cloned();
            let authority_host = warehouse.storage_config.get("authority_host").cloned();
            
            let mut signer = AzureSasSigner::new(
                account_name,
                account_key,
                tenant_id,
                client_id,
                client_secret,
                container,
            );
            
            // Set custom authority host if provided (for testing)
            if let Some(host) = authority_host {
                signer = signer.with_authority_host(host);
            }
            
            Ok(Box::new(signer))
        },
        "gcs" | "gcp" => {
            let project_id = warehouse.storage_config.get("project_id")
                .cloned()
                .ok_or_else(|| anyhow!("GCP warehouse missing project_id"))?;
            
            let bucket = warehouse.storage_config.get("bucket")
                .cloned()
                .ok_or_else(|| anyhow!("GCP warehouse missing bucket"))?;
            
            let service_account_key = warehouse.storage_config.get("service_account_key").cloned();
            
            Ok(Box::new(GcpTokenSigner::new(
                project_id,
                bucket,
                service_account_key,
            )))
        },
        "s3" | _ => {
            let bucket = warehouse.storage_config.get("bucket")
                .cloned()
                .ok_or_else(|| anyhow!("S3 warehouse missing bucket"))?;
            
            let role_arn = if warehouse.use_sts {
                warehouse.storage_config.get("role_arn").cloned()
            } else {
                None
            };
            
            let external_id = warehouse.storage_config.get("external_id").cloned();
            let access_key = warehouse.storage_config.get("access_key_id").cloned();
            let secret_key = warehouse.storage_config.get("secret_access_key").cloned();
            let region = warehouse.storage_config.get("region").cloned();
            let endpoint = warehouse.storage_config.get("endpoint").cloned();
            
            Ok(Box::new(S3Signer::new(
                role_arn,
                external_id,
                access_key,
                secret_key,
                bucket,
                region,
                endpoint,
            )))
        }
    }
}

/// Vend credentials for a given warehouse and resource path
pub async fn vend_credentials_for_warehouse(
    warehouse: &Warehouse,
    resource_path: &str,
    permissions: &[String],
) -> Result<std::collections::HashMap<String, String>> {
    let signer = create_signer_from_warehouse(warehouse)?;
    
    // Default to 1 hour expiration
    let duration = Duration::hours(1);
    
    let vended_creds = signer.generate_credentials(resource_path, permissions, duration).await?;
    
    Ok(vended_creds.config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use uuid::Uuid;
    
    #[test]
    fn test_create_s3_signer() {
        let mut storage_config = HashMap::new();
        storage_config.insert("type".to_string(), "s3".to_string());
        storage_config.insert("bucket".to_string(), "test-bucket".to_string());
        storage_config.insert("access_key_id".to_string(), "AKIATEST".to_string());
        storage_config.insert("secret_access_key".to_string(), "secret".to_string());
        
        let warehouse = Warehouse {
            id: Uuid::new_v4(),
            name: "test-warehouse".to_string(),
            tenant_id: Uuid::new_v4(),
            storage_config,
            use_sts: false,
            vending_strategy: None,
        };
        
        let result = create_signer_from_warehouse(&warehouse);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().storage_type(), "s3");
    }
    
    #[test]
    fn test_create_azure_signer() {
        let mut storage_config = HashMap::new();
        storage_config.insert("type".to_string(), "azure".to_string());
        storage_config.insert("account_name".to_string(), "testaccount".to_string());
        storage_config.insert("container".to_string(), "testcontainer".to_string());
        storage_config.insert("account_key".to_string(), "testkey".to_string());
        
        let warehouse = Warehouse {
            id: Uuid::new_v4(),
            name: "test-warehouse".to_string(),
            tenant_id: Uuid::new_v4(),
            storage_config,
            use_sts: false,
            vending_strategy: None,
        };
        
        let result = create_signer_from_warehouse(&warehouse);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().storage_type(), "azure");
    }
    
    #[test]
    fn test_create_gcp_signer() {
        let mut storage_config = HashMap::new();
        storage_config.insert("type".to_string(), "gcs".to_string());
        storage_config.insert("project_id".to_string(), "test-project".to_string());
        storage_config.insert("bucket".to_string(), "test-bucket".to_string());
        
        let warehouse = Warehouse {
            id: Uuid::new_v4(),
            name: "test-warehouse".to_string(),
            tenant_id: Uuid::new_v4(),
            storage_config,
            use_sts: false,
            vending_strategy: None,
        };
        
        let result = create_signer_from_warehouse(&warehouse);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().storage_type(), "gcs");
    }
    
    #[test]
    fn test_missing_required_config() {
        let mut storage_config = HashMap::new();
        storage_config.insert("type".to_string(), "s3".to_string());
        // Missing bucket
        
        let warehouse = Warehouse {
            id: Uuid::new_v4(),
            name: "test-warehouse".to_string(),
            tenant_id: Uuid::new_v4(),
            storage_config,
            use_sts: false,
            vending_strategy: None,
        };
        
        let result = create_signer_from_warehouse(&warehouse);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("missing bucket"));
        }
    }
}
