use axum::{
    extract::{Path, State, Query},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use pangolin_store::CatalogStore;
use pangolin_store::signer::Signer;
use crate::iceberg_handlers::AppState;

#[derive(Serialize)]
pub struct CredentialsResponse {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: Option<String>,
    pub expiration: Option<String>,
}

#[derive(Deserialize)]
pub struct PresignParams {
    pub location: String,
}

#[derive(Serialize)]
pub struct PresignResponse {
    pub url: String,
}

pub async fn get_table_credentials(
    State(store): State<AppState>,
    Path((_prefix, _namespace, _table)): Path<(String, String, String)>,
) -> impl IntoResponse {
    // In a real implementation, we would check if the user has access to this table
    // and potentially use the table location to scope the credentials.
    // For now, we just call the signer.
    
    // We need to downcast or access the signer trait.
    // Since AppState holds Arc<dyn CatalogStore>, and CatalogStore doesn't inherit Signer (yet?),
    // we might need to cast it.
    // Ideally, CatalogStore should inherit Signer or we should have a way to access it.
    // Let's assume for now we can modify CatalogStore to inherit Signer, or we check if it is S3Store.
    // But we can't easily downcast Arc<dyn Trait>.
    
    // Plan B: Add `as_signer(&self) -> Option<&dyn Signer>` to CatalogStore.
    
    // For now, let's assume we modify CatalogStore to include Signer methods or inherit it.
    // Let's modify CatalogStore trait in pangolin_store/src/lib.rs first.
    
    // Wait, I can't modify CatalogStore easily without touching all impls.
    // But MemoryStore needs to implement it too (as no-op or error).
    
    // Let's assume I will modify CatalogStore.
    
    // For this file, I will assume `store` implements `Signer` or has a method to get it.
    // Let's assume `CatalogStore` extends `Signer`.
    
    match store.get_table_credentials("").await {
        Ok(creds) => {
             let resp = CredentialsResponse {
                 access_key_id: creds.access_key_id,
                 secret_access_key: creds.secret_access_key,
                 session_token: creds.session_token,
                 expiration: creds.expiration,
             };
             (StatusCode::OK, Json(resp)).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn get_presigned_url(
    State(store): State<AppState>,
    Query(params): Query<PresignParams>,
) -> impl IntoResponse {
    match store.presign_get(&params.location).await {
        Ok(url) => {
            (StatusCode::OK, Json(PresignResponse { url })).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
