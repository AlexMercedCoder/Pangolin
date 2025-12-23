// Live integration tests for Azure OAuth2 with oidc-server-mock
use pangolin_api::credential_vending::create_signer_from_warehouse;
use pangolin_core::model::Warehouse;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
#[ignore] // Run with: cargo test --test live_azure_oauth2 -- --ignored
async fn test_live_azure_oauth2_with_oidc_mock() {
    println!("\n🧪 Testing Azure OAuth2 with oidc-server-mock...");
    
    // Create Azure warehouse configuration with OAuth2 pointing to mock server
    let mut storage_config = HashMap::new();
    storage_config.insert("type".to_string(), "azure".to_string());
    storage_config.insert("account_name".to_string(), "devstoreaccount1".to_string());
    storage_config.insert("container".to_string(), "test-container".to_string());
    
    // OAuth2 configuration pointing to oidc-server-mock
    storage_config.insert("tenant_id".to_string(), "mock-tenant".to_string());
    storage_config.insert("client_id".to_string(), "pangolin-client".to_string());
    storage_config.insert("client_secret".to_string(), "secret".to_string());
    storage_config.insert("authority_host".to_string(), "http://localhost:8081".to_string());
    
    let warehouse = Warehouse {
        id: Uuid::new_v4(),
        name: "azure-oauth-warehouse".to_string(),
        tenant_id: Uuid::new_v4(),
        storage_config,
        use_sts: true, // Enable OAuth2 mode
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
    
    assert!(result.is_ok(), "Failed to vend Azure OAuth2 credentials: {:?}", result.err());
    let creds = result.unwrap();
    
    // Verify credentials
    assert_eq!(creds.prefix, "abfss://test-container@devstoreaccount1.dfs.core.windows.net/");
    assert_eq!(creds.config.get("credential-type").unwrap(), "azure-oauth");
    assert!(creds.config.contains_key("azure-oauth-token"));
    assert!(creds.expires_at.is_some());
    
    println!("✅ Azure OAuth2 with oidc-server-mock test passed!");
    println!("   Prefix: {}", creds.prefix);
    println!("   Token type: {}", creds.config.get("credential-type").unwrap());
    println!("   Has OAuth token: {}", creds.config.contains_key("azure-oauth-token"));
}
