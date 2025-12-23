// Integration tests for credential vending
use pangolin_api::credential_signers::{
    CredentialSigner,
    azure_signer::AzureSasSigner,
    gcp_signer::GcpTokenSigner,
    s3_signer::S3Signer,
    mock_signer::MockSigner,
};
use chrono::Duration;

#[tokio::test]
async fn test_mock_azure_signer_integration() {
    let signer = MockSigner::new("azure".to_string());
    
    let result: Result<_, anyhow::Error> = signer.generate_credentials(
        "container/test-path",
        &["read".to_string(), "write".to_string()],
        Duration::hours(1),
    ).await;
    
    assert!(result.is_ok());
    let creds = result.unwrap();
    
    assert_eq!(creds.prefix, "abfss://mockcontainer@mockaccount.dfs.core.windows.net/");
    assert_eq!(creds.config.get("credential-type").unwrap(), "azure-sas");
    assert!(creds.config.contains_key("azure-sas-token"));
    assert!(creds.config.contains_key("azure-account-name"));
    assert!(creds.config.contains_key("azure-container"));
    assert!(creds.expires_at.is_some());
}

#[tokio::test]
async fn test_mock_gcp_signer_integration() {
    let signer = MockSigner::new("gcs".to_string());
    
    let result: Result<_, anyhow::Error> = signer.generate_credentials(
        "bucket/test-path",
        &["read".to_string()],
        Duration::hours(2),
    ).await;
    
    assert!(result.is_ok());
    let creds = result.unwrap();
    
    assert_eq!(creds.prefix, "gs://mock-bucket/");
    assert_eq!(creds.config.get("credential-type").unwrap(), "gcp-oauth");
    assert!(creds.config.contains_key("gcp-oauth-token"));
    assert!(creds.config.contains_key("gcp-project-id"));
    assert!(creds.config.contains_key("gcp-bucket"));
    assert!(creds.expires_at.is_some());
}

#[tokio::test]
async fn test_mock_s3_signer_integration() {
    let signer = MockSigner::new("s3".to_string());
    
    let result: Result<_, anyhow::Error> = signer.generate_credentials(
        "bucket/test-path",
        &["read".to_string(), "write".to_string(), "delete".to_string()],
        Duration::hours(1),
    ).await;
    
    assert!(result.is_ok());
    let creds = result.unwrap();
    
    assert_eq!(creds.prefix, "s3://mock-bucket/");
    assert_eq!(creds.config.get("credential-type").unwrap(), "aws-static");
    assert!(creds.config.contains_key("s3.access-key-id"));
    assert!(creds.config.contains_key("s3.secret-access-key"));
}

#[tokio::test]
async fn test_mock_signer_error_handling() {
    let signer = MockSigner::new("azure".to_string()).with_failure();
    
    let result: Result<_, anyhow::Error> = signer.generate_credentials(
        "container/test-path",
        &["read".to_string()],
        Duration::hours(1),
    ).await;
    
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.to_string(), "Mock signer configured to fail");
}

#[tokio::test]
async fn test_azure_signer_without_feature() {
    // Test Azure signer when azure-oauth feature is not enabled
    // This should return placeholder credentials
    let signer = AzureSasSigner::new(
        "testaccount".to_string(),
        Some("testkey".to_string()),
        None,
        None,
        None,
        "testcontainer".to_string(),
    );
    
    let result = signer.generate_credentials(
        "container/path",
        &["read".to_string()],
        Duration::hours(1),
    ).await;
    
    // Should succeed with placeholder credentials when feature is disabled
    assert!(result.is_ok());
    let creds = result.unwrap();
    assert_eq!(creds.config.get("credential-type").unwrap(), "azure-sas");
}

#[tokio::test]
async fn test_gcp_signer_without_feature() {
    // Test GCP signer when gcp-oauth feature is not enabled
    let signer = GcpTokenSigner::new(
        "test-project".to_string(),
        "test-bucket".to_string(),
        Some("{}".to_string()),
    );
    
    let result = signer.generate_credentials(
        "bucket/path",
        &["read".to_string(), "write".to_string()],
        Duration::hours(1),
    ).await;
    
    // Should succeed with placeholder credentials when feature is disabled
    assert!(result.is_ok());
    let creds = result.unwrap();
    assert_eq!(creds.config.get("credential-type").unwrap(), "gcp-oauth");
}

#[tokio::test]
async fn test_s3_signer_static_credentials() {
    let signer = S3Signer::new(
        None, // No role ARN
        None,
        Some("AKIAIOSFODNN7EXAMPLE".to_string()),
        Some("wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string()),
        "test-bucket".to_string(),
        Some("us-west-2".to_string()),
        None,
    );
    
    let result = signer.generate_credentials(
        "bucket/path",
        &["read".to_string()],
        Duration::hours(1),
    ).await;
    
    assert!(result.is_ok());
    let creds = result.unwrap();
    assert_eq!(creds.config.get("credential-type").unwrap(), "aws-static");
    assert_eq!(creds.config.get("s3.access-key-id").unwrap(), "AKIAIOSFODNN7EXAMPLE");
    assert_eq!(creds.config.get("s3.region").unwrap(), "us-west-2");
    assert!(creds.expires_at.is_none()); // Static credentials don't expire
}

#[tokio::test]
async fn test_credential_expiration_times() {
    let signer = MockSigner::new("azure".to_string());
    
    // Test 1 hour duration
    let result1: Result<_, anyhow::Error> = signer.generate_credentials(
        "path",
        &["read".to_string()],
        Duration::hours(1),
    ).await;
    let result1 = result1.unwrap();
    
    assert!(result1.expires_at.is_some());
    let expires1 = result1.expires_at.unwrap();
    let now = chrono::Utc::now();
    let diff = (expires1 - now).num_minutes();
    assert!(diff >= 59 && diff <= 61); // Should be ~60 minutes
    
    // Test 12 hour duration
    let result12: Result<_, anyhow::Error> = signer.generate_credentials(
        "path",
        &["read".to_string()],
        Duration::hours(12),
    ).await;
    let result12 = result12.unwrap();
    
    assert!(result12.expires_at.is_some());
    let expires12 = result12.expires_at.unwrap();
    let diff12 = (expires12 - now).num_hours();
    assert!(diff12 >= 11 && diff12 <= 13); // Should be ~12 hours
}

#[tokio::test]
async fn test_multi_cloud_credential_vending() {
    // Test that we can vend credentials for all three cloud providers
    let azure_signer = MockSigner::new("azure".to_string());
    let gcp_signer = MockSigner::new("gcs".to_string());
    let s3_signer = MockSigner::new("s3".to_string());
    
    let azure_result: Result<_, anyhow::Error> = azure_signer.generate_credentials(
        "path",
        &["read".to_string()],
        Duration::hours(1),
    ).await;
    
    let gcp_result: Result<_, anyhow::Error> = gcp_signer.generate_credentials(
        "path",
        &["read".to_string()],
        Duration::hours(1),
    ).await;
    
    let s3_result: Result<_, anyhow::Error> = s3_signer.generate_credentials(
        "path",
        &["read".to_string()],
        Duration::hours(1),
    ).await;
    
    assert!(azure_result.is_ok());
    assert!(gcp_result.is_ok());
    assert!(s3_result.is_ok());
    
    // Verify each has correct storage type
    assert_eq!(azure_signer.storage_type(), "azure");
    assert_eq!(gcp_signer.storage_type(), "gcs");
    assert_eq!(s3_signer.storage_type(), "s3");
}

#[tokio::test]
async fn test_permission_scoping() {
    let signer = MockSigner::new("gcs".to_string());
    
    // Test read-only permissions
    let read_only: Result<_, anyhow::Error> = signer.generate_credentials(
        "path",
        &["read".to_string()],
        Duration::hours(1),
    ).await;
    let read_only = read_only.unwrap();
    
    assert!(read_only.config.contains_key("gcp-oauth-token"));
    
    // Test read-write permissions
    let read_write: Result<_, anyhow::Error> = signer.generate_credentials(
        "path",
        &["read".to_string(), "write".to_string()],
        Duration::hours(1),
    ).await;
    let read_write = read_write.unwrap();
    
    assert!(read_write.config.contains_key("gcp-oauth-token"));
    
    // Both should succeed but in real implementation would have different scopes
    assert!(read_only.config.get("gcp-oauth-token").is_some());
    assert!(read_write.config.get("gcp-oauth-token").is_some());
}
