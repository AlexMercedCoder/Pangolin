use axum::{
    extract::{Path, State, Extension},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use pangolin_store::CatalogStore;
use pangolin_core::model::{Asset, AssetType};
use uuid::Uuid;
use crate::auth::TenantId;
use crate::iceberg_handlers::{AppState, parse_table_identifier};

#[derive(Deserialize)]
pub struct CreateViewRequest {
    name: String,
    sql: String,
    dialect: Option<String>,
    properties: Option<std::collections::HashMap<String, String>>,
}

#[derive(Serialize)]
pub struct ViewResponse {
    name: String,
    sql: String,
    dialect: String,
    properties: std::collections::HashMap<String, String>,
}

impl From<Asset> for ViewResponse {
    fn from(asset: Asset) -> Self {
        Self {
            name: asset.name,
            sql: asset.properties.get("sql").cloned().unwrap_or_default(),
            dialect: asset.properties.get("dialect").cloned().unwrap_or_else(|| "ansi".to_string()),
            properties: asset.properties,
        }
    }
}

pub async fn create_view(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path((prefix, namespace)): Path<(String, String)>,
    Json(payload): Json<CreateViewRequest>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix;
    
    let (view_name, branch_from_name) = parse_table_identifier(&payload.name);
    let branch = branch_from_name.unwrap_or("main".to_string());
    
    // Parse namespace
    let namespace_parts: Vec<String> = namespace.split('\x1F').map(|s| s.to_string()).collect();

    let mut properties = payload.properties.unwrap_or_default();
    properties.insert("sql".to_string(), payload.sql);
    properties.insert("dialect".to_string(), payload.dialect.unwrap_or_else(|| "ansi".to_string()));

    let asset = Asset {
        name: view_name.clone(),
        kind: AssetType::View,
        location: "".to_string(), // Views might not have a physical location, or we store the SQL path?
        properties,
    };

    match store.create_asset(tenant_id, &catalog_name, Some(branch), namespace_parts, asset.clone()).await {
        Ok(_) => (StatusCode::CREATED, Json(ViewResponse::from(asset))).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

pub async fn get_view(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path((prefix, namespace, view)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix;
    
    let (view_name, branch_from_name) = parse_table_identifier(&view);
    let branch = branch_from_name.unwrap_or("main".to_string());
    
    let namespace_parts: Vec<String> = namespace.split('\x1F').map(|s| s.to_string()).collect();

    match store.get_asset(tenant_id, &catalog_name, Some(branch), namespace_parts, view_name).await {
        Ok(Some(asset)) => {
            if asset.kind == AssetType::View {
                (StatusCode::OK, Json(ViewResponse::from(asset))).into_response()
            } else {
                (StatusCode::NOT_FOUND, "Asset is not a view").into_response()
            }
        },
        Ok(None) => (StatusCode::NOT_FOUND, "View not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}
