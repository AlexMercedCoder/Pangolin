use super::{CredentialSigner, VendedCredentials};
use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use anyhow::{Result, anyhow};

#[cfg(feature = "azure-oauth")]
use azure_core::auth::TokenCredential;
#[cfg(feature = "azure-oauth")]
use azure_identity::ClientSecretCredential;

/// Azure ADLS Gen2 credential signer that generates OAuth2 tokens
pub struct AzureSasSigner {
    pub account_name: String,
    pub account_key: Option<String>,
    pub tenant_id: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub container: String,
    pub authority_host: Option<String>, // Custom authority host for testing
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
            authority_host: None, // Default to None (uses production Azure AD)
        }
    }
    
    /// Create a new Azure signer with custom authority host (for testing)
    pub fn with_authority_host(mut self, authority_host: String) -> Self {
        self.authority_host = Some(authority_host);
        self
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
        let expires_at = Utc::now() + duration;
        
        // Check for account key first (works regardless of feature flags)
        if let Some(account_key) = &self.account_key {
            let mut config = HashMap::new();
            // PyIceberg-compatible property names
            config.insert("adls.account-name".to_string(), self.account_name.clone());
            config.insert("adls.account-key".to_string(), account_key.clone());
            config.insert("adls.container".to_string(), self.container.clone());
            
            tracing::info!("✅ Using Azure account key credentials");
            
            return Ok(VendedCredentials {
                prefix: format!("abfss://{}@{}.dfs.core.windows.net/", 
                    self.container, self.account_name),
                config,
                expires_at: None, // Account keys don't expire
            });
        }
        
        // Try OAuth2 if azure-oauth feature is enabled
        #[cfg(feature = "azure-oauth")]
        {
            tracing::info!("🔑 Generating Azure OAuth2 credentials");
            
            // Try OAuth2 if credentials are available
            if let (Some(tenant_id), Some(client_id), Some(client_secret)) = 
                (&self.tenant_id, &self.client_id, &self.client_secret) {
                
                // Use custom authority host if provided (for testing), otherwise use production Azure AD
                let authority_url = if let Some(custom_host) = &self.authority_host {
                    format!("{}", custom_host)
                } else {
                    format!("https://login.microsoftonline.com/{}", tenant_id)
                };
                
                let authority_host = azure_core::Url::parse(&authority_url)
                    .map_err(|e| anyhow!("Failed to parse authority host: {}", e))?;
                
                tracing::info!("Using authority host: {}", authority_url);
                
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
                // PyIceberg-compatible property names
                config.insert("adls.token".to_string(), token.token.secret().to_string());
                config.insert("adls.account-name".to_string(), self.account_name.clone());
                config.insert("adls.container".to_string(), self.container.clone());
                
                tracing::info!("✅ Successfully generated Azure OAuth2 token");
                
                return Ok(VendedCredentials {
                    prefix: format!("abfss://{}@{}.dfs.core.windows.net/", 
                        self.container, self.account_name),
                    config,
                    expires_at: Some(expires_at),
                });
            }
        }
        
        // If we get here, no credentials are configured
        #[cfg(not(feature = "azure-oauth"))]
        {
            tracing::warn!("Azure OAuth feature not enabled and no account key provided");
        }
        
        Err(anyhow!("Azure credentials not configured properly. Provide either account_key or OAuth2 credentials (tenant_id, client_id, client_secret)"))
    }
    
    fn storage_type(&self) -> &str {
        "azure"
    }
}
