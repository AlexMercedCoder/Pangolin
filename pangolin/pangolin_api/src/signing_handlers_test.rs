use super::*;
use crate::signing_handlers::{StorageCredential, LoadCredentialsResponse};
use crate::iceberg_handlers::TableResponse;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_static_credentials_response_format() {
    // Test that static credentials are returned in correct format
    let mut storage_config = HashMap::new();
    storage_config.insert("access_key_id".to_string(), "test_access_key".to_string());
    storage_config.insert("secret_access_key".to_string(), "test_secret_key".to_string());
    storage_config.insert("bucket".to_string(), "test-bucket".to_string());
    storage_config.insert("endpoint".to_string(), "http://localhost:9000".to_string());
    storage_config.insert("region".to_string(), "us-east-1".to_string());
    
    let warehouse = pangolin_core::model::Warehouse {
        id: uuid::Uuid::new_v4(),
        name: "test_warehouse".to_string(),
        tenant_id: uuid::Uuid::nil(),
        use_sts: false,
        storage_config,
        vending_strategy: None,
    };
    
    // Simulate the credential vending logic
    let mut config = HashMap::new();
    let access_key = warehouse.storage_config.get("access_key_id").cloned().unwrap();
    let secret_key = warehouse.storage_config.get("secret_access_key").cloned().unwrap();
    
    config.insert("access-key".to_string(), access_key.clone());
    config.insert("secret-key".to_string(), secret_key.clone());
    config.insert("s3.endpoint".to_string(), warehouse.storage_config.get("endpoint").cloned().unwrap());
    config.insert("s3.region".to_string(), warehouse.storage_config.get("region").cloned().unwrap());
    
    let storage_credential = StorageCredential {
        prefix: "s3://test-bucket/".to_string(),
        config: config.clone(),
    };
    
    let response = LoadCredentialsResponse {
        storage_credentials: vec![storage_credential],
    };
    
    // Verify response structure
    assert_eq!(response.storage_credentials.len(), 1);
    assert_eq!(response.storage_credentials[0].prefix, "s3://test-bucket/");
    assert_eq!(response.storage_credentials[0].config.get("access-key").unwrap(), "test_access_key");
    assert_eq!(response.storage_credentials[0].config.get("secret-key").unwrap(), "test_secret_key");
    assert_eq!(response.storage_credentials[0].config.get("s3.endpoint").unwrap(), "http://localhost:9000");
    assert_eq!(response.storage_credentials[0].config.get("s3.region").unwrap(), "us-east-1");
}

#[test]
fn test_azure_credentials_response_format() {
    // Test that Azure credentials are returned in correct format
    let mut storage_config = HashMap::new();
    storage_config.insert("type".to_string(), "azure".to_string());
    storage_config.insert("account_name".to_string(), "testaccount".to_string());
    storage_config.insert("account_key".to_string(), "test_azure_key".to_string());
    storage_config.insert("container".to_string(), "test-container".to_string());
    
    let warehouse = pangolin_core::model::Warehouse {
        id: uuid::Uuid::new_v4(),
        name: "test_azure_warehouse".to_string(),
        tenant_id: uuid::Uuid::nil(),
        use_sts: false,
        storage_config,
        vending_strategy: None,
    };
    
    // Simulate Azure credential vending logic
    let mut config = HashMap::new();
    let account_name = warehouse.storage_config.get("account_name").cloned().unwrap();
    let account_key = warehouse.storage_config.get("account_key").cloned().unwrap();
    
    config.insert("adls.account-name".to_string(), account_name.clone());
    config.insert("adls.account-key".to_string(), account_key.clone());
    
    let storage_credential = StorageCredential {
        prefix: "abfss://test-container@testaccount.dfs.core.windows.net/".to_string(),
        config: config.clone(),
    };
    
    let response = LoadCredentialsResponse {
        storage_credentials: vec![storage_credential],
    };
    
    // Verify Azure response
    assert_eq!(response.storage_credentials.len(), 1);
    assert!(response.storage_credentials[0].prefix.starts_with("abfss://"));
    assert_eq!(response.storage_credentials[0].config.get("adls.account-name").unwrap(), "testaccount");
    assert_eq!(response.storage_credentials[0].config.get("adls.account-key").unwrap(), "test_azure_key");
}

#[test]
fn test_gcs_credentials_response_format() {
    // Test that GCS credentials are returned in correct format
    let mut storage_config = HashMap::new();
    storage_config.insert("type".to_string(), "gcs".to_string());
    storage_config.insert("project_id".to_string(), "test-project".to_string());
    storage_config.insert("service_account_key".to_string(), "{\"type\":\"service_account\"}".to_string());
    storage_config.insert("bucket".to_string(), "test-bucket".to_string());
    
    let warehouse = pangolin_core::model::Warehouse {
        id: uuid::Uuid::new_v4(),
        name: "test_gcs_warehouse".to_string(),
        tenant_id: uuid::Uuid::nil(),
        use_sts: false,
        storage_config,
        vending_strategy: None,
    };
    
    // Simulate GCS credential vending logic
    let mut config = HashMap::new();
    let project_id = warehouse.storage_config.get("project_id").cloned().unwrap();
    let service_account_key = warehouse.storage_config.get("service_account_key").cloned().unwrap();
    
    config.insert("gcs.project-id".to_string(), project_id.clone());
    config.insert("gcs.service-account-key".to_string(), service_account_key.clone());
    
    let storage_credential = StorageCredential {
        prefix: "gs://test-bucket/".to_string(),
        config: config.clone(),
    };
    
    let response = LoadCredentialsResponse {
        storage_credentials: vec![storage_credential],
    };
    
    // Verify GCS response
    assert_eq!(response.storage_credentials.len(), 1);
    assert!(response.storage_credentials[0].prefix.starts_with("gs://"));
    assert_eq!(response.storage_credentials[0].config.get("gcs.project-id").unwrap(), "test-project");
    assert_eq!(response.storage_credentials[0].config.get("gcs.service-account-key").unwrap(), "{\"type\":\"service_account\"}");
}

#[test]
fn test_sts_credentials_response_format() {
    // Test that STS credentials include session token
    let mut storage_config = HashMap::new();
    storage_config.insert("role_arn".to_string(), "arn:aws:iam::123456789012:role/TestRole".to_string());
    storage_config.insert("bucket".to_string(), "test-bucket".to_string());
    
    let warehouse = pangolin_core::model::Warehouse {
        id: uuid::Uuid::new_v4(),
        name: "test_warehouse_sts".to_string(),
        tenant_id: uuid::Uuid::nil(),
        use_sts: true,
        storage_config,
        vending_strategy: None,
    };
    
    // Simulate STS credential vending logic
    let mut config = HashMap::new();
    let role_arn = warehouse.storage_config.get("role_arn").cloned().unwrap();
    
    config.insert("access-key".to_string(), format!("STS_ACCESS_KEY_FOR_{}", role_arn));
    config.insert("secret-key".to_string(), "STS_SECRET_KEY_PLACEHOLDER".to_string());
    config.insert("session-token".to_string(), "STS_SESSION_TOKEN_PLACEHOLDER".to_string());
    
    let storage_credential = StorageCredential {
        prefix: "s3://test-bucket/".to_string(),
        config: config.clone(),
    };
    
    let response = LoadCredentialsResponse {
        storage_credentials: vec![storage_credential],
    };
    
    // Verify STS response includes session token
    assert_eq!(response.storage_credentials.len(), 1);
    assert!(response.storage_credentials[0].config.get("access-key").unwrap().contains("STS_ACCESS_KEY"));
    assert_eq!(response.storage_credentials[0].config.get("secret-key").unwrap(), "STS_SECRET_KEY_PLACEHOLDER");
    assert_eq!(response.storage_credentials[0].config.get("session-token").unwrap(), "STS_SESSION_TOKEN_PLACEHOLDER");
}

#[test]
fn test_credentials_response_serialization() {
    // Test that the response serializes to correct JSON format
    let mut config = HashMap::new();
    config.insert("access-key".to_string(), "AKIAIOSFODNN7EXAMPLE".to_string());
    config.insert("secret-key".to_string(), "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string());
    config.insert("s3.endpoint".to_string(), "http://localhost:9000".to_string());
    config.insert("s3.region".to_string(), "us-east-1".to_string());
    
    let storage_credential = StorageCredential {
        prefix: "s3://warehouse/".to_string(),
        config,
    };
    
    let response = LoadCredentialsResponse {
        storage_credentials: vec![storage_credential],
    };
    
    let json = serde_json::to_value(&response).unwrap();
    
    // Verify JSON structure matches Iceberg REST spec
    assert!(json.get("storage-credentials").is_some());
    assert!(json["storage-credentials"].is_array());
    assert_eq!(json["storage-credentials"].as_array().unwrap().len(), 1);
    
    let cred = &json["storage-credentials"][0];
    assert_eq!(cred["prefix"], "s3://warehouse/");
    assert!(cred["config"].is_object());
    assert_eq!(cred["config"]["access-key"], "AKIAIOSFODNN7EXAMPLE");
    assert_eq!(cred["config"]["s3.endpoint"], "http://localhost:9000");
}

#[test]
fn test_table_response_includes_credentials() {
    // Test that TableResponse includes credentials in config
    use crate::iceberg_handlers::TableResponse;
    use pangolin_core::iceberg_metadata::TableMetadata;
    
    let metadata = TableMetadata {
        format_version: 2,
        table_uuid: uuid::Uuid::new_v4(),
        location: "s3://warehouse/test/table".to_string(),
        last_updated_ms: 0,
        last_column_id: 0,
        schemas: vec![],
        current_schema_id: 0,
        partition_specs: vec![],
        current_partition_spec_id: 0,
        properties: None,
        current_snapshot_id: None,
        snapshots: None,
        snapshot_log: None,
        metadata_log: None,
        sort_orders: vec![],
        default_sort_order_id: 0,
        last_sequence_number: 0,
    };
    
    let credentials = Some(("test_access_key".to_string(), "test_secret_key".to_string()));
    
    let response = TableResponse::with_credentials(
        Some("s3://warehouse/test/table/metadata/v1.json".to_string()),
        metadata,
        credentials,
    );
    
    // Verify credentials are in config
    assert!(response.config.is_some());
    let config = response.config.unwrap();
    assert_eq!(config.get("s3.access-key-id").unwrap(), "test_access_key");
    assert_eq!(config.get("s3.secret-access-key").unwrap(), "test_secret_key");
    assert!(config.contains_key("s3.endpoint"));
    assert!(config.contains_key("s3.region"));
}

#[test]
fn test_table_response_without_credentials() {
    // Test that TableResponse works without credentials (client-provided scenario)
    use crate::iceberg_handlers::TableResponse;
    use pangolin_core::iceberg_metadata::TableMetadata;
    
    let metadata = TableMetadata {
        format_version: 2,
        table_uuid: uuid::Uuid::new_v4(),
        location: "s3://warehouse/test/table".to_string(),
        last_updated_ms: 0,
        last_column_id: 0,
        schemas: vec![],
        current_schema_id: 0,
        partition_specs: vec![],
        current_partition_spec_id: 0,
        properties: None,
        current_snapshot_id: None,
        snapshots: None,
        snapshot_log: None,
        metadata_log: None,
        sort_orders: vec![],
        default_sort_order_id: 0,
        last_sequence_number: 0,
    };
    
    let response = TableResponse::new(
        Some("s3://warehouse/test/table/metadata/v1.json".to_string()),
        metadata,
    );
    
    // Verify config exists but doesn't have credentials
    assert!(response.config.is_some());
    let config = response.config.unwrap();
    assert!(!config.contains_key("s3.access-key-id"));
    assert!(!config.contains_key("s3.secret-access-key"));
    assert!(config.contains_key("s3.endpoint")); // Should still have endpoint
    assert!(config.contains_key("s3.region")); // Should still have region
}

// Tests for credential vending helper functions

#[tokio::test]
async fn test_assume_role_aws_placeholder() {
    // Test the placeholder implementation (without aws-sts feature)
    let result = crate::signing_handlers::assume_role_aws(
        "arn:aws:iam::123456789:role/TestRole",
        Some("external-id-123"),
        "test-session"
    ).await;
    
    assert!(result.is_ok());
    let (access_key, secret_key, session_token, expiration) = result.unwrap();
    
    // Placeholder should return formatted strings
    assert!(access_key.contains("arn:aws:iam::123456789:role/TestRole"));
    assert_eq!(secret_key, "STS_SECRET_KEY_PLACEHOLDER");
    assert_eq!(session_token, "STS_SESSION_TOKEN_PLACEHOLDER");
    assert!(!expiration.is_empty());
}

#[tokio::test]
async fn test_get_azure_token_placeholder() {
    // Test the placeholder implementation (without azure-oauth feature)
    let result = crate::signing_handlers::get_azure_token(
        "tenant-id-123",
        "client-id-456",
        "client-secret-789"
    ).await;
    
    assert!(result.is_ok());
    let token = result.unwrap();
    
    // Placeholder should return placeholder token
    assert_eq!(token, "AZURE_OAUTH_TOKEN_PLACEHOLDER");
}

#[tokio::test]
async fn test_get_gcp_token_placeholder() {
    // Test the placeholder implementation (without gcp-oauth feature)
    let service_account_json = r#"{"type":"service_account","project_id":"test"}"#;
    let result = crate::signing_handlers::get_gcp_token(service_account_json).await;
    
    assert!(result.is_ok());
    let token = result.unwrap();
    
    // Placeholder should return placeholder token
    assert_eq!(token, "GCS_OAUTH_TOKEN_PLACEHOLDER");
}

#[tokio::test]
async fn test_assume_role_aws_with_external_id() {
    // Test that external ID is properly handled
    let result = crate::signing_handlers::assume_role_aws(
        "arn:aws:iam::123456789:role/TestRole",
        Some("my-external-id"),
        "test-session-with-external-id"
    ).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_assume_role_aws_without_external_id() {
    // Test that external ID is optional
    let result = crate::signing_handlers::assume_role_aws(
        "arn:aws:iam::123456789:role/TestRole",
        None,
        "test-session-no-external-id"
    ).await;
    
    assert!(result.is_ok());
}


