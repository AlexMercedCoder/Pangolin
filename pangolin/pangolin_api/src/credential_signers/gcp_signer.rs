use super::{CredentialSigner, VendedCredentials};
use anyhow::Result;
// Only the feature path raises errors with the macro; importing it
// unconditionally warns in a default build.
#[cfg(feature = "gcp-oauth")]
use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use std::collections::HashMap;

#[cfg(feature = "gcp-oauth")]
use gcp_auth::{CustomServiceAccount, TokenProvider};

/// GCP Cloud Storage credential signer that generates downscoped OAuth2 tokens
pub struct GcpTokenSigner {
    pub project_id: String,
    pub bucket: String,
    pub service_account_key_json: Option<String>,
}

impl GcpTokenSigner {
    pub fn new(
        project_id: String,
        bucket: String,
        service_account_key_json: Option<String>,
    ) -> Self {
        Self {
            project_id,
            bucket,
            service_account_key_json,
        }
    }
}

#[async_trait]
impl CredentialSigner for GcpTokenSigner {
    // These are read only under the feature; without the allow, a default
    // build warns on every one of them. Prefixing them with `_` instead -
    // which is what was here - meant the cfg block referenced names that did
    // not exist, so the feature could not compile at all.
    #[allow(unused_variables)]
    async fn generate_credentials(
        &self,
        resource_path: &str,
        permissions: &[String],
        duration: Duration,
    ) -> Result<VendedCredentials> {
        #[cfg(feature = "gcp-oauth")]
        {
            tracing::info!(
                "🔑 Generating GCP OAuth2 token for resource: {}",
                resource_path
            );

            if let Some(sa_key) = &self.service_account_key_json {
                let service_account = CustomServiceAccount::from_json(sa_key)
                    .map_err(|e| anyhow!("Failed to parse GCP service account key: {}", e))?;

                // Determine scopes based on permissions
                let mut scopes = vec![];
                let has_write = permissions.iter().any(|p| p == "write" || p == "delete");

                if has_write {
                    scopes.push("https://www.googleapis.com/auth/devstorage.read_write");
                } else {
                    scopes.push("https://www.googleapis.com/auth/devstorage.read_only");
                }

                let token = service_account
                    .token(&scopes)
                    .await
                    .map_err(|e| anyhow!("GCP token acquisition failed: {}", e))?;

                let expires_at = Utc::now() + duration;

                let mut config = HashMap::new();
                config.insert("credential-type".to_string(), "gcp-oauth".to_string());
                config.insert("gcp-oauth-token".to_string(), token.as_str().to_string());
                config.insert("gcp-project-id".to_string(), self.project_id.clone());
                config.insert("gcp-bucket".to_string(), self.bucket.clone());

                tracing::info!(
                    "✅ Successfully generated GCP OAuth2 token (expires: {})",
                    expires_at
                );

                return Ok(VendedCredentials {
                    prefix: format!("gs://{}/", self.bucket),
                    config,
                    expires_at: Some(expires_at),
                });
            }

            Err(anyhow!("GCP service account key not configured"))
        }

        #[cfg(not(feature = "gcp-oauth"))]
        {
            tracing::warn!("GCP OAuth feature not enabled, returning placeholder credentials");
            let mut config = HashMap::new();
            config.insert("credential-type".to_string(), "gcp-oauth".to_string());
            config.insert(
                "gcp-oauth-token".to_string(),
                "PLACEHOLDER_GCP_TOKEN".to_string(),
            );
            config.insert("gcp-project-id".to_string(), self.project_id.clone());
            config.insert("gcp-bucket".to_string(), self.bucket.clone());

            Ok(VendedCredentials {
                prefix: format!("gs://{}/", self.bucket),
                config,
                expires_at: Some(Utc::now() + duration),
            })
        }
    }

    fn storage_type(&self) -> &str {
        "gcs"
    }
}
