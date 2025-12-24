use pangolin_api::iceberg_handlers::TableResponse;
use pangolin_core::iceberg_metadata::{TableMetadata, Schema, PartitionSpec, SortOrder};
use std::collections::HashMap;

/// Test that TableResponse::with_credentials correctly merges S3 credentials
#[test]
fn test_table_response_with_s3_credentials() {
    let mut creds = HashMap::new();
    creds.insert("s3.access-key-id".to_string(), "AKIATEST123".to_string());
    creds.insert("s3.secret-access-key".to_string(), "secretkey123".to_string());
    creds.insert("s3.session-token".to_string(), "sessiontoken123".to_string());

    let metadata = create_test_metadata();
    let response = TableResponse::with_credentials(
        Some("s3://test-bucket/metadata.json".to_string()),
        metadata,
        Some(creds),
        None,  // asset_id
    );

    let config = response.config.expect("Config should be present");
    
    // Verify S3 credentials are present
    assert_eq!(config.get("s3.access-key-id"), Some(&"AKIATEST123".to_string()));
    assert_eq!(config.get("s3.secret-access-key"), Some(&"secretkey123".to_string()));
    assert_eq!(config.get("s3.session-token"), Some(&"sessiontoken123".to_string()));
    
    // Verify defaults are present
    assert!(config.contains_key("s3.endpoint"));
    assert!(config.contains_key("s3.region"));
}

/// Test that TableResponse::with_credentials correctly merges Azure credentials
#[test]
fn test_table_response_with_azure_credentials() {
    let mut creds = HashMap::new();
    creds.insert("adls.account-name".to_string(), "mystorageaccount".to_string());
    creds.insert("adls.account-key".to_string(), "accountkey123".to_string());
    creds.insert("adls.sas-token".to_string(), "sastoken123".to_string());

    let metadata = create_test_metadata();
    let response = TableResponse::with_credentials(
        Some("abfs://container@account.dfs.core.windows.net/path".to_string()),
        metadata,
        Some(creds),
        None,  // asset_id
    );

    let config = response.config.expect("Config should be present");
    
    // Verify Azure credentials are present
    assert_eq!(config.get("adls.account-name"), Some(&"mystorageaccount".to_string()));
    assert_eq!(config.get("adls.account-key"), Some(&"accountkey123".to_string()));
    assert_eq!(config.get("adls.sas-token"), Some(&"sastoken123".to_string()));
    
    // Verify S3 defaults are still present (for backward compatibility)
    assert!(config.contains_key("s3.endpoint"));
    assert!(config.contains_key("s3.region"));
}

/// Test that TableResponse::with_credentials correctly merges GCS credentials
#[test]
fn test_table_response_with_gcs_credentials() {
    let mut creds = HashMap::new();
    creds.insert("gcs.project-id".to_string(), "my-gcp-project".to_string());
    creds.insert("gcs.service-account-file".to_string(), "/path/to/sa.json".to_string());
    creds.insert("gcs.oauth2.token".to_string(), "oauth2token123".to_string());

    let metadata = create_test_metadata();
    let response = TableResponse::with_credentials(
        Some("gs://my-bucket/path".to_string()),
        metadata,
        Some(creds),
        None,  // asset_id
    );

    let config = response.config.expect("Config should be present");
    
    // Verify GCS credentials are present
    assert_eq!(config.get("gcs.project-id"), Some(&"my-gcp-project".to_string()));
    assert_eq!(config.get("gcs.service-account-file"), Some(&"/path/to/sa.json".to_string()));
    assert_eq!(config.get("gcs.oauth2.token"), Some(&"oauth2token123".to_string()));
    
    // Verify S3 defaults are still present (for backward compatibility)
    assert!(config.contains_key("s3.endpoint"));
    assert!(config.contains_key("s3.region"));
}

/// Test that TableResponse::with_credentials handles None credentials correctly
#[test]
fn test_table_response_with_no_credentials() {
    let metadata = create_test_metadata();
    let response = TableResponse::with_credentials(
        Some("s3://test-bucket/metadata.json".to_string()),
        metadata,
        None,
        None,  // asset_id
    );

    let config = response.config.expect("Config should be present");
    
    // Verify only defaults are present
    assert!(config.contains_key("s3.endpoint"));
    assert!(config.contains_key("s3.region"));
    assert!(!config.contains_key("s3.access-key-id"));
    assert!(!config.contains_key("s3.secret-access-key"));
}

/// Test that TableResponse::with_credentials doesn't override provided credentials with defaults
#[test]
fn test_table_response_credentials_override_defaults() {
    let mut creds = HashMap::new();
    creds.insert("s3.access-key-id".to_string(), "AKIATEST123".to_string());
    creds.insert("s3.secret-access-key".to_string(), "secretkey123".to_string());
    creds.insert("s3.endpoint".to_string(), "http://custom-endpoint:9000".to_string());
    creds.insert("s3.region".to_string(), "eu-west-1".to_string());

    let metadata = create_test_metadata();
    let response = TableResponse::with_credentials(
        Some("s3://test-bucket/metadata.json".to_string()),
        metadata,
        Some(creds),
        None,  // asset_id
    );

    let config = response.config.expect("Config should be present");
    
    // Verify custom endpoint and region are preserved (not overridden by defaults)
    assert_eq!(config.get("s3.endpoint"), Some(&"http://custom-endpoint:9000".to_string()));
    assert_eq!(config.get("s3.region"), Some(&"eu-west-1".to_string()));
}

/// Test that TableResponse::with_credentials handles mixed cloud credentials
#[test]
fn test_table_response_with_mixed_credentials() {
    let mut creds = HashMap::new();
    // S3 credentials
    creds.insert("s3.access-key-id".to_string(), "AKIATEST123".to_string());
    creds.insert("s3.secret-access-key".to_string(), "secretkey123".to_string());
    // Azure credentials
    creds.insert("adls.account-name".to_string(), "mystorageaccount".to_string());
    creds.insert("adls.account-key".to_string(), "accountkey123".to_string());

    let metadata = create_test_metadata();
    let response = TableResponse::with_credentials(
        Some("s3://test-bucket/metadata.json".to_string()),
        metadata,
        Some(creds),
        None,  // asset_id
    );

    let config = response.config.expect("Config should be present");
    
    // Verify both S3 and Azure credentials are present
    assert_eq!(config.get("s3.access-key-id"), Some(&"AKIATEST123".to_string()));
    assert_eq!(config.get("s3.secret-access-key"), Some(&"secretkey123".to_string()));
    assert_eq!(config.get("adls.account-name"), Some(&"mystorageaccount".to_string()));
    assert_eq!(config.get("adls.account-key"), Some(&"accountkey123".to_string()));
}

/// Helper function to create test metadata
fn create_test_metadata() -> TableMetadata {
    TableMetadata {
        format_version: 2,
        table_uuid: uuid::Uuid::new_v4(),
        location: "s3://test-bucket/table".to_string(),
        last_updated_ms: 1234567890,
        last_column_id: 1,
        schemas: vec![Schema {
            schema_id: 0,
            fields: vec![],
            identifier_field_ids: None,
        }],
        current_schema_id: 0,
        partition_specs: vec![PartitionSpec {
            spec_id: 0,
            fields: vec![],
        }],
        properties: Some(HashMap::new()),
        current_snapshot_id: None,
        snapshots: None,
        snapshot_log: None,
        metadata_log: None,
        sort_orders: vec![SortOrder {
            order_id: 0,
            fields: vec![],
        }],
        default_sort_order_id: 0,
        last_sequence_number: 0,
        current_partition_spec_id: 0,
    }
}
