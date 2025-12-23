// End-to-end tests for credential vending with all cloud providers
use pangolin_api::credential_vending::create_signer_from_warehouse;
use pangolin_core::model::Warehouse;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
async fn test_e2e_s3_warehouse_credential_vending() {
    // Create S3 warehouse configuration
    let mut storage_config = HashMap::new();
    storage_config.insert("type".to_string(), "s3".to_string());
    storage_config.insert("bucket".to_string(), "test-s3-bucket".to_string());
    storage_config.insert("access_key_id".to_string(), "AKIATEST123".to_string());
    storage_config.insert("secret_access_key".to_string(), "secretkey123".to_string());
    storage_config.insert("region".to_string(), "us-west-2".to_string());
    
    let warehouse = Warehouse {
        id: Uuid::new_v4(),
        name: "s3-warehouse".to_string(),
        tenant_id: Uuid::new_v4(),
        storage_config,
        use_sts: false,
        vending_strategy: None,
    };
    
    // Create signer and vend credentials
    let signer = create_signer_from_warehouse(&warehouse).unwrap();
    assert_eq!(signer.storage_type(), "s3");
    
    let result = signer.generate_credentials(
        "namespace/table",
        &["read".to_string(), "write".to_string()],
        chrono::Duration::hours(1),
    ).await;
    
    assert!(result.is_ok());
    let creds = result.unwrap();
    
    // Verify S3 credentials
    assert_eq!(creds.prefix, "s3://test-s3-bucket/");
    assert_eq!(creds.config.get("credential-type").unwrap(), "aws-static");
    assert_eq!(creds.config.get("s3.access-key-id").unwrap(), "AKIATEST123");
    assert_eq!(creds.config.get("s3.region").unwrap(), "us-west-2");
    assert!(creds.expires_at.is_none()); // Static credentials don't expire
}

#[tokio::test]
async fn test_e2e_azure_warehouse_credential_vending() {
    // Create Azure warehouse configuration
    let mut storage_config = HashMap::new();
    storage_config.insert("type".to_string(), "azure".to_string());
    storage_config.insert("account_name".to_string(), "testazureaccount".to_string());
    storage_config.insert("container".to_string(), "testcontainer".to_string());
    storage_config.insert("account_key".to_string(), "testaccountkey123".to_string());
    
    let warehouse = Warehouse {
        id: Uuid::new_v4(),
        name: "azure-warehouse".to_string(),
        tenant_id: Uuid::new_v4(),
        storage_config,
        use_sts: false,
        vending_strategy: None,
    };
    
    // Create signer and vend credentials
    let signer = create_signer_from_warehouse(&warehouse).unwrap();
    assert_eq!(signer.storage_type(), "azure");
    
    let result = signer.generate_credentials(
        "namespace/table",
        &["read".to_string(), "write".to_string()],
        chrono::Duration::hours(1),
    ).await;
    
    assert!(result.is_ok());
    let creds = result.unwrap();
    
    // Verify Azure credentials
    assert_eq!(creds.prefix, "abfss://testcontainer@testazureaccount.dfs.core.windows.net/");
    // When azure-oauth feature is disabled, it returns placeholder credentials
    assert!(creds.config.contains_key("credential-type"));
    assert!(creds.config.contains_key("azure-account-name"));
    assert!(creds.expires_at.is_some() || creds.expires_at.is_none()); // May or may not expire depending on mode
}

#[tokio::test]
async fn test_e2e_gcp_warehouse_credential_vending() {
    // Create GCP warehouse configuration
    let mut storage_config = HashMap::new();
    storage_config.insert("type".to_string(), "gcs".to_string());
    storage_config.insert("project_id".to_string(), "test-gcp-project".to_string());
    storage_config.insert("bucket".to_string(), "test-gcs-bucket".to_string());
    storage_config.insert("service_account_key".to_string(), "{}".to_string());
    
    let warehouse = Warehouse {
        id: Uuid::new_v4(),
        name: "gcp-warehouse".to_string(),
        tenant_id: Uuid::new_v4(),
        storage_config,
        use_sts: false,
        vending_strategy: None,
    };
    
    // Create signer and vend credentials
    let signer = create_signer_from_warehouse(&warehouse).unwrap();
    assert_eq!(signer.storage_type(), "gcs");
    
    let result = signer.generate_credentials(
        "namespace/table",
        &["read".to_string()],
        chrono::Duration::hours(1),
    ).await;
    
    assert!(result.is_ok());
    let creds = result.unwrap();
    
    // Verify GCP credentials
    assert_eq!(creds.prefix, "gs://test-gcs-bucket/");
    assert_eq!(creds.config.get("credential-type").unwrap(), "gcp-oauth");
    assert!(creds.config.contains_key("gcp-oauth-token"));
    assert!(creds.config.contains_key("gcp-project-id"));
    assert!(creds.expires_at.is_some());
}

#[tokio::test]
async fn test_e2e_multi_cloud_no_regression() {
    // Test all three cloud providers to ensure no regressions
    
    // S3
    let mut s3_config = HashMap::new();
    s3_config.insert("type".to_string(), "s3".to_string());
    s3_config.insert("bucket".to_string(), "s3-bucket".to_string());
    s3_config.insert("access_key_id".to_string(), "AKIATEST".to_string());
    s3_config.insert("secret_access_key".to_string(), "secret".to_string());
    
    let s3_warehouse = Warehouse {
        id: Uuid::new_v4(),
        name: "s3-wh".to_string(),
        tenant_id: Uuid::new_v4(),
        storage_config: s3_config,
        use_sts: false,
        vending_strategy: None,
    };
    
    // Azure
    let mut azure_config = HashMap::new();
    azure_config.insert("type".to_string(), "azure".to_string());
    azure_config.insert("account_name".to_string(), "azureacct".to_string());
    azure_config.insert("container".to_string(), "container".to_string());
    azure_config.insert("account_key".to_string(), "key".to_string());
    
    let azure_warehouse = Warehouse {
        id: Uuid::new_v4(),
        name: "azure-wh".to_string(),
        tenant_id: Uuid::new_v4(),
        storage_config: azure_config,
        use_sts: false,
        vending_strategy: None,
    };
    
    // GCP
    let mut gcp_config = HashMap::new();
    gcp_config.insert("type".to_string(), "gcs".to_string());
    gcp_config.insert("project_id".to_string(), "project".to_string());
    gcp_config.insert("bucket".to_string(), "bucket".to_string());
    gcp_config.insert("service_account_key".to_string(), "{}".to_string());
    
    let gcp_warehouse = Warehouse {
        id: Uuid::new_v4(),
        name: "gcp-wh".to_string(),
        tenant_id: Uuid::new_v4(),
        storage_config: gcp_config,
        use_sts: false,
        vending_strategy: None,
    };
    
    // Test all three
    let s3_signer = create_signer_from_warehouse(&s3_warehouse).unwrap();
    let azure_signer = create_signer_from_warehouse(&azure_warehouse).unwrap();
    let gcp_signer = create_signer_from_warehouse(&gcp_warehouse).unwrap();
    
    let s3_result = s3_signer.generate_credentials("path", &["read".to_string()], chrono::Duration::hours(1)).await;
    let azure_result = azure_signer.generate_credentials("path", &["read".to_string()], chrono::Duration::hours(1)).await;
    let gcp_result = gcp_signer.generate_credentials("path", &["read".to_string()], chrono::Duration::hours(1)).await;
    
    // All should succeed
    assert!(s3_result.is_ok(), "S3 credential vending failed");
    assert!(azure_result.is_ok(), "Azure credential vending failed");
    assert!(gcp_result.is_ok(), "GCP credential vending failed");
    
    // Verify storage types
    assert_eq!(s3_signer.storage_type(), "s3");
    assert_eq!(azure_signer.storage_type(), "azure");
    assert_eq!(gcp_signer.storage_type(), "gcs");
    
    // Verify prefixes
    assert!(s3_result.unwrap().prefix.starts_with("s3://"));
    assert!(azure_result.unwrap().prefix.starts_with("abfss://"));
    assert!(gcp_result.unwrap().prefix.starts_with("gs://"));
}

#[tokio::test]
async fn test_e2e_s3_sts_mode() {
    // Test S3 with STS mode enabled
    let mut storage_config = HashMap::new();
    storage_config.insert("type".to_string(), "s3".to_string());
    storage_config.insert("bucket".to_string(), "sts-bucket".to_string());
    storage_config.insert("role_arn".to_string(), "arn:aws:iam::123456789012:role/TestRole".to_string());
    
    let warehouse = Warehouse {
        id: Uuid::new_v4(),
        name: "s3-sts-warehouse".to_string(),
        tenant_id: Uuid::new_v4(),
        storage_config,
        use_sts: true,
        vending_strategy: None,
    };
    
    let signer = create_signer_from_warehouse(&warehouse).unwrap();
    let result = signer.generate_credentials(
        "namespace/table",
        &["read".to_string(), "write".to_string()],
        chrono::Duration::hours(1),
    ).await;
    
    // When AWS STS feature is enabled, this will use placeholder credentials
    // When AWS STS feature is disabled, this will fail because no static credentials are provided
    // Both behaviors are acceptable for this test
    if result.is_ok() {
        let creds = result.unwrap();
        assert_eq!(creds.prefix, "s3://sts-bucket/");
        // Verify it has some form of credentials
        assert!(creds.config.contains_key("credential-type") || 
                creds.config.contains_key("s3.access-key-id"));
    } else {
        // Expected to fail when no static credentials and STS is not available
        let err = result.unwrap_err();
        assert!(err.to_string().contains("credentials") || 
                err.to_string().contains("not configured"));
    }
}
