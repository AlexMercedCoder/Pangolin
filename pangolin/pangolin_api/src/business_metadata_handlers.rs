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
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct AddMetadataRequest {
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub properties: serde_json::Value,
    pub discoverable: bool,
}

#[derive(Serialize, ToSchema)]
pub struct MetadataResponse {
    pub metadata: BusinessMetadata,
}

#[utoipa::path(
    post,
    path = "/api/v1/assets/{asset_id}/metadata",
    tag = "Business Metadata",
    params(
        ("asset_id" = Uuid, Path, description = "Asset ID")
    ),
    request_body = AddMetadataRequest,
    responses(
        (status = 200, description = "Metadata added", body = MetadataResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Asset not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn add_business_metadata(
    State(store): State<AppState>,
    Extension(session): Extension<UserSession>,
    Path(asset_id): Path<Uuid>,
    Json(payload): Json<AddMetadataRequest>,
) -> impl IntoResponse {
    // If setting discoverable=true, check MANAGE_DISCOVERY permission
    if payload.discoverable {
        let tenant_id = session.tenant_id.unwrap_or_default();
        // New Optimized O(1) Lookup
        let (catalog_name, _namespace) = match store.get_asset_by_id(tenant_id, asset_id).await {
            Ok(Some((_, cat, ns))) => (cat, ns),
            Ok(None) => return (StatusCode::NOT_FOUND, "Asset not found").into_response(),
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Store error: {}", e)).into_response(),
        };
        
        // Granular Check: MANAGE_DISCOVERY on specific Catalog
        // Need to find catalog_id from name first? Or use name in scope if supported?
        // PermissionScope uses Catalog options.
        // Let's Resolve Catalog ID from Name
        let catalogs = store.list_catalogs(tenant_id).await.unwrap_or_default();
        let catalog_id = catalogs.iter().find(|c| c.name == catalog_name).map(|c| c.id);

        let has_perm = if let Some(cid) = catalog_id {
            match crate::authz::check_permission(
                &store, 
                &session, 
                &Action::ManageDiscovery, 
                &PermissionScope::Catalog { catalog_id: cid }
            ).await {
                Ok(v) => v,
                Err(_) => false,
            }
        } else {
            false
        };

        if !has_perm && !crate::authz::is_admin(&session.role) {
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

#[utoipa::path(
    get,
    path = "/api/v1/assets/{asset_id}/metadata",
    tag = "Business Metadata",
    params(
        ("asset_id" = Uuid, Path, description = "Asset ID")
    ),
    responses(
        (status = 200, description = "Metadata retrieved", body = MetadataResponse),
        (status = 404, description = "Metadata not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
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

#[utoipa::path(
    delete,
    path = "/api/v1/assets/{asset_id}/metadata",
    tag = "Business Metadata",
    params(
        ("asset_id" = Uuid, Path, description = "Asset ID")
    ),
    responses(
        (status = 204, description = "Metadata deleted"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
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

#[derive(Deserialize, Serialize, ToSchema)]
pub struct SearchRequest {
    pub query: String,
    pub tags: Option<Vec<String>>,
}

#[utoipa::path(
    get,
    path = "/api/v1/assets/search",
    tag = "Business Metadata",
    responses(
        (status = 200, description = "Search results"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn search_assets(
    State(store): State<AppState>,
    Extension(session): Extension<UserSession>,
    Query(params): Query<SearchRequest>,
) -> impl IntoResponse {
    let tenant_id = session.tenant_id.unwrap_or_default();
    
    // Handle #tag syntax in query
    let (query, tags) = if params.query.trim().starts_with('#') {
        let tag = params.query.trim()[1..].to_string();
        let mut t = params.tags.unwrap_or_default();
        t.push(tag);
        ("".to_string(), Some(t))
    } else {
         (params.query, params.tags)
    };
    
    match store.search_assets(tenant_id, &query, tags).await {
        Ok(results) => {
            tracing::info!("SEARCH DEBUG: Session user_id: {}", session.user_id);
            
            // Use authz_utils for consistent permission filtering
            let permissions = if matches!(session.role, UserRole::TenantUser) {
                store.list_user_permissions(session.user_id).await.unwrap_or_default()
            } else {
                Vec::new() // Root/TenantAdmin bypass filtering
            };
            
            // Build catalog ID map for filtering
            let catalogs = store.list_catalogs(tenant_id).await.unwrap_or_default();
            let catalog_map: std::collections::HashMap<_, _> = catalogs.iter()
                .map(|c| (c.name.clone(), c.id))
                .collect();

            // Apply permission-based filtering
            let filtered_results = crate::authz_utils::filter_assets(
                results,
                &permissions,
                session.role.clone(),
                &catalog_map
            );

            // Map to response format with has_access and discoverable flags
            let mapped_results: Vec<_> = filtered_results.into_iter().map(|(asset, metadata, catalog, namespace)| {
                // User has access if they are admin OR have specific read permission
                // If they only see it because it's discoverable, has_access should be false
                let has_access = if matches!(session.role, UserRole::Root | UserRole::TenantAdmin) {
                    true
                } else {
                    if let Some(&catalog_id) = catalog_map.get(&catalog) {
                        let ns_str = namespace.join(".");
                        let required_actions = vec![pangolin_core::permission::Action::Read];
                        crate::authz_utils::has_asset_access(
                            catalog_id,
                            &ns_str,
                            asset.id,
                            &permissions,
                            &required_actions
                        )
                    } else {
                        false
                    }
                };

                let is_discoverable = metadata.as_ref().map(|m| m.discoverable).unwrap_or(false);
                
                serde_json::json!({
                    "id": asset.id,
                    "name": asset.name,
                    "kind": asset.kind,
                    "location": asset.location,
                    "description": metadata.as_ref().and_then(|m| m.description.clone()),
                    "tags": metadata.as_ref().map(|m| &m.tags).unwrap_or(&vec![]),
                    "has_access": has_access,
                    "discoverable": is_discoverable,
                    "catalog": catalog,
                    "namespace": namespace.join(".")
                })
            }).collect();
            
            (StatusCode::OK, Json(mapped_results)).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Search failed: {}", e)).into_response(),
    }
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateAccessRequestPayload {
    pub reason: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/assets/{asset_id}/access-requests",
    tag = "Business Metadata",
    params(
        ("asset_id" = Uuid, Path, description = "Asset ID")
    ),
    request_body = CreateAccessRequestPayload,
    responses(
        (status = 201, description = "Access request created"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
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

#[utoipa::path(
    get,
    path = "/api/v1/access-requests",
    tag = "Business Metadata",
    responses(
        (status = 200, description = "List of access requests"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
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

#[derive(Deserialize, Serialize, ToSchema)]
pub struct UpdateRequestStatus {
    pub status: RequestStatus,
    pub comment: Option<String>,
}

#[utoipa::path(
    put,
    path = "/api/v1/access-requests/{request_id}",
    tag = "Business Metadata",
    params(
        ("request_id" = Uuid, Path, description = "Request ID")
    ),
    request_body = UpdateRequestStatus,
    responses(
        (status = 200, description = "Request updated"),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Request not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
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
                RequestStatus::Approved => {
                    request.approve(session.user_id, payload.comment);
                    
                    // Grant Read Permission automatically
                    if let Ok(Some((_, catalog_name, namespace_vec))) = store.get_asset_by_id(request.tenant_id, request.asset_id).await {
                         // Resolve Catalog ID from name (needed for PermissionScope)
                         if let Ok(catalogs) = store.list_catalogs(request.tenant_id).await {
                             if let Some(catalog) = catalogs.iter().find(|c| c.name == catalog_name) {
                                  // Construct the Permission Scope
                                  let scope = pangolin_core::permission::PermissionScope::Asset {
                                      catalog_id: catalog.id,
                                      namespace: namespace_vec.join("."),
                                      asset_id: request.asset_id
                                  };
                                  
                                  // Define Actions (Read)
                                  let mut actions = std::collections::HashSet::new();
                                  actions.insert(pangolin_core::permission::Action::Read);
                                  
                                  // Create the Permission Object
                                  let permission = pangolin_core::permission::Permission::new(
                                      request.user_id,
                                      scope,
                                      actions,
                                      session.user_id // Granted by the approving admin
                                  );
                                  
                                  // Save to Store
                                  let _ = store.create_permission(permission).await;
                                  tracing::info!("Automatically granted READ permission for approved request {}", request.id);
                             }
                         }
                    }
                },
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

#[utoipa::path(
    get,
    path = "/api/v1/access-requests/{request_id}",
    tag = "Business Metadata",
    params(
        ("request_id" = Uuid, Path, description = "Request ID")
    ),
    responses(
        (status = 200, description = "Request details"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Request not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
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

#[utoipa::path(
    get,
    path = "/api/v1/assets/{asset_id}",
    tag = "Business Metadata",
    params(
        ("asset_id" = Uuid, Path, description = "Asset ID")
    ),
    responses(
        (status = 200, description = "Asset details with metadata"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Asset not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_asset_details(
    State(store): State<AppState>,
    Extension(session): Extension<UserSession>,
    Path(asset_id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = session.tenant_id.unwrap_or_default();
    
    // O(1) Lookup
    let (asset, catalog_name, namespace) = match store.get_asset_by_id(tenant_id, asset_id).await {
        Ok(Some(t)) => t,
        Ok(None) => return (StatusCode::NOT_FOUND, "Asset not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Store error: {}", e)).into_response(),
    };

    // Metadata
    let metadata = store.get_business_metadata(asset_id).await.unwrap_or(None);
    let is_discoverable = metadata.as_ref().map(|m| m.discoverable).unwrap_or(false);

    // Permission Check
    // If not admin and not discoverable, check catalog read permission
    if !crate::authz::is_admin(&session.role) && !is_discoverable {
        let catalogs = store.list_catalogs(tenant_id).await.unwrap_or_default();
        let catalog_id = catalogs.iter().find(|c| c.name == catalog_name).map(|c| c.id);
        
        let has_perm = if let Some(cid) = catalog_id {
            match crate::authz::check_permission(
                &store, 
                &session, 
                &Action::Read, 
                &PermissionScope::Catalog { catalog_id: cid }
            ).await {
                Ok(v) => v,
                Err(_) => false,
            }
        } else {
            false
        };

        if !has_perm {
            return (StatusCode::FORBIDDEN, "Access denied").into_response();
        }
    }
    
    (StatusCode::OK, Json(serde_json::json!({
        "asset": asset,
        "metadata": metadata,
        "catalog": catalog_name,
        "namespace": namespace
    }))).into_response()
}
