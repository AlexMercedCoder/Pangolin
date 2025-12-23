use async_trait::async_trait;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use anyhow::{Result, anyhow};

/// Represents a set of vended credentials for cloud storage access
#[derive(Debug, Clone)]
pub struct VendedCredentials {
    /// The storage location prefix these credentials apply to
    pub prefix: String,
    /// Configuration map containing credential details
    pub config: HashMap<String, String>,
    /// Optional expiration time for the credentials
    pub expires_at: Option<DateTime<Utc>>,
}

/// Trait for credential signers that can vend temporary, scoped credentials
#[async_trait]
pub trait CredentialSigner: Send + Sync {
    /// Generate credentials for accessing a specific resource
    /// 
    /// # Arguments
    /// * `resource_path` - The storage path/resource to grant access to
    /// * `permissions` - List of permissions to grant (e.g., ["read", "write"])
    /// * `duration` - How long the credentials should be valid
    /// 
    /// # Returns
    /// A `VendedCredentials` struct containing the credentials and metadata
    async fn generate_credentials(
        &self,
        resource_path: &str,
        permissions: &[String],
        duration: Duration,
    ) -> Result<VendedCredentials>;
    
    /// Get the storage type this signer handles (e.g., "s3", "azure", "gcs")
    fn storage_type(&self) -> &str;
}

// Re-export submodules
pub mod azure_signer;
pub mod gcp_signer;
pub mod s3_signer;
pub mod mock_signer;
