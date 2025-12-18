use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use pangolin_core::model::{ConflictResolution, MergeConflict, MergeOperation, ResolutionStrategy};
use pangolin_store::CatalogStore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use crate::auth::TenantId;
use pangolin_core::user::UserSession;
use utoipa::ToSchema;

type AppState = Arc<dyn CatalogStore + Send + Sync>;

// Request/Response types
#[derive(Deserialize, ToSchema)]
pub struct ResolveConflictRequest {
    pub strategy: ResolutionStrategy,
    pub resolved_value: Option<serde_json::Value>,
}

#[derive(Serialize, ToSchema)]
pub struct MergeOperationResponse {
    pub id: Uuid,
    pub status: String,
    pub source_branch: String,
    pub target_branch: String,
    pub conflicts_count: usize,
    pub initiated_at: String,
    pub completed_at: Option<String>,
}

/// List all merge operations for a catalog
#[utoipa::path(
    get,
    path = "/api/v1/catalogs/{catalog_name}/merge-operations",
    tag = "Merge Operations",
    params(
        ("catalog_name" = String, Path, description = "Catalog name")
    ),
    responses(
        (status = 200, description = "List of merge operations", body = Vec<MergeOperationResponse>),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_merge_operations(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path(catalog_name): Path<String>,
) -> impl IntoResponse {
    match store.list_merge_operations(tenant.0, &catalog_name).await {
        Ok(operations) => {
            let responses: Vec<MergeOperationResponse> = operations
                .iter()
                .map(|op| MergeOperationResponse {
                    id: op.id,
                    status: format!("{:?}", op.status),
                    source_branch: op.source_branch.clone(),
                    target_branch: op.target_branch.clone(),
                    conflicts_count: op.conflicts.len(),
                    initiated_at: op.initiated_at.to_rfc3339(),
                    completed_at: op.completed_at.map(|t| t.to_rfc3339()),
                })
                .collect();
            (StatusCode::OK, Json(responses)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Get a specific merge operation
#[utoipa::path(
    get,
    path = "/api/v1/merge-operations/{operation_id}",
    tag = "Merge Operations",
    params(
        ("operation_id" = Uuid, Path, description = "Merge operation ID")
    ),
    responses(
        (status = 200, description = "Merge operation details", body = MergeOperation),
        (status = 404, description = "Merge operation not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_merge_operation(
    State(store): State<AppState>,
    Path(operation_id): Path<Uuid>,
) -> impl IntoResponse {
    match store.get_merge_operation(operation_id).await {
        Ok(Some(operation)) => (StatusCode::OK, Json(operation)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Merge operation not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// List conflicts for a merge operation
#[utoipa::path(
    get,
    path = "/api/v1/merge-operations/{operation_id}/conflicts",
    tag = "Merge Operations",
    params(
        ("operation_id" = Uuid, Path, description = "Merge operation ID")
    ),
    responses(
        (status = 200, description = "List of merge conflicts", body = Vec<MergeConflict>),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_merge_conflicts(
    State(store): State<AppState>,
    Path(operation_id): Path<Uuid>,
) -> impl IntoResponse {
    match store.list_merge_conflicts(operation_id).await {
        Ok(conflicts) => (StatusCode::OK, Json(conflicts)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Resolve a specific conflict
#[utoipa::path(
    post,
    path = "/api/v1/conflicts/{conflict_id}/resolve",
    tag = "Merge Operations",
    params(
        ("conflict_id" = Uuid, Path, description = "Conflict ID")
    ),
    request_body = ResolveConflictRequest,
    responses(
        (status = 200, description = "Conflict resolved"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn resolve_conflict(
    State(store): State<AppState>,
    Extension(session): Extension<UserSession>,
    Path(conflict_id): Path<Uuid>,
    Json(payload): Json<ResolveConflictRequest>,
) -> impl IntoResponse {
    let resolution = ConflictResolution {
        conflict_id,
        strategy: payload.strategy,
        resolved_value: payload.resolved_value,
        resolved_by: session.user_id,
        resolved_at: chrono::Utc::now(),
    };

    match store.resolve_merge_conflict(conflict_id, resolution).await {
        Ok(_) => {
            // Check if all conflicts for the operation are resolved
            if let Ok(Some(conflict)) = store.get_merge_conflict(conflict_id).await {
                if let Ok(all_conflicts) = store.list_merge_conflicts(conflict.merge_operation_id).await {
                    let all_resolved = all_conflicts.iter().all(|c| c.is_resolved());
                    
                    if all_resolved {
                        // Update operation status to Ready
                        let _ = store.update_merge_operation_status(
                            conflict.merge_operation_id,
                            pangolin_core::model::MergeStatus::Ready
                        ).await;
                    }
                }
            }

            (StatusCode::OK, Json(serde_json::json!({
                "status": "resolved",
                "conflict_id": conflict_id
            }))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Complete a merge operation (after all conflicts resolved)
#[utoipa::path(
    post,
    path = "/api/v1/merge-operations/{operation_id}/complete",
    tag = "Merge Operations",
    params(
        ("operation_id" = Uuid, Path, description = "Merge operation ID")
    ),
    responses(
        (status = 200, description = "Merge completed"),
        (status = 400, description = "Cannot complete merge"),
        (status = 404, description = "Merge operation not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn complete_merge(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Path(operation_id): Path<Uuid>,
) -> impl IntoResponse {
    // Get the operation
    let operation = match store.get_merge_operation(operation_id).await {
        Ok(Some(op)) => op,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Merge operation not found"})),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    };

    // Check if operation can be completed
    if !operation.can_complete() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Cannot complete merge: conflicts not resolved or operation not ready"
            })),
        )
            .into_response();
    }

    // Perform the actual merge
    match store.merge_branch(
        operation.tenant_id,
        &operation.catalog_name,
        operation.source_branch.clone(),
        operation.target_branch.clone(),
    ).await {
        Ok(_) => {
            // TODO: merge_branch should return the commit ID
            // For now, generate a placeholder UUID
            let commit_id = Uuid::new_v4();
            let _ = store.complete_merge_operation(operation_id, commit_id).await;

            // Audit log
            let _ = store.log_audit_event(tenant.0, pangolin_core::audit::AuditLogEntry::new(
                tenant.0,
                session.username.clone(),
                "complete_merge".to_string(),
                format!("{}/{}->{}", operation.catalog_name, operation.source_branch, operation.target_branch),
                None
            )).await;

            (StatusCode::OK, Json(serde_json::json!({
                "status": "completed",
                "operation_id": operation_id,
                "commit_id": commit_id
            }))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Abort a merge operation
#[utoipa::path(
    post,
    path = "/api/v1/merge-operations/{operation_id}/abort",
    tag = "Merge Operations",
    params(
        ("operation_id" = Uuid, Path, description = "Merge operation ID")
    ),
    responses(
        (status = 200, description = "Merge aborted"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn abort_merge(
    State(store): State<AppState>,
    Path(operation_id): Path<Uuid>,
) -> impl IntoResponse {
    match store.abort_merge_operation(operation_id).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({
            "status": "aborted",
            "operation_id": operation_id
        }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
