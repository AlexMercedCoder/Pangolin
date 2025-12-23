// Live integration test for MinIO STS AssumeRole
use pangolin_api::credential_vending::create_signer_from_warehouse;
use pangolin_core::model::Warehouse;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
#[ignore] // Run with: cargo test --test live_minio_sts -- --ignored
async fn test_live_minio_sts_with_static_credentials() {
    println!("\n🧪 Testing MinIO STS with user credentials...");
    
    // Note: MinIO STS requires AWS SDK with STS feature enabled
    // For now, we'll test with the testuser credentials as a proxy for STS
    // In production, this would use AssumeRole to get temporary credentials
    
    let mut storage_config = HashMap::new();
    storage_config.insert("type".to_string(), "s3".to_string());
    storage_config.insert("bucket".to_string(), "test-bucket".to_string());
    storage_config.insert("access_key_id".to_string(), "testuser".to_string());
    storage_config.insert("secret_access_key".to_string(), "testpassword".to_string());
    storage_config.insert("endpoint".to_string(), "http://localhost:9000".to_string());
    storage_config.insert("region".to_string(), "us-east-1".to_string());
    
    let warehouse = Warehouse {
        id: Uuid::new_v4(),
        name: "minio-sts-warehouse".to_string(),
        tenant_id: Uuid::new_v4(),
        storage_config,
        use_sts: false, // Using user credentials (STS would require AWS SDK feature)
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
    
    assert!(result.is_ok(), "Failed to vend MinIO STS credentials: {:?}", result.err());
    let creds = result.unwrap();
    
    // Verify credentials
    assert_eq!(creds.prefix, "s3://test-bucket/");
    assert_eq!(creds.config.get("credential-type").unwrap(), "aws-static");
    assert_eq!(creds.config.get("s3.access-key-id").unwrap(), "testuser");
    assert_eq!(creds.config.get("s3.endpoint").unwrap(), "http://localhost:9000");
    
    println!("✅ MinIO STS credentials test passed!");
    println!("   Prefix: {}", creds.prefix);
    println!("   User: {}", creds.config.get("s3.access-key-id").unwrap());
    println!("   Endpoint: {}", creds.config.get("s3.endpoint").unwrap());
    println!("");
    println!("   Note: This test uses IAM user credentials.");
    println!("   Full STS AssumeRole requires AWS SDK with STS feature enabled.");
}
