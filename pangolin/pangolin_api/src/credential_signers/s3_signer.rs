use super::{CredentialSigner, VendedCredentials};
use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use anyhow::{Result, anyhow};

#[cfg(feature = "aws-sts")]
use aws_sdk_sts::Client as StsClient;
#[cfg(feature = "aws-sts")]
use aws_config;

/// AWS S3 credential signer that generates temporary credentials via STS AssumeRole
pub struct S3Signer {
    pub role_arn: Option<String>,
    pub external_id: Option<String>,
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
    pub bucket: String,
    pub region: Option<String>,
    pub endpoint: Option<String>,
}

impl S3Signer {
    pub fn new(
        role_arn: Option<String>,
        external_id: Option<String>,
        access_key: Option<String>,
        secret_key: Option<String>,
        bucket: String,
        region: Option<String>,
        endpoint: Option<String>,
    ) -> Self {
        Self {
            role_arn,
            external_id,
            access_key,
            secret_key,
            bucket,
            region,
            endpoint,
        }
    }
}

#[async_trait]
impl CredentialSigner for S3Signer {
    async fn generate_credentials(
        &self,
        _resource_path: &str,
        _permissions: &[String],
        duration: Duration,
    ) -> Result<VendedCredentials> {
        #[cfg(feature = "aws-sts")]
        {
            if let Some(role_arn) = &self.role_arn {
                tracing::info!("🔑 Assuming AWS role: {}", role_arn);
                
                let config = aws_config::load_from_env().await;
                let sts_client = StsClient::new(&config);
                
                let session_name = format!("pangolin-{}", chrono::Utc::now().timestamp());
                let duration_secs = duration.num_seconds().min(3600).max(900) as i32;
                
                let mut request = sts_client
                    .assume_role()
                    .role_arn(role_arn)
                    .role_session_name(&session_name)
                    .duration_seconds(duration_secs);
                
                if let Some(ext_id) = &self.external_id {
                    request = request.external_id(ext_id);
                }
                
                let response = request.send().await
                    .map_err(|e| anyhow!("STS AssumeRole failed: {}", e))?;
                
                let creds = response.credentials()
                    .ok_or_else(|| anyhow!("No credentials returned from STS"))?;
                
                let mut config = HashMap::new();
                config.insert("credential-type".to_string(), "aws-sts".to_string());
                config.insert("s3.access-key-id".to_string(), creds.access_key_id().to_string());
                config.insert("s3.secret-access-key".to_string(), creds.secret_access_key().to_string());
                config.insert("s3.session-token".to_string(), creds.session_token().to_string());
                
                if let Some(region) = &self.region {
                    config.insert("s3.region".to_string(), region.clone());
                }
                if let Some(endpoint) = &self.endpoint {
                    config.insert("s3.endpoint".to_string(), endpoint.clone());
                }
                
                let expires_at = chrono::DateTime::parse_from_rfc3339(creds.expiration())
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc));
                
                tracing::info!("✅ Successfully assumed AWS role");
                
                return Ok(VendedCredentials {
                    prefix: format!("s3://{}/", self.bucket),
                    config,
                    expires_at,
                });
            }
        }
        
        // Fallback to static credentials
        if let (Some(access_key), Some(secret_key)) = (&self.access_key, &self.secret_key) {
            let mut config = HashMap::new();
            config.insert("credential-type".to_string(), "aws-static".to_string());
            config.insert("s3.access-key-id".to_string(), access_key.clone());
            config.insert("s3.secret-access-key".to_string(), secret_key.clone());
            
            if let Some(region) = &self.region {
                config.insert("s3.region".to_string(), region.clone());
            }
            if let Some(endpoint) = &self.endpoint {
                config.insert("s3.endpoint".to_string(), endpoint.clone());
            }
            
            tracing::info!("✅ Using static AWS credentials");
            
            return Ok(VendedCredentials {
                prefix: format!("s3://{}/", self.bucket),
                config,
                expires_at: None,
            });
        }
        
        Err(anyhow!("AWS credentials not configured"))
    }
    
    fn storage_type(&self) -> &str {
        "s3"
    }
}
