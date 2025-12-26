use axum::{
    extract::{Path, State, Query, Extension},
    Json,
    response::IntoResponse,
    http::{StatusCode, HeaderMap, Method},
};
use bytes::Bytes;
use pangolin_core::model::Namespace;
use pangolin_core::permission::{PermissionScope, Action};
use pangolin_core::user::UserSession;
use crate::auth::TenantId;
use crate::authz::check_permission;
use super::{check_and_forward_if_federated, AppState};
use super::types::*;

/// List namespaces in a catalog
#[utoipa::path(
    get,
    path = "/v1/{prefix}/namespaces",
    tag = "Iceberg REST",
    params(
        ("prefix" = String, Path, description = "Catalog name"),
        ListNamespaceParams
    ),
    responses(
        (status = 200, description = "List of namespaces", body = ListNamespacesResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Catalog not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_namespaces(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Path(prefix): Path<String>,
    Query(params): Query<ListNamespaceParams>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix.clone();
    tracing::info!("list_namespaces: tenant_id={}, catalog_name={}", tenant_id, catalog_name);
    
    // Check if this is a federated catalog and forward if so
    let path = "/namespaces".to_string(); 
    
    if let Some(response) = check_and_forward_if_federated(
        &store,
        tenant_id,
        &catalog_name,
        Method::GET,
        &path,
        None,
        HeaderMap::new(),
    ).await {
        return response;
    }
    
    // Local catalog handling
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "Catalog not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    };

    // Check Permissions
    let scope = PermissionScope::Catalog { catalog_id: catalog.id };
    match check_permission(&store, &session, &Action::List, &scope).await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, "Forbidden").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Permission check failed: {}", e)).into_response(),
    }
    
    match store.list_namespaces(tenant_id, &catalog_name, params.parent).await {
        Ok(namespaces) => {
            let ns_list: Vec<Vec<String>> = namespaces.into_iter().map(|n| n.name).collect();
            (StatusCode::OK, Json(ListNamespacesResponse { namespaces: ns_list })).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

/// Create a namespace
#[utoipa::path(
    post,
    path = "/v1/{prefix}/namespaces",
    tag = "Iceberg REST",
    params(
        ("prefix" = String, Path, description = "Catalog name")
    ),
    request_body = CreateNamespaceRequest,
    responses(
        (status = 200, description = "Namespace created", body = CreateNamespaceResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Catalog not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_namespace(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Path(prefix): Path<String>,
    Json(payload): Json<CreateNamespaceRequest>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix;
    
    tracing::info!("create_namespace: tenant_id={}, catalog_name={}", tenant_id, catalog_name);

    // Resolve catalog ID
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "Catalog not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    };
    
    // Check Permissions
    let scope = PermissionScope::Catalog { catalog_id: catalog.id };
    match check_permission(&store, &session, &Action::Create, &scope).await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, "Forbidden").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Permission check failed: {}", e)).into_response(),
    }

    let ns = Namespace {
        name: payload.namespace.clone(),
        properties: payload.properties.unwrap_or_default(),
    };

    match store.create_namespace(tenant_id, &catalog_name, ns.clone()).await {
        Ok(_) => {
            // Audit Log
            let _ = store.log_audit_event(tenant_id, pangolin_core::audit::AuditLogEntry::legacy_new(
                tenant_id,
                session.username.clone(),
                "create_namespace".to_string(),
                format!("{}/{}", catalog_name, ns.name.join(".")),
                None
            )).await;

            (StatusCode::OK, Json(CreateNamespaceResponse {
                namespace: ns.name,
                properties: ns.properties,
            })).into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

/// Delete a namespace
#[utoipa::path(
    delete,
    path = "/v1/{prefix}/namespaces/{namespace}",
    tag = "Iceberg REST",
    params(
        ("prefix" = String, Path, description = "Catalog name"),
        ("namespace" = String, Path, description = "Namespace name")
    ),
    responses(
        (status = 204, description = "Namespace deleted"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Namespace not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_namespace(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Path((prefix, namespace)): Path<(String, String)>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix;
    
    // Resolve catalog ID
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "Catalog not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    };
    
    let namespace_parts: Vec<String> = namespace.split('\x1F').map(|s| s.to_string()).collect();
    
    // Check Permissions
    let scope = PermissionScope::Namespace { 
        catalog_id: catalog.id, 
        namespace: namespace_parts.join(".") 
    };
    
    match check_permission(&store, &session, &Action::Delete, &scope).await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, "Forbidden").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Permission check failed: {}", e)).into_response(),
    }

    match store.delete_namespace(tenant_id, &catalog_name, namespace_parts.clone()).await {
        Ok(_) => {
            // Audit Log
            let _ = store.log_audit_event(tenant_id, pangolin_core::audit::AuditLogEntry::legacy_new(
                tenant_id,
                session.username.clone(),
                "delete_namespace".to_string(),
                format!("{}/{}", catalog_name, namespace_parts.join(".")),
                None
            )).await;

            StatusCode::NO_CONTENT.into_response()
        },
        Err(_) => (StatusCode::NOT_FOUND, "Namespace not found").into_response(),
    }
}

/// Update namespace properties
#[utoipa::path(
    post,
    path = "/v1/{prefix}/namespaces/{namespace}/properties",
    tag = "Iceberg REST",
    params(
        ("prefix" = String, Path, description = "Catalog name"),
        ("namespace" = String, Path, description = "Namespace name")
    ),
    request_body = UpdateNamespacePropertiesRequest,
    responses(
        (status = 200, description = "Properties updated", body = UpdateNamespacePropertiesResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Namespace not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_namespace_properties(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Path((prefix, namespace)): Path<(String, String)>,
    Json(payload): Json<UpdateNamespacePropertiesRequest>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix;
    
    // Federated check
    let path = format!("/namespaces/{}/properties", namespace);
    let body_bytes = serde_json::to_vec(&payload).ok().map(Bytes::from);
    
    if let Some(response) = check_and_forward_if_federated(
        &store,
        tenant_id,
        &catalog_name,
        Method::POST,
        &path,
        body_bytes,
        HeaderMap::new(),
    ).await {
         return response;
    }
    let namespace_parts: Vec<String> = namespace.split('\x1F').map(|s| s.to_string()).collect();

    // For MVP, we only support updates. Removals are ignored or TODO.
    if let Some(updates) = payload.updates {
        match store.update_namespace_properties(tenant_id, &catalog_name, namespace_parts, updates.clone()).await {
            Ok(_) => {
                let response = UpdateNamespacePropertiesResponse {
                    updated: updates.keys().cloned().collect(),
                    removed: vec![],
                    missing: vec![],
                };
                (StatusCode::OK, Json(response)).into_response()
            },
            Err(_) => (StatusCode::NOT_FOUND, "Namespace not found").into_response(),
        }
    } else {
        (StatusCode::OK, Json(UpdateNamespacePropertiesResponse { updated: vec![], removed: vec![], missing: vec![] })).into_response()
    }
}

/// List namespace tree structure for a catalog
#[utoipa::path(
    get,
    path = "/api/v1/catalogs/{prefix}/namespaces/tree",
    tag = "Data Explorer",
    params(
        ("prefix" = String, Path, description = "Catalog name")
    ),
    responses(
        (status = 200, description = "Namespace tree structure"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Catalog not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_namespaces_tree(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Path(prefix): Path<String>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix.clone();
    
    // Resolve catalog ID
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "Catalog not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    };

    // Check Permissions
    let scope = PermissionScope::Catalog { catalog_id: catalog.id };
    match check_permission(&store, &session, &Action::List, &scope).await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, "Forbidden").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Permission check failed: {}", e)).into_response(),
    }

    match store.list_namespaces(tenant_id, &catalog_name, None).await {
        Ok(namespaces) => {
            let mut root_nodes: Vec<NamespaceNode> = Vec::new();
            
            fn find_or_create_child<'a>(nodes: &'a mut Vec<NamespaceNode>, name: &str, full_path: Vec<String>) -> &'a mut NamespaceNode {
                if let Some(pos) = nodes.iter().position(|n| n.name == name) {
                    return &mut nodes[pos];
                }
                let new_node = NamespaceNode {
                    name: name.to_string(),
                    full_path,
                    children: Vec::new(),
                };
                nodes.push(new_node);
                nodes.last_mut().unwrap()
            }

            for ns in namespaces {
                let parts = ns.name;
                let mut current_level = &mut root_nodes;
                let mut current_path = Vec::new();
                
                for part in parts {
                    current_path.push(part.clone());
                    current_level = &mut find_or_create_child(current_level, &part, current_path.clone()).children;
                }
            }
            
            (StatusCode::OK, Json(ListNamespacesTreeResponse { root: root_nodes })).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to list namespaces: {}", e)).into_response(),
    }
}
