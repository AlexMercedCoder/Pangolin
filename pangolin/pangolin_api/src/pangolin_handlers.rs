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

// Placeholder for AppState
pub type AppState = Arc<dyn CatalogStore + Send + Sync>;

#[derive(Deserialize)]
pub struct CreateBranchRequest {
    name: String,
    branch_type: Option<String>, // "ingest" or "experimental", defaults to experimental
    catalog: Option<String>, // Optional catalog name, defaults to "default"
    from_branch: Option<String>, // Defaults to "main"
    assets: Option<Vec<String>>, // List of asset names to include. If None, include all? Or empty? User said "only to specified tables".
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
    Query(params): Query<CreateBranchRequest>, // Reuse struct for query params? No, let's make a new one or just use default
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    // For now assume "default" catalog if not specified in query (which we haven't implemented yet)
    // Or maybe we should add catalog to path?
    // Let's assume "default" for now to keep it simple, or add a query param.
    let catalog_name = "default";
    
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
    Json(payload): Json<CreateBranchRequest>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = payload.catalog.as_deref().unwrap_or("default");
    let from_branch = payload.from_branch.as_deref().unwrap_or("main");
    
    let b_type = match payload.branch_type.as_deref() {
        Some("ingest") => BranchType::Ingest,
        _ => BranchType::Experimental,
    };

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
        branch_type: b_type,
        assets: branch_assets,
    };

    match store.create_branch(tenant_id, catalog_name, branch.clone()).await {
        Ok(_) => (StatusCode::OK, Json(BranchResponse::from(branch))).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

pub async fn get_branch(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = "default"; // TODO: Support catalog in path
    
    match store.get_branch(tenant_id, catalog_name, name).await {
        Ok(Some(branch)) => (StatusCode::OK, Json(BranchResponse::from(branch))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Branch not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}
