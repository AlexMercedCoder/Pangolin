// Live integration tests with cloud storage emulators
// These tests validate credential vending against real storage services
use pangolin_api::credential_vending::create_signer_from_warehouse;
use pangolin_core::model::Warehouse;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
#[ignore] // Run with: cargo test --test live_credential_vending -- --ignored
async fn test_live_minio_static_credentials() {
    println!("\n🧪 Testing MinIO with static credentials...");
    
    // Create S3 warehouse configuration for MinIO
    let mut storage_config = HashMap::new();
    storage_config.insert("type".to_string(), "s3".to_string());
    storage_config.insert("bucket".to_string(), "test-bucket".to_string());
    storage_config.insert("access_key_id".to_string(), "minioadmin".to_string());
    storage_config.insert("secret_access_key".to_string(), "minioadmin".to_string());
    storage_config.insert("endpoint".to_string(), "http://localhost:9000".to_string());
    storage_config.insert("region".to_string(), "us-east-1".to_string());
    
    let warehouse = Warehouse {
        id: Uuid::new_v4(),
        name: "minio-warehouse".to_string(),
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
    
    assert!(result.is_ok(), "Failed to vend MinIO credentials: {:?}", result.err());
    let creds = result.unwrap();
    
    // Verify credentials
    assert_eq!(creds.prefix, "s3://test-bucket/");
    assert_eq!(creds.config.get("s3.access-key-id").unwrap(), "minioadmin");
    assert_eq!(creds.config.get("s3.endpoint").unwrap(), "http://localhost:9000");
    
    println!("✅ MinIO static credentials test passed!");
    println!("   Prefix: {}", creds.prefix);
    println!("   Endpoint: {}", creds.config.get("s3.endpoint").unwrap());
}

#[tokio::test]
#[ignore]
async fn test_live_azurite_credentials() {
    println!("\n🧪 Testing Azurite (Azure emulator)...");
    
    // Azurite uses well-known development credentials
    let mut storage_config = HashMap::new();
    storage_config.insert("type".to_string(), "azure".to_string());
    storage_config.insert("account_name".to_string(), "devstoreaccount1".to_string());
    storage_config.insert("container".to_string(), "test-container".to_string());
    storage_config.insert("account_key".to_string(), 
        "Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==".to_string());
    storage_config.insert("endpoint".to_string(), "http://localhost:10000/devstoreaccount1".to_string());
    
    let warehouse = Warehouse {
        id: Uuid::new_v4(),
        name: "azurite-warehouse".to_string(),
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
    
    assert!(result.is_ok(), "Failed to vend Azurite credentials: {:?}", result.err());
    let creds = result.unwrap();
    
    // Verify credentials (PyIceberg-compatible property names)
    assert_eq!(creds.prefix, "abfss://test-container@devstoreaccount1.dfs.core.windows.net/");
    assert!(creds.config.contains_key("adls.account-name"));
    assert!(creds.config.contains_key("adls.account-key"));
    
    println!("✅ Azurite credentials test passed!");
    println!("   Prefix: {}", creds.prefix);
    println!("   Account: {}", creds.config.get("adls.account-name").unwrap());
}

#[tokio::test]
#[ignore]
async fn test_live_fake_gcs_credentials() {
    println!("\n🧪 Testing fake-gcs-server (GCP emulator)...");
    
    // fake-gcs-server doesn't require real credentials
    let mut storage_config = HashMap::new();
    storage_config.insert("type".to_string(), "gcs".to_string());
    storage_config.insert("project_id".to_string(), "test-project".to_string());
    storage_config.insert("bucket".to_string(), "test-bucket".to_string());
    storage_config.insert("endpoint".to_string(), "http://localhost:4443".to_string());
    // Minimal service account key (fake-gcs doesn't validate)
    storage_config.insert("service_account_key".to_string(), 
        r#"{"type":"service_account","project_id":"test-project"}"#.to_string());
    
    let warehouse = Warehouse {
        id: Uuid::new_v4(),
        name: "fake-gcs-warehouse".to_string(),
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
    
    assert!(result.is_ok(), "Failed to vend fake-gcs credentials: {:?}", result.err());
    let creds = result.unwrap();
    
    // Verify credentials
    assert_eq!(creds.prefix, "gs://test-bucket/");
    assert!(creds.config.contains_key("gcp-project-id"));
    
    println!("✅ fake-gcs credentials test passed!");
    println!("   Prefix: {}", creds.prefix);
    println!("   Project: {}", creds.config.get("gcp-project-id").unwrap());
}

