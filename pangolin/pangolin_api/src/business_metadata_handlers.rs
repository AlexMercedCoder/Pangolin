use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Extension,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use pangolin_store::CatalogStore;
use pangolin_core::business_metadata::{BusinessMetadata, AccessRequest, RequestStatus};
use pangolin_core::user::{UserSession, UserRole};
use pangolin_core::permission::{Action, PermissionScope};
use uuid::Uuid;
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
    // If setting discoverable=true, check MANAGE_DISCOVERY permission
    if payload.discoverable {
        // Get catalog ID for this asset
        let catalog_name = match crate::authz::get_catalog_for_asset(&store, session.tenant_id.unwrap_or_default(), asset_id).await {
            Ok(name) => name,
            Err(_) => return (StatusCode::NOT_FOUND, "Asset not found").into_response(),
        };
        
        // For now, use a simplified check - just verify user is admin
        // TODO: Once we have catalog IDs, use proper scope checking
        if !crate::authz::is_admin(&session.role) {
            return (StatusCode::FORBIDDEN, "Missing MANAGE_DISCOVERY permission on this catalog").into_response();
        }
    }

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

pub async fn search_assets(
    State(store): State<AppState>,
    Extension(session): Extension<UserSession>,
    Query(params): Query<SearchRequest>,
) -> impl IntoResponse {
    let tenant_id = session.tenant_id.unwrap_or_default();
    
    match store.search_assets(tenant_id, &params.query, params.tags).await {
        Ok(results) => {
            // Filter to only return discoverable assets OR assets the user has permission to access
            let filtered_results: Vec<_> = results.into_iter()
                .filter(|(_, metadata)| {
                    // Show if: user is admin OR asset is discoverable
                    // TODO: Add check for user's direct read permissions on specific assets
                    crate::authz::is_admin(&session.role) || 
                    metadata.as_ref().map(|m| m.discoverable).unwrap_or(false)
                })
                .map(|(asset, metadata)| {
                    serde_json::json!({
                        "id": asset.id,
                        "name": asset.name,
                        "description": metadata.as_ref().and_then(|m| m.description.clone()),
                        "tags": metadata.as_ref().map(|m| &m.tags).unwrap_or(&vec![]),
                    })
                })
                .collect();
            
            (StatusCode::OK, Json(filtered_results)).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Search failed: {}", e)).into_response(),
    }
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
    let tenant_id = session.tenant_id.unwrap_or_default();
    let request = AccessRequest::new(tenant_id, session.user_id, asset_id, payload.reason);

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

pub async fn get_access_request(
    State(store): State<AppState>,
    Extension(session): Extension<UserSession>,
    Path(request_id): Path<Uuid>,
) -> impl IntoResponse {
    match store.get_access_request(request_id).await {
        Ok(Some(request)) => {
            // Check visibility: Admin or requester
            if session.role != UserRole::TenantAdmin && session.role != UserRole::Root && request.user_id != session.user_id {
                return (StatusCode::FORBIDDEN, "Access denied").into_response();
            }
            (StatusCode::OK, Json(request)).into_response()
        },
        Ok(None) => (StatusCode::NOT_FOUND, "Request not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal error: {}", e)).into_response(),
    }
}

pub async fn get_asset_details(
    State(store): State<AppState>,
    Extension(session): Extension<UserSession>,
    Path(asset_id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = session.tenant_id.unwrap_or_default();
    
    // Scan all assets to find the one with this ID
    // TODO: Optimize this with an index in CatalogStore or direct lookup if supported
    
    let catalogs = match store.list_catalogs(tenant_id).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to list catalogs: {}", e)).into_response(),
    };

    for catalog in catalogs {
        let namespaces = match store.list_namespaces(tenant_id, &catalog.name, None).await {
             Ok(n) => n,
             Err(_) => continue,
        };
        
        for ns in namespaces {
            let ns_parts = ns.name.clone(); 
            if let Ok(assets) = store.list_assets(tenant_id, &catalog.name, None, ns_parts).await {
                for asset in assets {
                    if asset.id == asset_id {
                        // Found it!
                        let metadata = store.get_business_metadata(asset_id).await.unwrap_or(None);
                        
                        return (StatusCode::OK, Json(serde_json::json!({
                            "asset": asset,
                            "metadata": metadata,
                            "catalog": catalog.name,
                            "namespace": ns.name
                        }))).into_response();
                    }
                }
            }
        }
    }
    
    (StatusCode::NOT_FOUND, "Asset not found").into_response()
}
