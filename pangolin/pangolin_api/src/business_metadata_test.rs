use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::{post, get, delete, put},
    Router,
};
use tower::ServiceExt;
use pangolin_core::user::{User, UserRole};
use pangolin_core::model::{Asset, AssetType};
use pangolin_store::memory::MemoryStore;
use pangolin_store::CatalogStore;
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use crate::business_metadata_handlers::{
    add_business_metadata, get_business_metadata, request_access, list_access_requests, update_access_request,
    AddMetadataRequest, CreateAccessRequestPayload, UpdateRequestStatus
};
use crate::auth_middleware::{auth_middleware, hash_password};
use crate::auth::TenantId;
use pangolin_core::business_metadata::RequestStatus;
use serial_test::serial;
use crate::tests_common::EnvGuard;

#[tokio::test]
#[serial]
async fn test_business_metadata_flow() {
    // 1. Setup Store
    let store_impl = MemoryStore::new();
    let store: Arc<dyn CatalogStore + Send + Sync> = Arc::new(store_impl);
    
    // 2. Setup Router
    let app = Router::new()
        .route("/api/v1/assets/:id/metadata", post(add_business_metadata).get(get_business_metadata))
        .route("/api/v1/assets/:id/request-access", post(request_access))
        .route("/api/v1/access-requests", get(list_access_requests))
        .route("/api/v1/access-requests/:id", put(update_access_request))
        .layer(axum::middleware::from_fn(auth_middleware))
        .with_state(store.clone());

    // 3. Create Tenant & User
    let tenant_id = Uuid::new_v4();
    // Insert tenant to memory store (bypass handler)
    let tenant = pangolin_core::model::Tenant {
        id: tenant_id,
        name: "test_tenant".to_string(),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant.clone()).await.unwrap();

    let password = "password";
    let hash = hash_password(password).unwrap();
    let user = User::new_root(
        "admin".to_string(),
        "admin@example.com".to_string(), 
        hash,
    );
    store.create_user(user.clone()).await.unwrap();

    // 4. Create Asset
    let asset_id = Uuid::new_v4();
    let asset = Asset {
        id: asset_id,
        name: "test_table".to_string(),
        kind: AssetType::IcebergTable,
        location: "s3://bucket".to_string(),
        properties: HashMap::new(),
    };
    // Need to insert asset properly to be found.
    // Create Asset manually in store
    store.create_asset(tenant_id, "default", None, vec!["ns".to_string()], asset.clone()).await.unwrap();

    // 5. Generate Token
    // We can use the logic from login handler or manually create JWT
    let secret = "secret"; 
    let _guard = EnvGuard::new("PANGOLIN_JWT_SECRET", secret);
    
    // Create session for token generation
    let session = pangolin_core::user::UserSession {
        user_id: user.id,
        username: user.username.clone(),
        tenant_id: Some(tenant_id),
        role: UserRole::Root,
        issued_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
    };
    
    let token = crate::auth_middleware::generate_token(session, secret).unwrap();
    let auth_header = format!("Bearer {}", token);

    // 6. Test Add Metadata
    let metadata_req = AddMetadataRequest {
        description: Some("Test description".to_string()),
        tags: vec!["pii".to_string()],
        properties: HashMap::new(),
        discoverable: true,
    };
    
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/assets/{}/metadata", asset_id))
        .header("Authorization", &auth_header)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&metadata_req).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 7. Test Get Metadata
    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/assets/{}/metadata", asset_id))
        .header("Authorization", &auth_header)
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // 8. Test Request Access
    let access_req = CreateAccessRequestPayload {
        reason: Some("Need access".to_string()),
    };
    
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/assets/{}/request-access", asset_id))
        .header("Authorization", &auth_header)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&access_req).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let access_res: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let request_id = access_res["id"].as_str().unwrap();

    // 9. Test List Access Requests
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/access-requests")
        .header("Authorization", &auth_header)
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // 10. Test Approve Request
    let update_req = UpdateRequestStatus {
        status: RequestStatus::Approved,
        comment: Some("Approved".to_string()),
    };
    
    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/access-requests/{}", request_id))
        .header("Authorization", &auth_header)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&update_req).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
