use axum::{
    extract::{Path, State, Extension, Query},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use pangolin_store::CatalogStore;
use pangolin_core::business_metadata::{BusinessMetadata, AccessRequest, RequestStatus};
use pangolin_core::user::UserRole;
use uuid::Uuid;
use pangolin_core::user::UserSession;
use crate::iceberg_handlers::AppState;

#[derive(Deserialize, Serialize)]
pub struct AddMetadataRequest {
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub properties: std::collections::HashMap<String, String>,
    pub discoverable: bool,
}

#[derive(Serialize)]
pub struct MetadataResponse {
    pub metadata: BusinessMetadata,
}

pub async fn add_business_metadata(
    State(store): State<AppState>,
    Extension(session): Extension<UserSession>,
    Path(asset_id): Path<Uuid>,
    Json(payload): Json<AddMetadataRequest>,
) -> impl IntoResponse {
    // Check permission - assumed checking done by middleware or earlier, or we add check_permission here
    // For now, let's assume any TenantUser with write access to asset can add metadata.
    // TODO: proper RBAC check (update action on asset scope)

    let mut metadata = BusinessMetadata::new(asset_id, session.user_id)
        .with_tags(payload.tags)
        .with_discoverable(payload.discoverable);

    if let Some(desc) = payload.description {
        metadata = metadata.with_description(desc);
    }
    
    metadata.properties = payload.properties;

    match store.upsert_business_metadata(metadata.clone()).await {
        Ok(_) => (StatusCode::OK, Json(MetadataResponse { metadata })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save metadata: {}", e)).into_response(),
    }
}

pub async fn get_business_metadata(
    State(store): State<AppState>,
    Extension(_session): Extension<UserSession>,
    Path(asset_id): Path<Uuid>,
) -> impl IntoResponse {
    // Check read permission? Or discoverability?
    // If user has read access, return it.
    // If not, return only if discoverable?
    
    match store.get_business_metadata(asset_id).await {
        Ok(Some(metadata)) => (StatusCode::OK, Json(MetadataResponse { metadata })).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Metadata not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal Server Error: {}", e)).into_response(),
    }
}

pub async fn delete_business_metadata(
    State(store): State<AppState>,
    Extension(_session): Extension<UserSession>,
    Path(asset_id): Path<Uuid>,
) -> impl IntoResponse {
    // Check permission logic
    
    match store.delete_business_metadata(asset_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete: {}", e)).into_response(),
    }
}

#[derive(Deserialize, Serialize)]
pub struct SearchRequest {
    pub query: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct SearchResponse {
    // TODO: return assets with metadata snippets
    pub results: Vec<String>, // Placeholder
}

pub async fn search_assets(
    State(_store): State<AppState>,
    Extension(_session): Extension<UserSession>,
    Query(_params): Query<SearchRequest>,
) -> impl IntoResponse {
    // TODO: Implement search across assets and metadata
    // Need a search method in CatalogStore
    (StatusCode::NOT_IMPLEMENTED, "Search not implemented").into_response()
}

#[derive(Deserialize, Serialize)]
pub struct CreateAccessRequestPayload {
    pub reason: Option<String>,
}

pub async fn request_access(
    State(store): State<AppState>,
    Extension(session): Extension<UserSession>,
    Path(asset_id): Path<Uuid>,
    Json(payload): Json<CreateAccessRequestPayload>,
) -> impl IntoResponse {
    let request = AccessRequest::new(session.user_id, asset_id, payload.reason);

    match store.create_access_request(request.clone()).await {
        Ok(_) => (StatusCode::CREATED, Json(request)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create request: {}", e)).into_response(),
    }
}

pub async fn list_access_requests(
    State(store): State<AppState>,
    Extension(session): Extension<UserSession>,
) -> impl IntoResponse {
    // List requests for tenant
    // Ideally filter by user if not admin?
    // AccessRequest logic:
    // - TenantAdmin sees all requests for tenant
    // - User sees their own requests?
    
    let tenant_id = session.tenant_id.unwrap_or_default(); // Or root tenant?
    
    match store.list_access_requests(tenant_id).await {
        Ok(requests) => {
            // Filter if not admin?
            let filtered = if session.role != UserRole::TenantAdmin && session.role != UserRole::Root {
                 requests.into_iter().filter(|r| r.user_id == session.user_id).collect()
            } else {
                requests
            };
            (StatusCode::OK, Json(filtered)).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to list requests: {}", e)).into_response(),
    }
}

#[derive(Deserialize, Serialize)]
pub struct UpdateRequestStatus {
    pub status: RequestStatus,
    pub comment: Option<String>,
}

pub async fn update_access_request(
    State(store): State<AppState>,
    Extension(session): Extension<UserSession>,
    Path(request_id): Path<Uuid>,
    Json(payload): Json<UpdateRequestStatus>,
) -> impl IntoResponse {
    // Only Admin can approve/reject
    if session.role != UserRole::TenantAdmin && session.role != UserRole::Root {
        return (StatusCode::FORBIDDEN, "Only admins can review requests").into_response();
    }

    match store.get_access_request(request_id).await {
        Ok(Some(mut request)) => {
            match payload.status {
                RequestStatus::Approved => request.approve(session.user_id, payload.comment),
                RequestStatus::Rejected => request.reject(session.user_id, payload.comment),
                RequestStatus::Pending => return (StatusCode::BAD_REQUEST, "Cannot set status back to pending").into_response(),
            }
            
            match store.update_access_request(request.clone()).await {
                Ok(_) => (StatusCode::OK, Json(request)).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update request: {}", e)).into_response(),
            }
        },
        Ok(None) => (StatusCode::NOT_FOUND, "Request not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal error: {}", e)).into_response(),
    }
}
