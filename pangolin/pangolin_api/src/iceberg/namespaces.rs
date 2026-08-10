use super::types::*;
use super::{
    check_and_forward_if_federated, forbidden, internal, namespace_already_exists,
    no_such_namespace, AppState,
};
use crate::auth::TenantId;
use crate::authz::check_permission;
use axum::{
    extract::{Extension, Path, Query, State},
    http::{HeaderMap, Method, StatusCode},
    response::IntoResponse,
    Json,
};
use bytes::Bytes;
use pangolin_core::model::Namespace;
use pangolin_core::permission::{Action, PermissionScope};
use pangolin_core::user::UserSession;
use pangolin_store::PaginationParams;

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
    Query(page): Query<IcebergPageParams>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix.clone();
    tracing::info!(
        "list_namespaces: tenant_id={}, catalog_name={}",
        tenant_id,
        catalog_name
    );

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
    )
    .await
    {
        return response;
    }

    // Local catalog handling
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return no_such_namespace(&catalog_name),
        Err(e) => {
            tracing::error!(error = %e, "list_namespaces: failed to load catalog");
            return internal("Failed to load catalog");
        }
    };

    // Check Permissions
    let scope = PermissionScope::Catalog {
        catalog_id: catalog.id,
    };
    match check_permission(&store, &session, &Action::List, &scope).await {
        Ok(true) => (),
        Ok(false) => return forbidden("Forbidden"),
        Err(e) => {
            tracing::error!(error = %e, "list_namespaces: permission check failed");
            return internal("Permission check failed");
        }
    }

    let (offset, limit) = page.resolve();
    let pagination = PaginationParams {
        limit: Some(limit as usize),
        offset: Some(offset as usize),
    };

    match store
        .list_namespaces(tenant_id, &catalog_name, params.parent, Some(pagination))
        .await
    {
        Ok(namespaces) => {
            let returned = namespaces.len();
            let ns_list: Vec<Vec<String>> = namespaces.into_iter().map(|n| n.name).collect();
            (
                StatusCode::OK,
                Json(ListNamespacesResponse {
                    namespaces: ns_list,
                    next_page_token: next_page_token(returned, offset, limit),
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "list_namespaces: failed to list namespaces");
            internal("Failed to list namespaces")
        }
    }
}

/// Load a namespace's metadata (`loadNamespaceMetadata`).
///
/// Part of completing the Iceberg REST surface: this endpoint was on the
/// README's "not implemented" list, so clients could create a namespace and set
/// its properties but never read them back.
#[utoipa::path(
    get,
    path = "/v1/{prefix}/namespaces/{namespace}",
    tag = "Iceberg REST",
    params(
        ("prefix" = String, Path, description = "Catalog name"),
        ("namespace" = String, Path, description = "Namespace name")
    ),
    responses(
        (status = 200, description = "Namespace metadata", body = CreateNamespaceResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Namespace not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn load_namespace_metadata(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Path((prefix, namespace)): Path<(String, String)>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix;

    let path = format!("/namespaces/{}", namespace);
    if let Some(response) = check_and_forward_if_federated(
        &store,
        tenant_id,
        &catalog_name,
        Method::GET,
        &path,
        None,
        HeaderMap::new(),
    )
    .await
    {
        return response;
    }

    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return no_such_namespace(&catalog_name),
        Err(e) => {
            tracing::error!(error = %e, "load_namespace_metadata: failed to load catalog");
            return internal("Failed to load catalog");
        }
    };

    let (namespace_parts, _branch) = parse_namespace(&namespace);

    let scope = PermissionScope::Namespace {
        catalog_id: catalog.id,
        namespace: namespace_parts.join("."),
    };
    match check_permission(&store, &session, &Action::Read, &scope).await {
        Ok(true) => (),
        Ok(false) => return forbidden("Forbidden"),
        Err(e) => {
            tracing::error!(error = %e, "load_namespace_metadata: permission check failed");
            return internal("Permission check failed");
        }
    }

    match store
        .get_namespace(tenant_id, &catalog_name, namespace_parts.clone())
        .await
    {
        Ok(Some(ns)) => (
            StatusCode::OK,
            Json(CreateNamespaceResponse {
                namespace: ns.name,
                properties: ns.properties,
            }),
        )
            .into_response(),
        Ok(None) => no_such_namespace(&namespace_parts.join(".")),
        Err(e) => {
            tracing::error!(error = %e, "load_namespace_metadata: failed to load namespace");
            internal("Failed to load namespace")
        }
    }
}

/// Check whether a namespace exists (`namespaceExists`).
///
/// `HEAD` with an empty body, per the spec. Also on the README's
/// "not implemented" list.
#[utoipa::path(
    head,
    path = "/v1/{prefix}/namespaces/{namespace}",
    tag = "Iceberg REST",
    params(
        ("prefix" = String, Path, description = "Catalog name"),
        ("namespace" = String, Path, description = "Namespace name")
    ),
    responses(
        (status = 204, description = "Namespace exists"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Namespace not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn namespace_exists(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Path((prefix, namespace)): Path<(String, String)>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix;

    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!(error = %e, "namespace_exists: failed to load catalog");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let (namespace_parts, _branch) = parse_namespace(&namespace);

    let scope = PermissionScope::Namespace {
        catalog_id: catalog.id,
        namespace: namespace_parts.join("."),
    };
    match check_permission(&store, &session, &Action::Read, &scope).await {
        Ok(true) => (),
        Ok(false) => return StatusCode::FORBIDDEN.into_response(),
        Err(e) => {
            tracing::error!(error = %e, "namespace_exists: permission check failed");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    match store
        .get_namespace(tenant_id, &catalog_name, namespace_parts)
        .await
    {
        Ok(Some(_)) => StatusCode::NO_CONTENT.into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!(error = %e, "namespace_exists: failed to load namespace");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
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

    tracing::info!(
        "create_namespace: tenant_id={}, catalog_name={}",
        tenant_id,
        catalog_name
    );

    // B16k: federated forwarding was missing here (and on delete and the
    // namespace tree) although `list_namespaces` had it. On a `Federated`
    // catalog, creating a namespace built a *local shadow* and returned 200
    // while `GET` listed the remote - the two views diverged permanently, and
    // delete reported success for a namespace still present upstream.
    let path = "/namespaces".to_string();
    let body_bytes = serde_json::to_vec(&payload).ok().map(Bytes::from);
    if let Some(response) = check_and_forward_if_federated(
        &store,
        tenant_id,
        &catalog_name,
        Method::POST,
        &path,
        body_bytes,
        HeaderMap::new(),
    )
    .await
    {
        return response;
    }

    // Resolve catalog ID
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return no_such_namespace(&catalog_name),
        Err(e) => {
            tracing::error!(error = %e, "create_namespace: failed to load catalog");
            return internal("Failed to load catalog");
        }
    };

    // Check Permissions
    let scope = PermissionScope::Catalog {
        catalog_id: catalog.id,
    };
    match check_permission(&store, &session, &Action::Create, &scope).await {
        Ok(true) => (),
        Ok(false) => return forbidden("Forbidden"),
        Err(e) => {
            tracing::error!(error = %e, "create_namespace: permission check failed");
            return internal("Permission check failed");
        }
    }

    let ns = Namespace {
        name: payload.namespace.clone(),
        properties: payload.properties.unwrap_or_default(),
    };

    // Report a conflict rather than silently overwriting an existing namespace's
    // properties, which is what the create path did on backends whose insert is
    // an upsert.
    match store
        .get_namespace(tenant_id, &catalog_name, ns.name.clone())
        .await
    {
        Ok(Some(_)) => return namespace_already_exists(&ns.name.join(".")),
        Ok(None) => {}
        Err(e) => {
            tracing::error!(error = %e, "create_namespace: existence check failed");
            return internal("Failed to check namespace");
        }
    }

    match store
        .create_namespace(tenant_id, &catalog_name, ns.clone())
        .await
    {
        Ok(_) => {
            // Audit Log
            let _ = store
                .log_audit_event(
                    tenant_id,
                    pangolin_core::audit::AuditLogEntry::success(
                        tenant_id,
                        Some(session.user_id),
                        session.username.clone(),
                        pangolin_core::audit::AuditAction::CreateNamespace,
                        pangolin_core::audit::ResourceType::Namespace,
                        None,
                        format!("{}/{}", catalog_name, ns.name.join(".")),
                    ),
                )
                .await;

            (
                StatusCode::OK,
                Json(CreateNamespaceResponse {
                    namespace: ns.name,
                    properties: ns.properties,
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "create_namespace: failed to create namespace");
            internal("Failed to create namespace")
        }
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

    // Federated forwarding (B16k) - see `create_namespace`.
    let path = format!("/namespaces/{}", namespace);
    if let Some(response) = check_and_forward_if_federated(
        &store,
        tenant_id,
        &catalog_name,
        Method::DELETE,
        &path,
        None,
        HeaderMap::new(),
    )
    .await
    {
        return response;
    }

    // Resolve catalog ID
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return no_such_namespace(&catalog_name),
        Err(e) => {
            tracing::error!(error = %e, "delete_namespace: failed to load catalog");
            return internal("Failed to load catalog");
        }
    };

    let (namespace_parts, _branch) = parse_namespace(&namespace);

    // Check Permissions
    let scope = PermissionScope::Namespace {
        catalog_id: catalog.id,
        namespace: namespace_parts.join("."),
    };

    match check_permission(&store, &session, &Action::Delete, &scope).await {
        Ok(true) => (),
        Ok(false) => return forbidden("Forbidden"),
        Err(e) => {
            tracing::error!(error = %e, "delete_namespace: permission check failed");
            return internal("Permission check failed");
        }
    }

    match store
        .delete_namespace(tenant_id, &catalog_name, namespace_parts.clone())
        .await
    {
        Ok(_) => {
            // Audit Log
            let _ = store
                .log_audit_event(
                    tenant_id,
                    pangolin_core::audit::AuditLogEntry::success(
                        tenant_id,
                        Some(session.user_id),
                        session.username.clone(),
                        pangolin_core::audit::AuditAction::DeleteNamespace,
                        pangolin_core::audit::ResourceType::Namespace,
                        None,
                        format!("{}/{}", catalog_name, namespace_parts.join(".")),
                    ),
                )
                .await;

            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            tracing::debug!(error = %e, "delete_namespace: namespace not deleted");
            no_such_namespace(&namespace_parts.join("."))
        }
    }
}

/// Update a namespace's properties.
///
/// Two fixes here:
///
/// * **B0d** - the handler bound `Extension(_session)` (deliberately discarding
///   it) and never called `check_permission`, and never resolved the catalog at
///   all. Any tenant member could rewrite any namespace's properties, including
///   `location`, which later table creation derives paths from.
/// * **B16h** - `removals` were silently ignored. A request carrying removals
///   got `200 OK` with `removed: []` / `missing: []` while nothing was removed:
///   exactly the "silent success" failure class 0.6.0 set out to eliminate. The
///   three response lists are now reported honestly.
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
    )
    .await
    {
        return response;
    }

    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return no_such_namespace(&catalog_name),
        Err(e) => {
            tracing::error!(error = %e, "update_namespace_properties: failed to load catalog");
            return internal("Failed to load catalog");
        }
    };

    let (namespace_parts, _branch) = parse_namespace(&namespace);

    let scope = PermissionScope::Namespace {
        catalog_id: catalog.id,
        namespace: namespace_parts.join("."),
    };
    match check_permission(&store, &session, &Action::Write, &scope).await {
        Ok(true) => (),
        Ok(false) => return forbidden("Forbidden"),
        Err(e) => {
            tracing::error!(error = %e, "update_namespace_properties: permission check failed");
            return internal("Permission check failed");
        }
    }

    let updates = payload.updates.unwrap_or_default();
    let removals = payload.removals.unwrap_or_default();

    // The spec rejects a key that appears in both lists rather than picking a
    // winner.
    if let Some(conflict) = removals.iter().find(|k| updates.contains_key(*k)) {
        return super::bad_request(&format!(
            "Property {conflict} appears in both updates and removals"
        ));
    }

    if updates.is_empty() && removals.is_empty() {
        return (
            StatusCode::OK,
            Json(UpdateNamespacePropertiesResponse {
                updated: vec![],
                removed: vec![],
                missing: vec![],
            }),
        )
            .into_response();
    }

    // Read-modify-write: removals cannot be expressed by the merging store
    // method, so the resulting map is computed here and written wholesale.
    let existing = match store
        .get_namespace(tenant_id, &catalog_name, namespace_parts.clone())
        .await
    {
        Ok(Some(ns)) => ns.properties,
        Ok(None) => return no_such_namespace(&namespace_parts.join(".")),
        Err(e) => {
            tracing::error!(error = %e, "update_namespace_properties: failed to load namespace");
            return internal("Failed to load namespace");
        }
    };

    let mut properties = existing;
    let mut removed = Vec::new();
    let mut missing = Vec::new();
    for key in &removals {
        if properties.remove(key).is_some() {
            removed.push(key.clone());
        } else {
            missing.push(key.clone());
        }
    }

    let updated: Vec<String> = updates.keys().cloned().collect();
    properties.extend(updates);

    match store
        .replace_namespace_properties(
            tenant_id,
            &catalog_name,
            namespace_parts.clone(),
            properties,
        )
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(UpdateNamespacePropertiesResponse {
                updated,
                removed,
                missing,
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "update_namespace_properties: failed to write properties");
            no_such_namespace(&namespace_parts.join("."))
        }
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

    // Federated forwarding (B16k): without it the tree renders the local shadow
    // while every other view shows the remote.
    if let Some(response) = check_and_forward_if_federated(
        &store,
        tenant_id,
        &catalog_name,
        Method::GET,
        "/namespaces",
        None,
        HeaderMap::new(),
    )
    .await
    {
        return response;
    }

    // Resolve catalog ID
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return no_such_namespace(&catalog_name),
        Err(e) => {
            tracing::error!(error = %e, "list_namespaces_tree: failed to load catalog");
            return internal("Failed to load catalog");
        }
    };

    // Check Permissions
    let scope = PermissionScope::Catalog {
        catalog_id: catalog.id,
    };
    match check_permission(&store, &session, &Action::List, &scope).await {
        Ok(true) => (),
        Ok(false) => return forbidden("Forbidden"),
        Err(e) => {
            tracing::error!(error = %e, "list_namespaces_tree: permission check failed");
            return internal("Permission check failed");
        }
    }

    match store
        .list_namespaces(tenant_id, &catalog_name, None, None)
        .await
    {
        Ok(namespaces) => {
            let mut root_nodes: Vec<NamespaceNode> = Vec::new();

            fn find_or_create_child<'a>(
                nodes: &'a mut Vec<NamespaceNode>,
                name: &str,
                full_path: Vec<String>,
            ) -> &'a mut NamespaceNode {
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
                    current_level =
                        &mut find_or_create_child(current_level, &part, current_path.clone())
                            .children;
                }
            }

            (
                StatusCode::OK,
                Json(ListNamespacesTreeResponse { root: root_nodes }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "list_namespaces_tree: failed to list namespaces");
            internal("Failed to list namespaces")
        }
    }
}
