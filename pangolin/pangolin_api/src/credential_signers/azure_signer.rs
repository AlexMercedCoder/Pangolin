use super::{CredentialSigner, VendedCredentials};
use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use anyhow::{Result, anyhow};

#[cfg(feature = "azure-oauth")]
use azure_core::auth::TokenCredential;
#[cfg(feature = "azure-oauth")]
use azure_identity::ClientSecretCredential;

/// Azure ADLS Gen2 credential signer that generates SAS tokens
pub struct AzureSasSigner {
    pub account_name: String,
    pub account_key: Option<String>,
    pub tenant_id: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub container: String,
}

impl AzureSasSigner {
    pub fn new(
        account_name: String,
        account_key: Option<String>,
        tenant_id: Option<String>,
        client_id: Option<String>,
        client_secret: Option<String>,
        container: String,
    ) -> Self {
        Self {
            account_name,
            account_key,
            tenant_id,
            client_id,
            client_secret,
            container,
        }
    }
}

#[async_trait]
impl CredentialSigner for AzureSasSigner {
    async fn generate_credentials(
        &self,
        _resource_path: &str,
        _permissions: &[String],
        duration: Duration,
    ) -> Result<VendedCredentials> {
        #[cfg(feature = "azure-oauth")]
        {
            tracing::info!("🔑 Generating Azure credentials");
            
            let expires_at = Utc::now() + duration;
            
            // Try OAuth2 if credentials are available
            if let (Some(tenant_id), Some(client_id), Some(client_secret)) = 
                (&self.tenant_id, &self.client_id, &self.client_secret) {
                
                let authority_host = azure_core::Url::parse("https://login.microsoftonline.com")
                    .map_err(|e| anyhow!("Failed to parse authority host: {}", e))?;
                
                let credential = ClientSecretCredential::new(
                    azure_core::new_http_client(),
                    authority_host,
                    tenant_id.clone(),
                    client_id.clone(),
                    client_secret.clone(),
                );
                
                let token = credential
                    .get_token(&["https://storage.azure.com/.default"])
                    .await
                    .map_err(|e| anyhow!("Azure token acquisition failed: {}", e))?;
                
                let mut config = HashMap::new();
                config.insert("credential-type".to_string(), "azure-oauth".to_string());
                config.insert("azure-oauth-token".to_string(), token.token.secret().to_string());
                config.insert("azure-account-name".to_string(), self.account_name.clone());
                config.insert("azure-container".to_string(), self.container.clone());
                
                tracing::info!("✅ Successfully generated Azure OAuth2 token");
                
                return Ok(VendedCredentials {
                    prefix: format!("abfss://{}@{}.dfs.core.windows.net/", 
                        self.container, self.account_name),
                    config,
                    expires_at: Some(expires_at),
                });
            }
            
            // Fallback to account key if available
            if let Some(account_key) = &self.account_key {
                let mut config = HashMap::new();
                config.insert("credential-type".to_string(), "azure-key".to_string());
                config.insert("azure-account-name".to_string(), self.account_name.clone());
                config.insert("azure-account-key".to_string(), account_key.clone());
                config.insert("azure-container".to_string(), self.container.clone());
                
                tracing::info!("✅ Using Azure account key credentials");
                
                return Ok(VendedCredentials {
                    prefix: format!("abfss://{}@{}.dfs.core.windows.net/", 
                        self.container, self.account_name),
                    config,
                    expires_at: None, // Account keys don't expire
                });
            }
            
            Err(anyhow!("Azure credentials not configured properly"))
        }
        
        #[cfg(not(feature = "azure-oauth"))]
        {
            tracing::warn!("Azure OAuth feature not enabled, returning placeholder credentials");
            let mut config = HashMap::new();
            config.insert("credential-type".to_string(), "azure-sas".to_string());
            config.insert("azure-sas-token".to_string(), "PLACEHOLDER_SAS_TOKEN".to_string());
            config.insert("azure-account-name".to_string(), self.account_name.clone());
            config.insert("azure-container".to_string(), self.container.clone());
            
            Ok(VendedCredentials {
                prefix: format!("abfss://{}@{}.dfs.core.windows.net/", 
                    self.container, self.account_name),
                config,
                expires_at: Some(Utc::now() + duration),
            })
        }
    }
    
    fn storage_type(&self) -> &str {
        "azure"
    }
}
