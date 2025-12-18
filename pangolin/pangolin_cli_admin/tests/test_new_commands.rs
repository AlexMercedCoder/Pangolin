use pangolin_cli_admin::handlers;
use pangolin_cli_common::client::PangolinClient;
use mockito::{mock, server_url};
use serde_json::json;

#[tokio::test]
async fn test_update_tenant() {
    let _m = mock("PUT", "/api/v1/tenants/test-id")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"test-id","name":"updated-tenant"}"#)
        .create();

    let client = PangolinClient::new(&server_url(), None);
    let result = handlers::handle_update_tenant(
        &client,
        "test-id".to_string(),
        Some("updated-tenant".to_string()),
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_tenant_error() {
    let _m = mock("PUT", "/api/v1/tenants/invalid-id")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error":"Tenant not found"}"#)
        .create();

    let client = PangolinClient::new(&server_url(), None);
    let result = handlers::handle_update_tenant(
        &client,
        "invalid-id".to_string(),
        Some("updated-tenant".to_string()),
    )
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_user() {
    let _m = mock("PUT", "/api/v1/users/user-id")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"user-id","username":"updated-user"}"#)
        .create();

    let client = PangolinClient::new(&server_url(), None);
    let result = handlers::handle_update_user(
        &client,
        "user-id".to_string(),
        Some("updated-user".to_string()),
        Some("new@email.com".to_string()),
        Some(true),
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_warehouse() {
    let _m = mock("PUT", "/api/v1/warehouses/wh-id")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"wh-id","name":"updated-warehouse"}"#)
        .create();

    let client = PangolinClient::new(&server_url(), None);
    let result = handlers::handle_update_warehouse(
        &client,
        "wh-id".to_string(),
        Some("updated-warehouse".to_string()),
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_catalog() {
    let _m = mock("PUT", "/api/v1/catalogs/cat-id")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"cat-id","name":"updated-catalog"}"#)
        .create();

    let client = PangolinClient::new(&server_url(), None);
    let result = handlers::handle_update_catalog(
        &client,
        "cat-id".to_string(),
        Some("updated-catalog".to_string()),
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_revoke_token() {
    let _m = mock("POST", "/api/v1/tokens/revoke")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":"Token revoked"}"#)
        .create();

    let client = PangolinClient::new(&server_url(), None);
    let result = handlers::handle_revoke_token(&client).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_revoke_token_by_id() {
    let _m = mock("POST", "/api/v1/tokens/revoke/token-id")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":"Token revoked"}"#)
        .create();

    let client = PangolinClient::new(&server_url(), None);
    let result = handlers::handle_revoke_token_by_id(&client, "token-id".to_string()).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_merge_operations() {
    let _m = mock("GET", "/api/v1/merge-operations")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"[{"id":"merge-1","source_branch":"feature","target_branch":"main","status":"pending"}]"#)
        .create();

    let client = PangolinClient::new(&server_url(), None);
    let result = handlers::handle_list_merge_operations(&client).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_merge_operation() {
    let _m = mock("GET", "/api/v1/merge-operations/merge-id")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"merge-id","source_branch":"feature","target_branch":"main","status":"pending"}"#)
        .create();

    let client = PangolinClient::new(&server_url(), None);
    let result = handlers::handle_get_merge_operation(&client, "merge-id".to_string()).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_conflicts() {
    let _m = mock("GET", "/api/v1/merge-operations/merge-id/conflicts")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"[{"id":"conflict-1","asset_id":"asset-1","conflict_type":"schema","resolved":false}]"#)
        .create();

    let client = PangolinClient::new(&server_url(), None);
    let result = handlers::handle_list_conflicts(&client, "merge-id".to_string()).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_resolve_conflict() {
    let _m = mock("POST", "/api/v1/merge-operations/merge-id/conflicts/conflict-id/resolve")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":"Conflict resolved"}"#)
        .create();

    let client = PangolinClient::new(&server_url(), None);
    let result = handlers::handle_resolve_conflict(
        &client,
        "merge-id".to_string(),
        "conflict-id".to_string(),
        "use-source".to_string(),
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_complete_merge() {
    let _m = mock("POST", "/api/v1/merge-operations/merge-id/complete")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":"Merge completed"}"#)
        .create();

    let client = PangolinClient::new(&server_url(), None);
    let result = handlers::handle_complete_merge(&client, "merge-id".to_string()).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_abort_merge() {
    let _m = mock("POST", "/api/v1/merge-operations/merge-id/abort")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":"Merge aborted"}"#)
        .create();

    let client = PangolinClient::new(&server_url(), None);
    let result = handlers::handle_abort_merge(&client, "merge-id".to_string()).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_metadata() {
    let _m = mock("DELETE", "/api/v1/business-metadata/asset-id")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":"Metadata deleted"}"#)
        .create();

    let client = PangolinClient::new(&server_url(), None);
    let result = handlers::handle_delete_metadata(&client, "asset-id".to_string()).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_request_access() {
    let _m = mock("POST", "/api/v1/access-requests")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"request-id","asset_id":"asset-id","status":"pending"}"#)
        .create();

    let client = PangolinClient::new(&server_url(), None);
    let result = handlers::handle_request_access(
        &client,
        "asset-id".to_string(),
        "Need access for analysis".to_string(),
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_access_requests() {
    let _m = mock("GET", "/api/v1/access-requests")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"[{"id":"req-1","asset_id":"asset-1","status":"pending"}]"#)
        .create();

    let client = PangolinClient::new(&server_url(), None);
    let result = handlers::handle_list_access_requests(&client).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_access_request() {
    let _m = mock("PUT", "/api/v1/access-requests/request-id")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"request-id","status":"approved"}"#)
        .create();

    let client = PangolinClient::new(&server_url(), None);
    let result = handlers::handle_update_access_request(
        &client,
        "request-id".to_string(),
        "approved".to_string(),
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_asset_details() {
    let _m = mock("GET", "/api/v1/assets/asset-id")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"asset-id","name":"test-asset","asset_type":"table"}"#)
        .create();

    let client = PangolinClient::new(&server_url(), None);
    let result = handlers::handle_get_asset_details(&client, "asset-id".to_string()).await;

    assert!(result.is_ok());
}
