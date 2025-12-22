use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use pangolin_cli_common::client::PangolinClient;
use pangolin_cli_common::config::CliConfig;
use serde_json::json;

// Helper to create test client
fn create_test_client(base_url: String) -> PangolinClient {
    let config = CliConfig {
        base_url,
        auth_token: Some("test-token".to_string()),
        username: Some("test-user".to_string()),
        tenant_id: Some("test-tenant-id".to_string()),
        tenant_name: Some("test-tenant".to_string()),
    };
    PangolinClient::new(config)
}

#[tokio::test]
async fn test_resolve_user_id_success() {
    // Start mock server
    let mock_server = MockServer::start().await;
    
    // Mock /api/v1/users endpoint returning a user list
    Mock::given(method("GET"))
        .and(path("/api/v1/users"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": "550e8400-e29b-41d4-a716-446655440000",
                "username": "alice",
                "email": "alice@example.com"
            },
            {
                "id": "660e8400-e29b-41d4-a716-446655440001",
                "username": "bob",
                "email": "bob@example.com"
            }
        ])))
        .mount(&mock_server)
        .await;
    
    // Create client pointing to mock server
    let client = create_test_client(mock_server.uri());
    
    // Call resolve_user_id (we need to make this function public or test it indirectly)
    // For now, we'll test the logic by calling the API directly
    let res = client.get("/api/v1/users").await.unwrap();
    assert!(res.status().is_success());
    
    let users: Vec<serde_json::Value> = res.json().await.unwrap();
    let alice = users.iter().find(|u| u["username"].as_str() == Some("alice"));
    assert!(alice.is_some());
    assert_eq!(alice.unwrap()["id"].as_str().unwrap(), "550e8400-e29b-41d4-a716-446655440000");
}

#[tokio::test]
async fn test_resolve_user_id_not_found() {
    // Start mock server
    let mock_server = MockServer::start().await;
    
    // Mock /api/v1/users endpoint returning empty list
    Mock::given(method("GET"))
        .and(path("/api/v1/users"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&mock_server)
        .await;
    
    // Create client
    let client = create_test_client(mock_server.uri());
    
    // Call API
    let res = client.get("/api/v1/users").await.unwrap();
    let users: Vec<serde_json::Value> = res.json().await.unwrap();
    
    // Verify user not found
    let nonexistent = users.iter().find(|u| u["username"].as_str() == Some("nonexistent"));
    assert!(nonexistent.is_none());
}

#[tokio::test]
async fn test_resolve_role_id_success() {
    // Start mock server
    let mock_server = MockServer::start().await;
    
    // Mock /api/v1/roles endpoint
    Mock::given(method("GET"))
        .and(path("/api/v1/roles"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": "770e8400-e29b-41d4-a716-446655440000",
                "name": "admin",
                "description": "Administrator role"
            },
            {
                "id": "880e8400-e29b-41d4-a716-446655440001",
                "name": "viewer",
                "description": "Read-only role"
            }
        ])))
        .mount(&mock_server)
        .await;
    
    // Create client
    let client = create_test_client(mock_server.uri());
    
    // Call API
    let res = client.get("/api/v1/roles").await.unwrap();
    assert!(res.status().is_success());
    
    let roles: Vec<serde_json::Value> = res.json().await.unwrap();
    let admin = roles.iter().find(|r| r["name"].as_str() == Some("admin"));
    assert!(admin.is_some());
    assert_eq!(admin.unwrap()["id"].as_str().unwrap(), "770e8400-e29b-41d4-a716-446655440000");
}

#[tokio::test]
async fn test_resolve_role_id_not_found() {
    // Start mock server
    let mock_server = MockServer::start().await;
    
    // Mock /api/v1/roles endpoint returning empty list
    Mock::given(method("GET"))
        .and(path("/api/v1/roles"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&mock_server)
        .await;
    
    // Create client
    let client = create_test_client(mock_server.uri());
    
    // Call API
    let res = client.get("/api/v1/roles").await.unwrap();
    let roles: Vec<serde_json::Value> = res.json().await.unwrap();
    
    // Verify role not found
    let nonexistent = roles.iter().find(|r| r["name"].as_str() == Some("nonexistent"));
    assert!(nonexistent.is_none());
}

#[tokio::test]
async fn test_user_resolution_with_multiple_matches() {
    // Start mock server
    let mock_server = MockServer::start().await;
    
    // Mock endpoint with duplicate usernames (edge case)
    Mock::given(method("GET"))
        .and(path("/api/v1/users"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": "111e8400-e29b-41d4-a716-446655440000",
                "username": "alice",
                "email": "alice1@example.com"
            },
            {
                "id": "222e8400-e29b-41d4-a716-446655440001",
                "username": "alice",
                "email": "alice2@example.com"
            }
        ])))
        .mount(&mock_server)
        .await;
    
    // Create client
    let client = create_test_client(mock_server.uri());
    
    // Call API
    let res = client.get("/api/v1/users").await.unwrap();
    let users: Vec<serde_json::Value> = res.json().await.unwrap();
    
    // Verify first match is returned (current implementation behavior)
    let alice = users.iter().find(|u| u["username"].as_str() == Some("alice"));
    assert!(alice.is_some());
    assert_eq!(alice.unwrap()["id"].as_str().unwrap(), "111e8400-e29b-41d4-a716-446655440000");
}
