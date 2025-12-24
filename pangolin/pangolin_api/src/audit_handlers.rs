use axum::{
    extract::{Path, Query, State, Extension},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use pangolin_core::audit::{AuditLogEntry, AuditLogFilter};
use pangolin_core::user::{UserSession, UserRole};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use pangolin_store::CatalogStore;
use crate::auth::TenantId;
use utoipa::ToSchema;

pub type AppState = Arc<dyn CatalogStore + Send + Sync>;

/// Query parameters for listing audit events
#[derive(Debug, Deserialize, ToSchema)]
pub struct AuditListQuery {
    /// Filter by user ID
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub user_id: Option<Uuid>,
    /// Filter by action (e.g., "create_table", "update_catalog")
    #[schema(example = "create_table")]
    pub action: Option<String>,
    /// Filter by resource type (e.g., "table", "catalog", "user")
    #[schema(example = "table")]
    pub resource_type: Option<String>,
    /// Filter by resource ID
    #[schema(example = "660e8400-e29b-41d4-a716-446655440000")]
    pub resource_id: Option<Uuid>,
    /// Filter by start time (ISO 8601 format)
    #[schema(example = "2025-12-01T00:00:00Z")]
    pub start_time: Option<String>,
    /// Filter by end time (ISO 8601 format)
    #[schema(example = "2025-12-31T23:59:59Z")]
    pub end_time: Option<String>,
    /// Filter by result ("success" or "failure")
    #[schema(example = "success")]
    pub result: Option<String>,
    /// Maximum number of results to return (default: 100, max: 1000)
    #[schema(example = 50)]
    pub limit: Option<usize>,
    /// Number of results to skip for pagination
    #[schema(example = 0)]
    pub offset: Option<usize>,
}

/// Response for audit event count
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuditCountResponse {
    /// Total number of audit events matching the filter
    #[schema(example = 42)]
    pub count: usize,
}

/// List audit events with optional filtering
///
/// Returns a list of audit events with support for powerful filtering and pagination.
/// All filters are optional and can be combined.
#[utoipa::path(
    get,
    path = "/api/v1/audit",
    tag = "Audit Logging",
    params(
        ("user_id" = Option<Uuid>, Query, description = "Filter by user UUID"),
        ("action" = Option<String>, Query, description = "Filter by action (e.g., 'create_table')"),
        ("resource_type" = Option<String>, Query, description = "Filter by resource type (e.g., 'table')"),
        ("resource_id" = Option<Uuid>, Query, description = "Filter by resource UUID"),
        ("start_time" = Option<String>, Query, description = "Filter by start time (ISO 8601)"),
        ("end_time" = Option<String>, Query, description = "Filter by end time (ISO 8601)"),
        ("result" = Option<String>, Query, description = "Filter by result ('success' or 'failure')"),
        ("limit" = Option<usize>, Query, description = "Maximum results (default: 100, max: 1000)"),
        ("offset" = Option<usize>, Query, description = "Skip N results for pagination"),
    ),
    responses(
        (status = 200, description = "List of audit events", body = Vec<AuditLogEntry>),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_audit_events(
    State(store): State<AppState>,
    Extension(tenant_id): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Query(query): Query<AuditListQuery>,
) -> impl IntoResponse {
    // Parse query parameters into AuditLogFilter
    let filter = AuditLogFilter {
        user_id: query.user_id,
        action: query.action.and_then(|s| {
            serde_json::from_str(&format!("\"{}\"", s.to_lowercase())).ok()
        }),
        resource_type: query.resource_type.and_then(|s| {
            serde_json::from_str(&format!("\"{}\"", s.to_lowercase())).ok()
        }),
        resource_id: query.resource_id,
        start_time: query.start_time.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        }),
        end_time: query.end_time.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        }),
        result: query.result.and_then(|s| {
            serde_json::from_str(&format!("\"{}\"", s.to_lowercase())).ok()
        }),
        limit: Some(query.limit.unwrap_or(100).min(1000)),
        offset: query.offset,
    };

    // Root users can see events from all tenants
    let events_result = if session.role == UserRole::Root {
        // Fetch events from all tenants and aggregate
        match store.list_tenants().await {
            Ok(tenants) => {
                let mut all_events = Vec::new();
                for tenant in tenants {
                    match store.list_audit_events(tenant.id, Some(filter.clone())).await {
                        Ok(mut events) => all_events.append(&mut events),
                        Err(e) => {
                            tracing::warn!("Failed to fetch audit events for tenant {}: {}", tenant.id, e);
                        }
                    }
                }
                // Sort by timestamp descending
                all_events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                // Apply limit and offset
                let offset = filter.offset.unwrap_or(0);
                let limit = filter.limit.unwrap_or(100);
                let events: Vec<_> = all_events.into_iter().skip(offset).take(limit).collect();
                Ok(events)
            },
            Err(e) => Err(e)
        }
    } else {
        // Non-root users only see events from their tenant
        store.list_audit_events(tenant_id.0, Some(filter)).await
    };

    match events_result {
        Ok(events) => (StatusCode::OK, Json(events)).into_response(),
        Err(e) => {
            tracing::error!("Failed to list audit events: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to retrieve audit events",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

/// Get a specific audit event by ID
///
/// Retrieves a single audit event by its unique identifier.
#[utoipa::path(
    get,
    path = "/api/v1/audit/{event_id}",
    tag = "Audit Logging",
    params(
        ("event_id" = Uuid, Path, description = "Audit event ID"),
    ),
    responses(
        (status = 200, description = "Audit event details", body = AuditLogEntry),
        (status = 404, description = "Audit event not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_audit_event(
    State(store): State<AppState>,
    Extension(tenant_id): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Path(event_id): Path<Uuid>,
) -> impl IntoResponse {
    // Root users can access events from any tenant
    let event_result = if session.role == UserRole::Root {
        // Try to find the event in any tenant
        match store.list_tenants().await {
            Ok(tenants) => {
                let mut found_event = None;
                for tenant in tenants {
                    if let Ok(Some(event)) = store.get_audit_event(tenant.id, event_id).await {
                        found_event = Some(event);
                        break;
                    }
                }
                Ok(found_event)
            },
            Err(e) => Err(e)
        }
    } else {
        // Non-root users only see events from their tenant
        store.get_audit_event(tenant_id.0, event_id).await
    };

    match event_result {
        Ok(Some(event)) => (StatusCode::OK, Json(event)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Audit event not found",
                "event_id": event_id
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to get audit event {}: {}", event_id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to retrieve audit event",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

/// Count audit events with optional filtering
///
/// Returns the total count of audit events matching the specified filters.
/// Accepts the same query parameters as list_audit_events.
#[utoipa::path(
    get,
    path = "/api/v1/audit/count",
    tag = "Audit Logging",
    params(
        ("user_id" = Option<Uuid>, Query, description = "Filter by user UUID"),
        ("action" = Option<String>, Query, description = "Filter by action (e.g., 'create_table')"),
        ("resource_type" = Option<String>, Query, description = "Filter by resource type (e.g., 'table')"),
        ("resource_id" = Option<Uuid>, Query, description = "Filter by resource UUID"),
        ("start_time" = Option<String>, Query, description = "Filter by start time (ISO 8601)"),
        ("end_time" = Option<String>, Query, description = "Filter by end time (ISO 8601)"),
        ("result" = Option<String>, Query, description = "Filter by result ('success' or 'failure')"),
    ),
    responses(
        (status = 200, description = "Count of audit events", body = AuditCountResponse),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn count_audit_events(
    State(store): State<AppState>,
    Extension(tenant_id): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Query(query): Query<AuditListQuery>,
) -> impl IntoResponse {
    // Parse query parameters into AuditLogFilter (same as list_audit_events)
    let filter = AuditLogFilter {
        user_id: query.user_id,
        action: query.action.and_then(|s| {
            serde_json::from_str(&format!("\"{}\"", s.to_lowercase())).ok()
        }),
        resource_type: query.resource_type.and_then(|s| {
            serde_json::from_str(&format!("\"{}\"", s.to_lowercase())).ok()
        }),
        resource_id: query.resource_id,
        start_time: query.start_time.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        }),
        end_time: query.end_time.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        }),
        result: query.result.and_then(|s| {
            serde_json::from_str(&format!("\"{}\"", s.to_lowercase())).ok()
        }),
        limit: None,  // Not needed for counting
        offset: None, // Not needed for counting
    };

    // Root users can count events from all tenants
    let count_result = if session.role == UserRole::Root {
        // Count events from all tenants and sum
        match store.list_tenants().await {
            Ok(tenants) => {
                let mut total_count = 0;
                for tenant in tenants {
                    match store.count_audit_events(tenant.id, Some(filter.clone())).await {
                        Ok(count) => total_count += count,
                        Err(e) => {
                            tracing::warn!("Failed to count audit events for tenant {}: {}", tenant.id, e);
                        }
                    }
                }
                Ok(total_count)
            },
            Err(e) => Err(e)
        }
    } else {
        // Non-root users only count events from their tenant
        store.count_audit_events(tenant_id.0, Some(filter)).await
    };

    match count_result {
        Ok(count) => (StatusCode::OK, Json(AuditCountResponse { count })).into_response(),
        Err(e) => {
            tracing::error!("Failed to count audit events: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to count audit events",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}
