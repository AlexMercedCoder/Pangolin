use axum::{
    extract::{Path, State, Query, Extension},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use pangolin_store::CatalogStore;
use pangolin_core::model::{Branch, BranchType};
use uuid::Uuid;
use crate::auth::TenantId;
use pangolin_core::permission::{PermissionScope, Action};
use pangolin_core::user::{UserSession, UserRole};

// Placeholder for AppState
pub type AppState = Arc<dyn CatalogStore + Send + Sync>;

#[derive(Deserialize)]
pub struct CreateBranchRequest {
    name: String,
    branch_type: Option<String>, // "ingest" or "experimental", defaults to experimental
    catalog: Option<String>, // Optional catalog name, defaults to "default"
    from_branch: Option<String>, // Defaults to "main"
    assets: Option<Vec<String>>, // List of asset names to include.
}

#[derive(Deserialize)]
pub struct ListBranchParams {
    name: Option<String>,
    catalog: Option<String>,
}

#[derive(Deserialize)]
pub struct MergeBranchRequest {
    pub source_branch: String,
    pub target_branch: String,
    pub catalog: Option<String>,
}

#[derive(Serialize)]
pub struct BranchResponse {
    name: String,
    head_commit_id: Option<Uuid>,
    branch_type: String,
    assets: Vec<String>,
}

impl From<Branch> for BranchResponse {
    fn from(b: Branch) -> Self {
        Self {
            name: b.name,
            head_commit_id: b.head_commit_id,
            branch_type: match b.branch_type {
                BranchType::Ingest => "ingest".to_string(),
                BranchType::Experimental => "experimental".to_string(),
            },
            assets: b.assets,
        }
    }
}

pub async fn list_branches(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Query(params): Query<ListBranchParams>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    
    // Use catalog from query params or default to "default"
    let catalog_name_string = params.catalog.clone().unwrap_or_else(|| "default".to_string());
    let catalog_name = catalog_name_string.as_str();
    
    match store.list_branches(tenant_id, catalog_name).await {
        Ok(branches) => {
            let resp: Vec<BranchResponse> = branches.into_iter().map(|b| b.into()).collect();
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

pub async fn create_branch(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Json(payload): Json<CreateBranchRequest>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = payload.catalog.as_deref().unwrap_or("default");
    
    // Resolve catalog ID for permission check
    let catalog = match store.get_catalog(tenant_id, catalog_name.to_string()).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "Catalog not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    };

    let b_type = match payload.branch_type.as_deref() {
        Some("ingest") => BranchType::Ingest,
        _ => BranchType::Experimental,
    };
    
    let required_action = match b_type {
        BranchType::Ingest => Action::IngestBranching,
        BranchType::Experimental => Action::ExperimentalBranching,
    };

    let scope = PermissionScope::Catalog { catalog_id: catalog.id };
    
    match crate::authz::check_permission(&store, &session, &required_action, &scope).await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, "Forbidden").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Permission check failed").into_response(),
    }

    let from_branch = payload.from_branch.as_deref().unwrap_or("main");

    // Logic for partial branching:
    // 1. Create the branch object.
    // 2. If assets are specified, copy them from `from_branch`.
    
    let mut branch_assets = vec![];

    if let Some(assets_to_copy) = &payload.assets {
        for asset_name in assets_to_copy {
            // We need the namespace for the asset. 
            // The current API `CreateBranchRequest` only lists asset names, but assets are scoped by namespace.
            // This is a limitation. The user request said "specified tables".
            // Assuming for now that `assets` contains "namespace.table" strings or we need to change the request to be more structured.
            // Let's assume "namespace.table" format for simplicity in this MVP.
            
            let parts: Vec<&str> = asset_name.split('.').collect();
            if parts.len() < 2 {
                continue; // Skip invalid format
            }
            let table_name = parts.last().unwrap().to_string();
            let namespace_parts = parts[0..parts.len()-1].iter().map(|s| s.to_string()).collect::<Vec<String>>();
            
            // Get asset from source branch
            if let Ok(Some(asset)) = store.get_asset(tenant_id, catalog_name, Some(from_branch.to_string()), namespace_parts.clone(), table_name.clone()).await {
                // Create asset in new branch
                if let Ok(_) = store.create_asset(tenant_id, catalog_name, Some(payload.name.clone()), namespace_parts, asset).await {
                    branch_assets.push(asset_name.clone());
                }
            }
        }
    } else {
        // If no assets specified, maybe we copy ALL assets? 
        // Or create an empty branch?
        // The user said "a branch shouldn't apply to the whole catalog but only to specified tables".
        // This implies if you don't specify tables, you might get an empty branch or it's an error?
        // Let's assume empty branch if None.
    }

    let branch = Branch {
        name: payload.name.clone(),
        head_commit_id: None,
        branch_type: b_type.clone(),
        assets: branch_assets,
    };

    match store.create_branch(tenant_id, catalog_name, branch).await {
        Ok(_) => {
            // Audit Log
            let _ = store.log_audit_event(tenant_id, pangolin_core::audit::AuditLogEntry::new(
                tenant_id,
                session.username.clone(), // Get user from auth context
                "create_branch".to_string(),
                format!("{}/{}", catalog_name, payload.name),
                None
            )).await;
            
            (StatusCode::CREATED, Json(BranchResponse {
                name: payload.name,
                head_commit_id: None,
                branch_type: match b_type {
                    BranchType::Ingest => "ingest".to_string(),
                    BranchType::Experimental => "experimental".to_string(),
                },
                assets: vec![], // Assets are not returned in this simplified response
            })).into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

pub async fn get_branch(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path(name): Path<String>,
    Query(params): Query<ListBranchParams>, // Reusing ListBranchParams which has optional catalog
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    
    let catalog_name_string = params.catalog.clone().unwrap_or_else(|| "default".to_string());
    let catalog_name = catalog_name_string.as_str();
    
    match store.get_branch(tenant_id, catalog_name, name).await {
        Ok(Some(branch)) => (StatusCode::OK, Json(BranchResponse::from(branch))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Branch not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

pub async fn merge_branch(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Json(payload): Json<MergeBranchRequest>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = payload.catalog.as_deref().unwrap_or("default");

    // Attempt to detect base commit
    let base_commit_id = find_common_ancestor(
        &store,
        tenant, // Pass extension which wraps TenantId
        catalog_name,
        &payload.source_branch,
        &payload.target_branch
    ).await;

    // Create a merge operation
    let operation = pangolin_core::model::MergeOperation::new(
        tenant_id,
        catalog_name.to_string(),
        payload.source_branch.clone(),
        payload.target_branch.clone(),
        base_commit_id, 
        session.user_id,
    );

    // Store the merge operation
    if let Err(e) = store.create_merge_operation(operation.clone()).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Failed to create merge operation: {}", e)
        }))).into_response();
    }

    // Detect conflicts
    let detector = crate::conflict_detector::ConflictDetector::new(store.clone());
    let conflicts = match detector.detect_conflicts(&operation).await {
        Ok(c) => c,
        Err(e) => {
            let _ = store.abort_merge_operation(operation.id).await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": format!("Failed to detect conflicts: {}", e)
            }))).into_response();
        }
    };

    // If conflicts detected, store them and return conflict response
    if !conflicts.is_empty() {
        for conflict in &conflicts {
            if let Err(e) = store.create_merge_conflict(conflict.clone()).await {
                eprintln!("Failed to store conflict: {}", e);
            }
            if let Err(e) = store.add_conflict_to_operation(operation.id, conflict.id).await {
                eprintln!("Failed to link conflict to operation: {}", e);
            }
        }

        // Update operation status to Conflicted
        let _ = store.update_merge_operation_status(
            operation.id,
            pangolin_core::model::MergeStatus::Conflicted
        ).await;

        return (StatusCode::CONFLICT, Json(serde_json::json!({
            "status": "conflicted",
            "operation_id": operation.id,
            "conflicts": conflicts.len(),
            "message": format!("Merge has {} conflicts that need resolution", conflicts.len())
        }))).into_response();
    }

    // No conflicts - proceed with merge
    match store.merge_branch(tenant_id, catalog_name, payload.source_branch.clone(), payload.target_branch.clone()).await {
        Ok(_) => {
            // Complete the merge operation
            let commit_id = uuid::Uuid::new_v4(); // In real implementation, get actual commit ID
            let _ = store.complete_merge_operation(operation.id, commit_id).await;

            // Audit Log
            let _ = store.log_audit_event(tenant_id, pangolin_core::audit::AuditLogEntry::new(
                tenant_id,
                session.username.clone(),
                "merge_branch".to_string(),
                format!("{}/{}->{}", catalog_name, payload.source_branch, payload.target_branch),
                None
            )).await;
            
            (StatusCode::OK, Json(serde_json::json!({
                "status": "merged",
                "operation_id": operation.id,
                "commit_id": commit_id
            }))).into_response()
        },
        Err(e) => {
            let _ = store.abort_merge_operation(operation.id).await;
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": e.to_string()
            }))).into_response()
        }
    }
}

pub async fn list_commits(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path(branch_name): Path<String>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = "default"; // TODO: Support catalog in path

    // Get branch to find head commit
    let branch = match store.get_branch(tenant_id, catalog_name, branch_name.clone()).await {
        Ok(Some(b)) => b,
        Ok(None) => return (StatusCode::NOT_FOUND, "Branch not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get branch").into_response(),
    };

    let mut commits = Vec::new();
    let mut current_commit_id = branch.head_commit_id;

    while let Some(commit_id) = current_commit_id {
        match store.get_commit(tenant_id, commit_id).await {
            Ok(Some(commit)) => {
                current_commit_id = commit.parent_id;
                commits.push(commit);
            },
            Ok(None) => break, // Should not happen if consistency is maintained
            Err(_) => break,
        }
    }

    (StatusCode::OK, Json(commits)).into_response()
}

#[derive(Deserialize)]
pub struct CreateTagRequest {
    name: String,
    commit_id: Uuid,
}

#[derive(Serialize)]
pub struct TagResponse {
    name: String,
    commit_id: Uuid,
}

impl From<pangolin_core::model::Tag> for TagResponse {
    fn from(t: pangolin_core::model::Tag) -> Self {
        Self {
            name: t.name,
            commit_id: t.commit_id,
        }
    }
}

pub async fn list_tags(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = "default"; // TODO: Support catalog in path

    match store.list_tags(tenant_id, catalog_name).await {
        Ok(tags) => {
            let resp: Vec<TagResponse> = tags.into_iter().map(|t| t.into()).collect();
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

pub async fn create_tag(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Json(payload): Json<CreateTagRequest>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = "default";

    let tag = pangolin_core::model::Tag {
        name: payload.name.clone(),
        commit_id: payload.commit_id,
    };

    match store.create_tag(tenant_id, catalog_name, tag.clone()).await {
        Ok(_) => (StatusCode::OK, Json(TagResponse::from(tag))).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

pub async fn delete_tag(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = "default";

    match store.delete_tag(tenant_id, catalog_name, name).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

pub async fn list_audit_events(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    match store.list_audit_events(tenant_id).await {
        Ok(events) => (StatusCode::OK, Json(events)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

// Catalog Management
#[derive(Deserialize)]
pub struct CreateCatalogRequest {
    name: String,
    warehouse_name: Option<String>, // Reference to warehouse for credential vending
    storage_location: Option<String>, // Base path for this catalog in the warehouse
    properties: Option<std::collections::HashMap<String, String>>,
}

#[derive(Deserialize)]
pub struct UpdateCatalogRequest {
    warehouse_name: Option<String>,
    storage_location: Option<String>,
    properties: Option<std::collections::HashMap<String, String>>,
}

#[derive(Serialize)]
pub struct CatalogResponse {
    id: Uuid,
    name: String,
    warehouse_name: Option<String>,
    storage_location: Option<String>,
    properties: std::collections::HashMap<String, String>,
}

impl From<pangolin_core::model::Catalog> for CatalogResponse {
    fn from(c: pangolin_core::model::Catalog) -> Self {
        Self {
            id: c.id,
            name: c.name,
            warehouse_name: c.warehouse_name,
            storage_location: c.storage_location,
            properties: c.properties,
        }
    }
}

pub async fn list_catalogs(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
) -> impl IntoResponse {
    tracing::info!("list_catalogs called with tenant_id: {}", tenant.0);
    match store.list_catalogs(tenant.0).await {
        Ok(catalogs) => {
            tracing::info!("list_catalogs returning {} catalogs for tenant {}", catalogs.len(), tenant.0);
            let resp: Vec<CatalogResponse> = catalogs.into_iter().map(|c| c.into()).collect();
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

pub async fn create_catalog(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Json(payload): Json<CreateCatalogRequest>,
) -> impl IntoResponse {
    if session.role == UserRole::Root {
         return (StatusCode::FORBIDDEN, "Root user cannot create catalogs. Please login as Tenant Admin.").into_response();
    }

    let tenant_id = tenant.0;
    tracing::info!("create_catalog: tenant_id={}, catalog_name={}", tenant_id, payload.name);
    
    // Validate warehouse exists if specified
    if let Some(ref warehouse_name) = payload.warehouse_name {
        if !warehouse_name.is_empty() {
             match store.get_warehouse(tenant_id, warehouse_name.clone()).await {
                Ok(Some(_)) => {}, // Warehouse exists, continue
                Ok(None) => return (StatusCode::BAD_REQUEST, "Warehouse not found").into_response(),
                Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to validate warehouse").into_response(),
            }
        }
    }
    
    let catalog = pangolin_core::model::Catalog {
        id: Uuid::new_v4(),
        name: payload.name.clone(),
        catalog_type: pangolin_core::model::CatalogType::Local,
        warehouse_name: payload.warehouse_name.clone().filter(|n| !n.is_empty()), // Filter empty string to None
        storage_location: payload.storage_location.clone(),
        federated_config: None,
        properties: payload.properties.clone().unwrap_or_default(),
    };

    match store.create_catalog(tenant_id, catalog.clone()).await {
        Ok(_) => (StatusCode::CREATED, Json(CatalogResponse::from(catalog))).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

pub async fn get_catalog(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match store.get_catalog(tenant.0, name).await {
        Ok(Some(catalog)) => (StatusCode::OK, Json(CatalogResponse::from(catalog))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Catalog not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

pub async fn delete_catalog(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    tracing::info!("delete_catalog: tenant_id={}, catalog_name={}", tenant_id, name);
    
    match store.delete_catalog(tenant_id, name.clone()).await {
        Ok(_) => {
            tracing::info!("Successfully deleted catalog: {}", name);
            StatusCode::NO_CONTENT.into_response()
        },
        Err(e) => {
            tracing::error!("Failed to delete catalog {}: {}", name, e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        }
    }
}

pub async fn update_catalog(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path(name): Path<String>,
    Json(payload): Json<UpdateCatalogRequest>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    
    let updates = pangolin_core::model::CatalogUpdate {
        warehouse_name: payload.warehouse_name,
        storage_location: payload.storage_location,
        properties: payload.properties,
    };
    
    match store.update_catalog(tenant_id, name, updates).await {
        Ok(catalog) => (StatusCode::OK, Json(CatalogResponse::from(catalog))).into_response(),
        Err(e) => {
            if e.to_string().contains("not found") {
                (StatusCode::NOT_FOUND, "Catalog not found").into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            }
        }
    }
}

/// Helper: Find the common ancestor commit between two branches
async fn find_common_ancestor(
    store: &Arc<dyn CatalogStore + Send + Sync>,
    tenant_id: crate::auth::TenantId,
    catalog_name: &str,
    source_branch_name: &str,
    target_branch_name: &str,
) -> Option<uuid::Uuid> {
    // Get branches
    let source_branch = store.get_branch(tenant_id.0, catalog_name, source_branch_name.to_string()).await.ok()??;
    let target_branch = store.get_branch(tenant_id.0, catalog_name, target_branch_name.to_string()).await.ok()??;

    // Get commit chains
    let source_chain = get_commit_chain(store, tenant_id, source_branch.head_commit_id).await;
    let target_chain = get_commit_chain(store, tenant_id, target_branch.head_commit_id).await;

    // Find first common commit
    for commit_id in source_chain {
        if target_chain.contains(&commit_id) {
            return Some(commit_id);
        }
    }

    None
}

/// Helper: Get the chain of commit IDs starting from a head commit
async fn get_commit_chain(
    store: &Arc<dyn CatalogStore + Send + Sync>,
    tenant_id: crate::auth::TenantId,
    start_commit_id: Option<uuid::Uuid>,
) -> Vec<uuid::Uuid> {
    let mut chain = Vec::new();
    let mut current_id = start_commit_id;

    while let Some(id) = current_id {
        chain.push(id);
        if let Ok(Some(commit)) = store.get_commit(tenant_id.0, id).await {
            current_id = commit.parent_id;
        } else {
            break;
        }
    }

    chain
}
