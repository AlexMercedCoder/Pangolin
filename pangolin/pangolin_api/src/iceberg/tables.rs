use axum::{
    extract::{Path, State, Query, Extension},
    Json,
    response::IntoResponse,
    http::{StatusCode, HeaderMap, Method},
};
use bytes::Bytes;
use pangolin_core::model::{Asset, AssetType};
use pangolin_core::permission::{PermissionScope, Action};
use pangolin_core::user::UserSession;
use pangolin_core::iceberg_metadata::{TableMetadata, Schema, PartitionSpec, SortOrder, Snapshot, NestedField, Type};
use crate::auth::TenantId;
use crate::authz::check_permission;
use super::{check_and_forward_if_federated, AppState};
use super::types::*;
use std::collections::HashMap;
use chrono::Utc;
use uuid::Uuid;
use std::sync::Arc;

/// List tables in a namespace
#[utoipa::path(
    get,
    path = "/v1/{prefix}/namespaces/{namespace}/tables",
    tag = "Iceberg REST",
    params(
        ("prefix" = String, Path, description = "Catalog name"),
        ("namespace" = String, Path, description = "Namespace (optionally with @branch)")
    ),
    responses(
        (status = 200, description = "List of tables", body = ListTablesResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Catalog or namespace not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_tables(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Path((prefix, namespace)): Path<(String, String)>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix.clone();
    
    // Federated check
    let path = format!("/namespaces/{}/tables", namespace);
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
    
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "Catalog not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    };
    
    let (ns_name, branch) = parse_table_identifier(&namespace);
    
    // Check Permissions
    let scope = PermissionScope::Namespace { catalog_id: catalog.id, namespace: ns_name.clone() };
    match check_permission(&store, &session, &Action::List, &scope).await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, "Forbidden").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Permission check failed: {}", e)).into_response(),
    }
    
    let ns_vec = vec![ns_name];

    match store.list_assets(tenant_id, &catalog_name, branch, ns_vec.clone()).await {
        Ok(assets) => {
            let identifiers: Vec<TableIdentifier> = assets.into_iter()
                .filter(|a| a.kind == AssetType::IcebergTable)
                .map(|a| TableIdentifier {
                    namespace: ns_vec.clone(),
                    name: a.name,
                }).collect();
            (StatusCode::OK, Json(ListTablesResponse { identifiers })).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}


#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct MaintenanceRequest {
    pub job_type: String, // "expire_snapshots" or "remove_orphan_files"
    pub retention_ms: Option<i64>,
    pub older_than_ms: Option<i64>,
}

/// Perform maintenance on a table
#[utoipa::path(
    post,
    path = "/api/v1/catalogs/{prefix}/namespaces/{namespace}/tables/{table}/maintenance",
    tag = "Data Explorer",
    params(
        ("prefix" = String, Path, description = "Catalog name"),
        ("namespace" = String, Path, description = "Namespace"),
        ("table" = String, Path, description = "Table")
    ),
    request_body = MaintenanceRequest,
    responses(
        (status = 200, description = "Maintenance accepted", body = serde_json::Value),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn perform_maintenance(
    State(store): State<Arc<dyn pangolin_store::CatalogStore + Send + Sync>>,
    Extension(tenant_id): Extension<TenantId>,
    Path((_prefix, namespace, table)): Path<(String, String, String)>,
    Json(payload): Json<MaintenanceRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let namespace_parts: Vec<String> = namespace.split('\x1F').map(|s| s.to_string()).collect();
    // Parse table@branch
    let (table_name, branch_name) = if let Some((t, b)) = table.split_once('@') {
        (t.to_string(), Some(b.to_string()))
    } else {
        (table.to_string(), None)
    };

    match payload.job_type.as_str() {
        "expire_snapshots" => {
            let retention = payload.retention_ms.unwrap_or(86400000); // Default 1 day
            store.expire_snapshots(tenant_id.0, "default", branch_name, namespace_parts, table_name, retention).await
                .map_err(|e| {
                    tracing::error!("Maintenance failed: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
        },
        "remove_orphan_files" => {
             let older_than = payload.older_than_ms.unwrap_or(86400000); // Default 1 day
             store.remove_orphan_files(tenant_id.0, "default", branch_name, namespace_parts, table_name, older_than).await
                .map_err(|e| {
                    tracing::error!("Maintenance failed: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
        },
        _ => return Err(StatusCode::BAD_REQUEST),
    }

    Ok(Json(serde_json::json!({ "status": "accepted" })))
}

/// Create a table
#[utoipa::path(
    post,
    path = "/v1/{prefix}/namespaces/{namespace}/tables",
    tag = "Iceberg REST",
    params(
        ("prefix" = String, Path, description = "Catalog name"),
        ("namespace" = String, Path, description = "Namespace (optionally with @branch)")
    ),
    request_body = CreateTableRequest,
    responses(
        (status = 200, description = "Table created", body = TableResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Catalog or namespace not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_table(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Path((prefix, namespace)): Path<(String, String)>,
    Query(params): Query<HashMap<String, String>>,
    Json(payload): Json<CreateTableRequest>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix;
    
    let mut path = format!("/namespaces/{}/tables", namespace);
    if !params.is_empty() {
        let query_string: String = params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        path.push_str("?");
        path.push_str(&query_string);
    }

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
    
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "Catalog not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    };
    
    let (tbl_name, branch_from_name) = parse_table_identifier(&payload.name);
    let (ns_name, branch_from_ns) = parse_table_identifier(&namespace);
    let branch_from_query = params.get("branch").cloned();
    let branch = branch_from_name.or(branch_from_ns).or(branch_from_query);
    
    let scope = PermissionScope::Namespace { catalog_id: catalog.id, namespace: ns_name.clone() };
    match check_permission(&store, &session, &Action::Create, &scope).await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, "Forbidden").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Permission check failed: {}", e)).into_response(),
    }
    
    let ns_vec = vec![ns_name.clone()];

    let table_uuid = Uuid::new_v4();
    let location = if let Some(loc) = payload.location {
        loc
    } else if let Some(base_loc) = &catalog.storage_location {
         format!("{}/{}/{}", base_loc.trim_end_matches('/'), ns_vec.join("/"), tbl_name)
    } else {
        let (bucket_name, scheme) = if let Some(warehouse_name) = &catalog.warehouse_name {
            if let Ok(Some(wh)) = store.get_warehouse(tenant_id, warehouse_name.clone()).await {
                 let b = wh.storage_config.get("s3.bucket").or(wh.storage_config.get("bucket")).cloned().unwrap_or_else(|| "warehouse".to_string());
                 let s = if wh.storage_config.contains_key("azure.container") { "abfss" }
                        else if wh.storage_config.contains_key("gcp.bucket") { "gs" }
                        else { "s3" };
                 (b, s)
            } else {
                ("warehouse".to_string(), "s3")
            }
        } else {
            ("warehouse".to_string(), "s3")
        };
        format!("{}://{}/{}/{}/{}", scheme, bucket_name, catalog_name, ns_vec.join("/"), tbl_name)
    };

    let schema_fields = if let Some(schema_value) = &payload.schema {
        if let Some(fields) = schema_value.get("fields").and_then(|f| f.as_array()) {
            fields.iter().filter_map(|field| {
                let id = field.get("id")?.as_i64()? as i32;
                let name = field.get("name")?.as_str()?.to_string();
                let required = false;
                let field_type_str = field.get("type")?.as_str()?;
                
                let field_type = match field_type_str {
                    "int" | "integer" => Type::Primitive("long".to_string()),
                    "long" => Type::Primitive("long".to_string()),
                    "string" => Type::Primitive("string".to_string()),
                    "boolean" => Type::Primitive("boolean".to_string()),
                    "float" => Type::Primitive("float".to_string()),
                    "double" => Type::Primitive("double".to_string()),
                    "date" => Type::Primitive("date".to_string()),
                    "time" => Type::Primitive("time".to_string()),
                    "timestamp" => Type::Primitive("timestamp".to_string()),
                    "timestamptz" => Type::Primitive("timestamptz".to_string()),
                    "binary" => Type::Primitive("binary".to_string()),
                    "uuid" => Type::Primitive("uuid".to_string()),
                    _ => Type::Primitive(field_type_str.to_string()),
                };
                
                Some(NestedField {
                    id,
                    name,
                    required,
                    field_type,
                    doc: None,
                })
            }).collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    };
    
    let metadata = TableMetadata {
        format_version: 2,
        table_uuid,
        location: location.clone(),
        last_sequence_number: 0,
        last_updated_ms: Utc::now().timestamp_millis(),
        last_column_id: schema_fields.iter().map(|f| f.id).max().unwrap_or(0),
        schemas: vec![Schema {
            schema_id: 0,
            identifier_field_ids: Some(vec![]),
            fields: schema_fields,
        }],
        current_schema_id: 0,
        current_partition_spec_id: 0,
        partition_specs: vec![PartitionSpec { spec_id: 0, fields: vec![] }],
        default_sort_order_id: 0,
        sort_orders: vec![SortOrder { order_id: 0, fields: vec![] }],
        properties: payload.properties.clone(),
        current_snapshot_id: None,
        snapshots: Some(vec![]),
        snapshot_log: Some(vec![]),
        metadata_log: Some(vec![]),
    };

    let metadata_json = serde_json::to_string(&metadata).unwrap();
    let metadata_location = format!("{}/metadata/00000-{}.metadata.json", location, Uuid::new_v4());
    
    let mut properties = payload.properties.unwrap_or_default();
    properties.insert("metadata_location".to_string(), metadata_location.clone());

    let asset = Asset {
        id: table_uuid,
        name: tbl_name.clone(),
        kind: AssetType::IcebergTable,
        location: location.clone(),
        properties: {
            let mut p = properties.clone();
            p.insert("metadata_location".to_string(), metadata_location.clone());
            p
        },
    };

    match store.create_asset(tenant_id, &catalog_name, branch, ns_vec, asset.clone()).await {
        Ok(_) => {
            if let Err(e) = store.write_file(&metadata_location, metadata_json.into_bytes()).await {
                 tracing::error!("Failed to write metadata file: {}", e);
                 return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to write metadata").into_response();
            }

            let _ = store.log_audit_event(tenant_id, pangolin_core::audit::AuditLogEntry::legacy_new(
                tenant_id,
                session.username.clone(),
                "create_table".to_string(),
                format!("{}/{}/{}", catalog_name, ns_name, tbl_name),
                Some(location.clone())
            )).await;

            let credentials = match store.get_catalog(tenant_id, catalog_name.clone()).await {
                Ok(Some(c)) => {
                    if let Some(warehouse_name) = c.warehouse_name {
                        match store.get_warehouse(tenant_id, warehouse_name).await {
                            Ok(Some(warehouse)) => {
                                let mut creds = HashMap::new();
                                if let Some(ak) = warehouse.storage_config.get("s3.access-key-id") { creds.insert("s3.access-key-id".to_string(), ak.clone()); }
                                if let Some(sk) = warehouse.storage_config.get("s3.secret-access-key") { creds.insert("s3.secret-access-key".to_string(), sk.clone()); }
                                if let Some(token) = warehouse.storage_config.get("s3.session-token") { creds.insert("s3.session-token".to_string(), token.clone()); }
                                if !creds.is_empty() { Some(creds) } else { None }
                            },
                            _ => None
                        }
                    } else { None }
                },
                _ => None
            };

            (StatusCode::OK, Json(TableResponse::with_credentials(
                Some(location.clone()),
                metadata,
                credentials,
                Some(table_uuid),
            ))).into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

/// Load a table
#[utoipa::path(
    get,
    path = "/v1/{prefix}/namespaces/{namespace}/tables/{table}",
    tag = "Iceberg REST",
    params(
        ("prefix" = String, Path, description = "Catalog name"),
        ("namespace" = String, Path, description = "Namespace (optionally with @branch)"),
        ("table" = String, Path, description = "Table name (optionally with @branch)")
    ),
    responses(
        (status = 200, description = "Table metadata", body = TableResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Table not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn load_table(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Path((prefix, namespace, table)): Path<(String, String, String)>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix;
    
    let mut path = format!("/namespaces/{}/tables/{}", namespace, table);
    if !params.is_empty() {
        let query_string: String = params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        path.push_str("?");
        path.push_str(&query_string);
    }
    
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
    
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "Catalog not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    };
    
    let (tbl_name, branch_from_name) = parse_table_identifier(&table);
    let (ns_name, branch_from_ns) = parse_table_identifier(&namespace);
    let branch = branch_from_name.or(branch_from_ns);
    
    let ns_vec = vec![ns_name];

    let asset = match store.get_asset(tenant_id, &catalog_name, branch.clone(), ns_vec.clone(), tbl_name.clone()).await {
        Ok(Some(a)) => a,
        Ok(None) => return (StatusCode::NOT_FOUND, "Table not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    };
    
    let scope = PermissionScope::Asset { 
        catalog_id: catalog.id, 
        namespace: ns_vec.join("."), 
        asset_id: asset.id 
    };
    
    match check_permission(&store, &session, &Action::Read, &scope).await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, "Forbidden").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Permission check failed: {}", e)).into_response(),
    }

    let current_metadata_location = asset.properties.get("metadata_location").cloned();

    if let Some(location) = current_metadata_location {
        let metadata_bytes = match store.read_file(&location).await {
            Ok(bytes) => bytes,
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read metadata file").into_response(),
        };
        
        let metadata: TableMetadata = match serde_json::from_slice(&metadata_bytes) {
            Ok(m) => m,
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse metadata").into_response(),
        };

        // Credential vending logic...
        let credentials = match store.get_catalog(tenant_id, catalog_name.clone()).await {
            Ok(Some(catalog)) => {
                if let Some(warehouse_name) = catalog.warehouse_name {
                    match store.get_warehouse(tenant_id, warehouse_name).await {
                        Ok(Some(warehouse)) => {
                            let mut creds = HashMap::new();
                            if let Some(ak) = warehouse.storage_config.get("s3.access-key-id") { creds.insert("s3.access-key-id".to_string(), ak.clone()); }
                            if let Some(sk) = warehouse.storage_config.get("s3.secret-access-key") { creds.insert("s3.secret-access-key".to_string(), sk.clone()); }
                            if let Some(token) = warehouse.storage_config.get("s3.session-token") { creds.insert("s3.session-token".to_string(), token.clone()); }
                            if !creds.is_empty() { Some(creds) } else { None }
                        },
                        _ => None
                    }
                } else { None }
            },
            _ => None
        };

        (StatusCode::OK, Json(TableResponse::with_credentials(
            Some(location),
            metadata,
            credentials,
            Some(asset.id),
        ))).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Metadata location not found").into_response()
    }
}

/// Update a table (Commit)
#[utoipa::path(
    post,
    path = "/v1/{prefix}/namespaces/{namespace}/tables/{table}",
    tag = "Iceberg REST",
    params(
        ("prefix" = String, Path, description = "Catalog name"),
        ("namespace" = String, Path, description = "Namespace (optionally with @branch)"),
        ("table" = String, Path, description = "Table name (optionally with @branch)")
    ),
    request_body = CommitTableRequest,
    responses(
        (status = 200, description = "Table updated", body = TableResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Table not found"),
        (status = 409, description = "Conflict (OCC failed)"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_table(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Path((prefix, namespace, table)): Path<(String, String, String)>,
    Json(payload): Json<CommitTableRequest>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix;
    
    let path = format!("/namespaces/{}/tables/{}", namespace, table);
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
    
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "Catalog not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    };

    let (table_name, branch_from_name) = parse_table_identifier(&table);
    let branch = branch_from_name.unwrap_or("main".to_string());
    let namespace_parts: Vec<String> = namespace.split('\x1F').map(|s| s.to_string()).collect();
    
    let asset = match store.get_asset(tenant_id, &catalog_name, Some(branch.clone()), namespace_parts.clone(), table_name.clone()).await {
        Ok(Some(a)) => a,
        Ok(None) => return (StatusCode::NOT_FOUND, "Table not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    };
    
    let scope = PermissionScope::Asset { 
        catalog_id: catalog.id, 
        namespace: namespace_parts.join("."), 
        asset_id: asset.id 
    };
    
    match check_permission(&store, &session, &Action::Write, &scope).await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, "Forbidden").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Permission check failed: {}", e)).into_response(),
    }
    
    let mut retries = 0;
    const MAX_RETRIES: i32 = 5;

    while retries < MAX_RETRIES {
        let current_asset = match store.get_asset(tenant_id, &catalog_name, Some(branch.clone()), namespace_parts.clone(), table_name.clone()).await {
            Ok(Some(a)) => a,
            Ok(None) => return (StatusCode::NOT_FOUND, "Table not found").into_response(),
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
        };
        
        let current_metadata_location = current_asset.properties.get("metadata_location").cloned();
        
        let metadata_bytes = if let Some(loc) = &current_metadata_location {
             match store.read_file(loc).await {
                 Ok(bytes) => bytes,
                 Err(_) => return (StatusCode::NOT_FOUND, "Failed to read metadata file").into_response()
             }
        } else {
             return (StatusCode::INTERNAL_SERVER_ERROR, "Table corrupted (no metadata)").into_response()
        };
        
        let mut metadata: TableMetadata = match serde_json::from_slice(&metadata_bytes) {
            Ok(m) => m,
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse metadata").into_response()
        };

        // Requirements check
        for requirement in &payload.requirements {
            match requirement {
                CommitRequirement::AssertCurrentSchemaId { current_schema_id } => {
                    if let Some(req_id) = current_schema_id {
                        if metadata.current_schema_id != *req_id {
                            return (StatusCode::CONFLICT, "Current schema ID mismatch").into_response();
                        }
                    }
                },
                CommitRequirement::AssertTableUuid { uuid } => {
                     if metadata.table_uuid.to_string() != *uuid {
                         return (StatusCode::CONFLICT, "Table UUID mismatch").into_response();
                     }
                },
                 _ => {}
            }
        }

        // Apply updates
        for update in &payload.updates {
            match update {
                CommitUpdate::AddSnapshot { snapshot } => {
                    match serde_json::from_value::<Snapshot>(snapshot.clone()) {
                        Ok(snapshot_obj) => {
                            if let Some(ref mut snapshots) = metadata.snapshots {
                                snapshots.push(snapshot_obj.clone());
                            } else {
                                metadata.snapshots = Some(vec![snapshot_obj.clone()]);
                            }
                            metadata.current_snapshot_id = Some(snapshot_obj.snapshot_id);
                            metadata.last_updated_ms = Utc::now().timestamp_millis();
                            metadata.last_sequence_number = snapshot_obj.snapshot_id;
                        },
                        Err(_) => {
                            if let Some(snapshot_id) = snapshot.get("snapshot-id").and_then(|v| v.as_i64()) {
                                metadata.current_snapshot_id = Some(snapshot_id);
                                metadata.last_updated_ms = Utc::now().timestamp_millis();
                                metadata.last_sequence_number = snapshot_id;
                            }
                        }
                    }
                },
                CommitUpdate::AddSchema { schema } => {
                    if let Ok(new_schema) = serde_json::from_value::<pangolin_core::iceberg_metadata::Schema>(schema.clone()) {
                         metadata.schemas.push(new_schema);
                    }
                },
                CommitUpdate::SetCurrentSchema { schema_id } => {
                    if *schema_id == -1 {
                        if let Some(last) = metadata.schemas.last() {
                            metadata.current_schema_id = last.schema_id;
                        } else {
                             metadata.current_schema_id = *schema_id;
                        }
                    } else {
                        metadata.current_schema_id = *schema_id;
                    }
                },
                _ => {}
            }
        }

        let new_metadata_location = format!("{}/metadata/00000-{}.metadata.json", metadata.location, Uuid::new_v4());
        let metadata_json = serde_json::to_string(&metadata).unwrap();

        if let Err(_) = store.write_file(&new_metadata_location, metadata_json.into_bytes()).await {
             return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to write new metadata").into_response();
        }

        match store.update_metadata_location(tenant_id, &catalog_name, Some(branch.clone()), namespace_parts.clone(), table_name.clone(), current_metadata_location.clone(), new_metadata_location.clone()).await {
            Ok(_) => {
                let _ = store.log_audit_event(tenant_id, pangolin_core::audit::AuditLogEntry::legacy_new(
                    tenant_id,
                    session.username.clone(),
                    "update_table".to_string(),
                    format!("{}/{}/{}", catalog_name, namespace, table_name),
                    Some(new_metadata_location.clone())
                )).await;

                return (StatusCode::OK, Json(TableResponse::new(
                    Some(new_metadata_location.clone()),
                    metadata,
                    Some(asset.id),
                ))).into_response();
            },
            Err(_) => {
                retries += 1;
                continue;
            }
        }
    }

    (StatusCode::CONFLICT, "Failed to commit after retries").into_response()
}

/// Rename a table
#[utoipa::path(
    post,
    path = "/v1/{prefix}/tables/rename",
    tag = "Iceberg REST",
    params(
        ("prefix" = String, Path, description = "Catalog name")
    ),
    request_body = RenameTableRequest,
    responses(
        (status = 204, description = "Table renamed"),
        (status = 404, description = "Source table not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn rename_table(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Path(prefix): Path<String>,
    Json(payload): Json<RenameTableRequest>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix;
    
    let path = "/tables/rename".to_string();
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
    
    let source_ns = payload.source.namespace;
    let source_name = payload.source.name;
    let dest_ns = payload.destination.namespace;
    let dest_name = payload.destination.name;
    let branch = Some("main".to_string());

    match store.rename_asset(tenant_id, &catalog_name, branch, source_ns.clone(), source_name.clone(), dest_ns.clone(), dest_name.clone()).await {
        Ok(_) => {
            let _ = store.log_audit_event(tenant_id, pangolin_core::audit::AuditLogEntry::legacy_new(
                tenant_id,
                session.username.clone(),
                "rename_table".to_string(),
                format!("{}/{}.{} -> {}.{}", catalog_name, source_ns.join("."), source_name, dest_ns.join("."), dest_name),
                None
            )).await;
            
            StatusCode::NO_CONTENT.into_response()
        },
        Err(_) => (StatusCode::NOT_FOUND, "Table not found").into_response(),
    }
}

/// Delete a table
#[utoipa::path(
    delete,
    path = "/v1/{prefix}/namespaces/{namespace}/tables/{table}",
    tag = "Iceberg REST",
    params(
        ("prefix" = String, Path, description = "Catalog name"),
        ("namespace" = String, Path, description = "Namespace (optionally with @branch)"),
        ("table" = String, Path, description = "Table name (optionally with @branch)")
    ),
    responses(
        (status = 204, description = "Table deleted"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Table not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_table(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Path((prefix, namespace, table)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix;
    
    let path = format!("/namespaces/{}/tables/{}", namespace, table);
    if let Some(response) = check_and_forward_if_federated(
        &store,
        tenant_id,
        &catalog_name,
        Method::DELETE,
        &path,
        None,
        HeaderMap::new(),
    ).await {
         return response;
    }
    
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "Catalog not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response(),
    };
    
    let (table_name, branch_from_name) = parse_table_identifier(&table);
    let branch = branch_from_name.or(Some("main".to_string()));
    let namespace_parts: Vec<String> = namespace.split('\x1F').map(|s| s.to_string()).collect();
    
    let asset = match store.get_asset(tenant_id, &catalog_name, branch.clone(), namespace_parts.clone(), table_name.clone()).await {
        Ok(Some(a)) => a,
        Ok(None) => return (StatusCode::NOT_FOUND, "Table not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    };
    
    let scope = PermissionScope::Asset { 
        catalog_id: catalog.id, 
        namespace: namespace_parts.join("."), 
        asset_id: asset.id 
    };
    
    match check_permission(&store, &session, &Action::Delete, &scope).await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, "Forbidden").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Permission check failed: {}", e)).into_response(),
    }

    match store.delete_asset(tenant_id, &catalog_name, branch, namespace_parts, table_name).await {
        Ok(_) => {
             let _ = store.log_audit_event(tenant_id, pangolin_core::audit::AuditLogEntry::legacy_new(
                tenant_id,
                session.username.clone(),
                "delete_table".to_string(),
                format!("{}/{}/{}", catalog_name, namespace, table),
                None
            )).await;
            
            StatusCode::NO_CONTENT.into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

/// Check if a table exists
#[utoipa::path(
    head,
    path = "/v1/{prefix}/namespaces/{namespace}/tables/{table}",
    tag = "Iceberg REST",
    params(
        ("prefix" = String, Path, description = "Catalog name"),
        ("namespace" = String, Path, description = "Namespace"),
        ("table" = String, Path, description = "Table")
    ),
    responses(
        (status = 200, description = "Table exists"),
        (status = 404, description = "Table not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn table_exists(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path((prefix, namespace, table)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix;
    
    let path = format!("/namespaces/{}/tables/{}", namespace, table);
    if let Some(response) = check_and_forward_if_federated(
        &store,
        tenant_id,
        &catalog_name,
        Method::HEAD,
        &path,
        None,
        HeaderMap::new(),
    ).await {
         return response;
    }
    
    let namespace_parts: Vec<String> = namespace.split('\x1F').map(|s| s.to_string()).collect();
    let (table_name, branch_name) = if let Some((t, b)) = table.split_once('@') {
        (t.to_string(), Some(b.to_string()))
    } else {
        (table.to_string(), None)
    };

    match store.get_asset(tenant_id, &catalog_name, branch_name, namespace_parts, table_name).await {
        Ok(Some(_)) => StatusCode::OK.into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Report metrics for a table
#[utoipa::path(
    post,
    path = "/v1/{prefix}/namespaces/{namespace}/tables/{table}/metrics",
    tag = "Iceberg REST",
    params(
        ("prefix" = String, Path, description = "Catalog name"),
        ("namespace" = String, Path, description = "Namespace"),
        ("table" = String, Path, description = "Table")
    ),
    responses(
        (status = 204, description = "Metrics reported"),
    )
)]
pub async fn report_metrics(
    Path((_prefix, _namespace, _table)): Path<(String, String, String)>,
) -> impl IntoResponse {
    tracing::info!("Received metrics report");
    StatusCode::NO_CONTENT
}
