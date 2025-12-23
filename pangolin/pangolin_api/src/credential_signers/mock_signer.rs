use super::{CredentialSigner, VendedCredentials};
use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use anyhow::Result;

/// Mock credential signer for testing
pub struct MockSigner {
    pub storage_type: String,
    pub should_fail: bool,
    pub mock_token: String,
}

impl MockSigner {
    pub fn new(storage_type: String) -> Self {
        Self {
            storage_type,
            should_fail: false,
            mock_token: "mock-token-12345".to_string(),
        }
    }
    
    pub fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
    
    pub fn with_token(mut self, token: String) -> Self {
        self.mock_token = token;
        self
    }
}

#[async_trait]
impl CredentialSigner for MockSigner {
    async fn generate_credentials(
        &self,
        resource_path: &str,
        permissions: &[String],
        duration: Duration,
    ) -> Result<VendedCredentials> {
        if self.should_fail {
            return Err(anyhow::anyhow!("Mock signer configured to fail"));
        }
        
        let expires_at = Utc::now() + duration;
        let mut config = HashMap::new();
        
        match self.storage_type.as_str() {
            "azure" => {
                config.insert("credential-type".to_string(), "azure-sas".to_string());
                config.insert("azure-sas-token".to_string(), self.mock_token.clone());
                config.insert("azure-account-name".to_string(), "mockaccount".to_string());
                config.insert("azure-container".to_string(), "mockcontainer".to_string());
                
                Ok(VendedCredentials {
                    prefix: "abfss://mockcontainer@mockaccount.dfs.core.windows.net/".to_string(),
                    config,
                    expires_at: Some(expires_at),
                })
            },
            "gcs" => {
                config.insert("credential-type".to_string(), "gcp-oauth".to_string());
                config.insert("gcp-oauth-token".to_string(), self.mock_token.clone());
                config.insert("gcp-project-id".to_string(), "mock-project".to_string());
                config.insert("gcp-bucket".to_string(), "mock-bucket".to_string());
                
                Ok(VendedCredentials {
                    prefix: "gs://mock-bucket/".to_string(),
                    config,
                    expires_at: Some(expires_at),
                })
            },
            "s3" => {
                config.insert("credential-type".to_string(), "aws-static".to_string());
                config.insert("s3.access-key-id".to_string(), "MOCK_ACCESS_KEY".to_string());
                config.insert("s3.secret-access-key".to_string(), "MOCK_SECRET_KEY".to_string());
                
                Ok(VendedCredentials {
                    prefix: "s3://mock-bucket/".to_string(),
                    config,
                    expires_at: Some(expires_at),
                })
            },
            _ => Err(anyhow::anyhow!("Unknown storage type: {}", self.storage_type)),
        }
    }
    
    fn storage_type(&self) -> &str {
        &self.storage_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mock_azure_signer_success() {
        let signer = MockSigner::new("azure".to_string());
        let result = signer.generate_credentials(
            "container/path",
            &["read".to_string()],
            Duration::hours(1),
        ).await;
        
        assert!(result.is_ok());
        let creds = result.unwrap();
        assert_eq!(creds.config.get("credential-type").unwrap(), "azure-sas");
        assert!(creds.config.contains_key("azure-sas-token"));
        assert!(creds.expires_at.is_some());
    }
    
    #[tokio::test]
    async fn test_mock_gcp_signer_success() {
        let signer = MockSigner::new("gcs".to_string());
        let result = signer.generate_credentials(
            "bucket/path",
            &["read".to_string(), "write".to_string()],
            Duration::hours(1),
        ).await;
        
        assert!(result.is_ok());
        let creds = result.unwrap();
        assert_eq!(creds.config.get("credential-type").unwrap(), "gcp-oauth");
        assert!(creds.config.contains_key("gcp-oauth-token"));
    }
    
    #[tokio::test]
    async fn test_mock_s3_signer_success() {
        let signer = MockSigner::new("s3".to_string());
        let result = signer.generate_credentials(
            "bucket/path",
            &["read".to_string()],
            Duration::hours(1),
        ).await;
        
        assert!(result.is_ok());
        let creds = result.unwrap();
        assert_eq!(creds.config.get("credential-type").unwrap(), "aws-static");
    }
    
    #[tokio::test]
    async fn test_mock_signer_failure() {
        let signer = MockSigner::new("azure".to_string()).with_failure();
        let result = signer.generate_credentials(
            "container/path",
            &["read".to_string()],
            Duration::hours(1),
        ).await;
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Mock signer configured to fail");
    }
    
    #[tokio::test]
    async fn test_mock_signer_custom_token() {
        let custom_token = "custom-test-token-xyz";
        let signer = MockSigner::new("azure".to_string())
            .with_token(custom_token.to_string());
        
        let result = signer.generate_credentials(
            "container/path",
            &["read".to_string()],
            Duration::hours(1),
        ).await;
        
        assert!(result.is_ok());
        let creds = result.unwrap();
        assert_eq!(creds.config.get("azure-sas-token").unwrap(), custom_token);
    }
}
