use anyhow::Result;
use std::collections::HashMap;
use aws_config::Region;
use aws_credential_types::Credentials as AwsCredentials;
use aws_sdk_sts::Client as StsClient;
use crate::signer::Credentials;

/// Creates an AWS STS Client using base credentials from the storage config.
/// Handles region selection, endpoint override, and static credential loading.
pub async fn create_sts_client(config: &HashMap<String, String>) -> Result<StsClient> {
    let access_key = config.get("s3.access-key-id")
        .or_else(|| config.get("access-key-id"))
        .ok_or_else(|| anyhow::anyhow!("Missing s3.access-key-id for base credentials"))?;
        
    let secret_key = config.get("s3.secret-access-key")
        .or_else(|| config.get("secret-access-key"))
        .ok_or_else(|| anyhow::anyhow!("Missing s3.secret-access-key for base credentials"))?;
        
    let region = config.get("s3.region")
        .or_else(|| config.get("region"))
        .map(|s| s.as_str())
        .unwrap_or("us-east-1");
        
    let endpoint = config.get("s3.endpoint")
        .or_else(|| config.get("endpoint"))
        .map(|s| s.as_str());

    // Create base credentials
    let creds = AwsCredentials::new(
        access_key.to_string(),
        secret_key.to_string(),
        None,
        None,
        "pangolin_base_provider"
    );
    
    // Load config with credentials and region
    let config_loader = aws_config::from_env()
        .region(Region::new(region.to_string()))
        .credentials_provider(creds);
        
    // Apply endpoint override if present
    let sdk_config = if let Some(ep) = endpoint {
        config_loader.endpoint_url(ep).load().await
    } else {
        config_loader.load().await
    };
    
    Ok(StsClient::new(&sdk_config))
}

/// Helper to execute AssumeRole or GetSessionToken based on role_arn presence.
/// 
/// If `role_arn` is provided, performs `assume_role`.
/// If `role_arn` is None, performs `get_session_token` (legacy behavior).
pub async fn assume_role(
    client: &StsClient,
    role_arn: Option<&str>,
    external_id: Option<&str>,
    session_name: &str
) -> Result<Credentials> {
    if let Some(arn) = role_arn {
        // Build AssumeRole request
        let mut req = client.assume_role()
            .role_arn(arn)
            .role_session_name(session_name);
            
        if let Some(ext_id) = external_id {
            req = req.external_id(ext_id);
        }
        
        let resp = req.send().await
            .map_err(|e| anyhow::anyhow!("STS AssumeRole failed: {}", e))?;
            
        let c = resp.credentials.ok_or_else(|| anyhow::anyhow!("No credentials in AssumeRole response"))?;
        
        Ok(Credentials::Aws {
            access_key_id: c.access_key_id,
            secret_access_key: c.secret_access_key,
            session_token: Some(c.session_token),
            expiration: Some(chrono::DateTime::from_timestamp(c.expiration.secs(), c.expiration.subsec_nanos()).unwrap_or_default()),
        })
    } else {
        // Fallback: GetSessionToken
        // Note: GetSessionToken does NOT support external_id, so we ignore it here.
        let resp = client.get_session_token()
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("STS GetSessionToken failed: {}", e))?;
            
        let c = resp.credentials.ok_or_else(|| anyhow::anyhow!("No credentials in GetSessionToken response"))?;
        
        Ok(Credentials::Aws {
            access_key_id: c.access_key_id,
            secret_access_key: c.secret_access_key,
            session_token: Some(c.session_token),
            expiration: Some(chrono::DateTime::from_timestamp(c.expiration.secs(), c.expiration.subsec_nanos()).unwrap_or_default()),
        })
    }
}
