use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Credentials {
    // AWS S3 credentials (STS or Static)
    Aws {
        access_key_id: String,
        secret_access_key: String,
        session_token: Option<String>,
        expiration: Option<DateTime<Utc>>,
    },
    // Azure SAS token
    Azure {
        sas_token: String,
        account_name: String, // Added account_name as it's often needed by clients
        expiration: DateTime<Utc>,
    },
    // GCP OAuth2 token
    Gcp {
        access_token: String,
        expiration: DateTime<Utc>,
    },
}

#[async_trait]
pub trait Signer: Send + Sync {
    /// Get temporary credentials for accessing a specific table location.
    /// In a real implementation, this might scope permissions to that prefix.
    async fn get_table_credentials(&self, location: &str) -> Result<Credentials>;

    /// Generate a presigned URL for a specific file location.
    async fn presign_get(&self, location: &str) -> Result<String>;
}

#[derive(Clone)]
pub struct SignerImpl {
    key: String,
}

impl SignerImpl {
    pub fn new(key: String) -> Self {
        Self { key }
    }
}

#[async_trait]
impl Signer for SignerImpl {
    async fn get_table_credentials(&self, _location: &str) -> Result<Credentials> {
        Err(anyhow::anyhow!("Credential vending not supported by bare SignerImpl"))
    }

    async fn presign_get(&self, _location: &str) -> Result<String> {
        Err(anyhow::anyhow!("Presigning not supported by bare SignerImpl"))
    }
}
