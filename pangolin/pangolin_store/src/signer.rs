use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Credentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: Option<String>,
    pub expiration: Option<String>,
}

#[async_trait]
pub trait Signer: Send + Sync {
    /// Get temporary credentials for accessing a specific table location.
    /// In a real implementation, this might scope permissions to that prefix.
    async fn get_table_credentials(&self, location: &str) -> Result<Credentials>;

    /// Generate a presigned URL for a specific file location.
    async fn presign_get(&self, location: &str) -> Result<String>;
}
