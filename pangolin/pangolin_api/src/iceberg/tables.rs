use super::types::*;
use super::{
    bad_request, check_and_forward_if_federated, commit, forbidden, iceberg_error, internal,
    no_such_namespace, no_such_table, table_already_exists, AppState,
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
use chrono::Utc;
use pangolin_core::iceberg_metadata::{
    MetadataLogEntry, PartitionSpec, Schema, SortOrder, TableMetadata,
};
use pangolin_core::model::{Asset, AssetType};
use pangolin_core::permission::{Action, PermissionScope};
use pangolin_core::user::UserSession;
use pangolin_store::PaginationParams;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// How many previous metadata files to keep in `metadata-log` when the table
/// does not set `write.metadata.previous-versions-max`. Matches the Iceberg
/// default.
const DEFAULT_PREVIOUS_VERSIONS_MAX: usize = 100;

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
    Query(page): Query<IcebergPageParams>,
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
    )
    .await
    {
        return response;
    }

    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return no_such_namespace(&catalog_name),
        Err(e) => {
            tracing::error!(error = %e, "list_tables: failed to load catalog");
            return internal("Failed to load catalog");
        }
    };

    let (ns_vec, branch) = parse_namespace(&namespace);

    // Check Permissions
    let scope = PermissionScope::Namespace {
        catalog_id: catalog.id,
        namespace: ns_vec.join("."),
    };
    match check_permission(&store, &session, &Action::List, &scope).await {
        Ok(true) => (),
        Ok(false) => return forbidden("Forbidden"),
        Err(e) => {
            tracing::error!(error = %e, "list_tables: permission check failed");
            return internal("Permission check failed");
        }
    }

    let (offset, limit) = page.resolve();
    let pagination = PaginationParams {
        limit: Some(limit as usize),
        offset: Some(offset as usize),
    };

    match store
        .list_assets(
            tenant_id,
            &catalog_name,
            branch,
            ns_vec.clone(),
            Some(pagination),
        )
        .await
    {
        Ok(assets) => {
            // The page token has to be computed from the number of rows the
            // store returned, before the asset-type filter narrows it - the
            // store is what applied `limit`, so a full page of rows means there
            // may be more even if none of them survive the filter.
            let returned = assets.len();
            let identifiers: Vec<TableIdentifier> = assets
                .into_iter()
                .filter(|a| a.kind == AssetType::IcebergTable)
                .map(|a| TableIdentifier {
                    namespace: ns_vec.clone(),
                    name: a.name,
                })
                .collect();
            (
                StatusCode::OK,
                Json(ListTablesResponse {
                    identifiers,
                    next_page_token: next_page_token(returned, offset, limit),
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "list_tables: failed to list assets");
            internal("Failed to list tables")
        }
    }
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct MaintenanceRequest {
    pub job_type: String, // "expire_snapshots" or "remove_orphan_files"
    pub retention_ms: Option<i64>,
    pub older_than_ms: Option<i64>,
}

/// Perform maintenance on a table
///
/// Two bugs fixed here (B0f):
///   1. the catalog from the path was discarded and the literal `"default"` was
///      passed to `expire_snapshots`/`remove_orphan_files`, so destructive
///      maintenance ran against the wrong catalog entirely;
///   2. there was no session and no permission check, so any tenant member could
///      trigger snapshot expiry and orphan-file deletion on any table.
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
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Catalog or table not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn perform_maintenance(
    State(store): State<Arc<dyn pangolin_store::CatalogStore + Send + Sync>>,
    Extension(tenant_id): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Path((prefix, namespace, table)): Path<(String, String, String)>,
    Json(payload): Json<MaintenanceRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let tenant = tenant_id.0;
    let catalog_name = prefix;

    let (namespace_parts, branch_from_ns) = parse_namespace(&namespace);
    let (table_name, branch_from_table) = parse_table_identifier(&table);
    let branch_name = branch_from_table.or(branch_from_ns);

    let catalog = match store.get_catalog(tenant, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!(error = %e, "maintenance: failed to load catalog");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let asset = match store
        .get_asset(
            tenant,
            &catalog_name,
            branch_name.clone(),
            namespace_parts.clone(),
            table_name.clone(),
        )
        .await
    {
        Ok(Some(a)) => a,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!(error = %e, "maintenance: failed to load asset");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Snapshot expiry and orphan-file removal both destroy data, so they need
    // Delete, not merely Write.
    let scope = PermissionScope::Asset {
        catalog_id: catalog.id,
        namespace: namespace_parts.join("."),
        asset_id: asset.id,
    };
    match check_permission(&store, &session, &Action::Delete, &scope).await {
        Ok(true) => (),
        Ok(false) => return Err(StatusCode::FORBIDDEN),
        Err(e) => {
            tracing::error!(error = %e, "maintenance: permission check failed");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    match payload.job_type.as_str() {
        "expire_snapshots" => {
            let retention = payload.retention_ms.unwrap_or(86400000); // Default 1 day
            store
                .expire_snapshots(
                    tenant,
                    &catalog_name,
                    branch_name,
                    namespace_parts,
                    table_name,
                    retention,
                )
                .await
                .map_err(|e| {
                    tracing::error!("Maintenance failed: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
        }
        "remove_orphan_files" => {
            let older_than = payload.older_than_ms.unwrap_or(86400000); // Default 1 day
            store
                .remove_orphan_files(
                    tenant,
                    &catalog_name,
                    branch_name,
                    namespace_parts,
                    table_name,
                    older_than,
                )
                .await
                .map_err(|e| {
                    tracing::error!("Maintenance failed: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
        }
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
        let query_string: String = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        path.push('?');
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
    )
    .await
    {
        return response;
    }

    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return no_such_namespace(&catalog_name),
        Err(e) => {
            tracing::error!(error = %e, "create_table: failed to load catalog");
            return internal("Failed to load catalog");
        }
    };

    let (tbl_name, branch_from_name) = parse_table_identifier(&payload.name);
    let (ns_vec, branch_from_ns) = parse_namespace(&namespace);
    let branch_from_query = params.get("branch").cloned();
    let branch = branch_from_name.or(branch_from_ns).or(branch_from_query);
    let ns_name = ns_vec.join(".");

    let scope = PermissionScope::Namespace {
        catalog_id: catalog.id,
        namespace: ns_name.clone(),
    };
    match check_permission(&store, &session, &Action::Create, &scope).await {
        Ok(true) => (),
        Ok(false) => return forbidden("Forbidden"),
        Err(e) => {
            tracing::error!(error = %e, "create_table: permission check failed");
            return internal("Permission check failed");
        }
    }

    let table_uuid = Uuid::new_v4();
    let location = if let Some(loc) = payload.location {
        loc
    } else if let Some(base_loc) = &catalog.storage_location {
        format!(
            "{}/{}/{}",
            base_loc.trim_end_matches('/'),
            ns_vec.join("/"),
            tbl_name
        )
    } else {
        let (bucket_name, scheme) = if let Some(warehouse_name) = &catalog.warehouse_name {
            if let Ok(Some(wh)) = store.get_warehouse(tenant_id, warehouse_name.clone()).await {
                let b = wh
                    .storage_config
                    .get("s3.bucket")
                    .or(wh.storage_config.get("bucket"))
                    .cloned()
                    .unwrap_or_else(|| "warehouse".to_string());
                let s = if wh.storage_config.contains_key("azure.container") {
                    "abfss"
                } else if wh.storage_config.contains_key("gcp.bucket") {
                    "gs"
                } else {
                    "s3"
                };
                (b, s)
            } else {
                ("warehouse".to_string(), "s3")
            }
        } else {
            ("warehouse".to_string(), "s3")
        };
        format!(
            "{}://{}/{}/{}/{}",
            scheme,
            bucket_name,
            catalog_name,
            ns_vec.join("/"),
            tbl_name
        )
    };

    // B16f: the schema used to be hand-parsed field by field, which silently
    // lost data three ways - `required` was hardcoded to `false` so every column
    // became optional, `int` was widened to `long`, and `field.get("type")?
    // .as_str()?` inside a `filter_map` returned `None` for any struct/list/map/
    // decimal/fixed column, so those columns were *dropped* and `last_column_id`
    // was computed from the survivors. The table was created `200 OK` with a
    // schema missing columns.
    //
    // Deserializing straight into the core `Schema` (as the commit path already
    // does) keeps nullability and complex types, and a malformed schema is now a
    // 400 rather than a quietly mangled table.
    let schema = match &payload.schema {
        Some(schema_value) => match serde_json::from_value::<Schema>(schema_value.clone()) {
            Ok(mut s) => {
                s.schema_id = 0;
                if s.identifier_field_ids.is_none() {
                    s.identifier_field_ids = Some(vec![]);
                }
                s
            }
            Err(e) => {
                return bad_request(&format!("Invalid schema: {}", e));
            }
        },
        None => Schema {
            type_: Schema::STRUCT.to_string(),
            schema_id: 0,
            identifier_field_ids: Some(vec![]),
            fields: vec![],
        },
    };

    let last_column_id = schema.max_field_id();

    let metadata = TableMetadata {
        format_version: 2,
        table_uuid,
        location: location.clone(),
        last_sequence_number: 0,
        last_updated_ms: Utc::now().timestamp_millis(),
        last_column_id,
        schemas: vec![schema],
        current_schema_id: 0,
        current_partition_spec_id: 0,
        partition_specs: vec![PartitionSpec {
            spec_id: 0,
            fields: vec![],
        }],
        // Required by the v2 spec; an empty unpartitioned spec assigns nothing,
        // so the highest assigned partition id is the "unpartitioned" sentinel.
        last_partition_id: pangolin_core::iceberg_metadata::PARTITION_FIELD_ID_START - 1,
        default_sort_order_id: 0,
        sort_orders: vec![SortOrder {
            order_id: 0,
            fields: vec![],
        }],
        properties: payload.properties.clone(),
        current_snapshot_id: None,
        snapshots: Some(vec![]),
        snapshot_log: Some(vec![]),
        metadata_log: Some(vec![]),
        refs: None,
    };

    let metadata_json = match serde_json::to_string(&metadata) {
        Ok(json) => json,
        Err(e) => {
            tracing::error!(error = %e, "create_table: failed to serialize metadata");
            return internal("Failed to serialize table metadata");
        }
    };
    let metadata_location = format!(
        "{}/metadata/00000-{}.metadata.json",
        location,
        Uuid::new_v4()
    );

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

    // B16g: write the metadata file *first*, then register the asset. The old
    // order registered the asset and only then wrote the file, so a failed write
    // left a permanently broken table - registered, pointing at a file that does
    // not exist, with `load_table` 500ing and `update_table` 404ing and no repair
    // path. This is also the order the commit path already uses.
    if let Err(e) = store
        .write_file(&metadata_location, metadata_json.into_bytes())
        .await
    {
        tracing::error!("Failed to write metadata file: {}", e);
        return internal("Failed to write metadata");
    }

    match store
        .create_asset(
            tenant_id,
            &catalog_name,
            branch,
            ns_vec.clone(),
            asset.clone(),
        )
        .await
    {
        Ok(_) => {
            let _ = store
                .log_audit_event(
                    tenant_id,
                    pangolin_core::audit::AuditLogEntry::success(
                        tenant_id,
                        Some(session.user_id),
                        session.username.clone(),
                        pangolin_core::audit::AuditAction::CreateTable,
                        pangolin_core::audit::ResourceType::Table,
                        Some(asset.id),
                        format!("{}/{}/{}", catalog_name, ns_name, tbl_name),
                    )
                    .with_metadata(serde_json::json!({ "location": location.clone() })),
                )
                .await;

            let credentials = match store.get_catalog(tenant_id, catalog_name.clone()).await {
                Ok(Some(c)) => {
                    if let Some(warehouse_name) = c.warehouse_name {
                        match store.get_warehouse(tenant_id, warehouse_name).await {
                            Ok(Some(warehouse)) => {
                                let mut creds = HashMap::new();
                                if let Some(ak) = warehouse.storage_config.get("s3.access-key-id") {
                                    creds.insert("s3.access-key-id".to_string(), ak.clone());
                                }
                                if let Some(sk) =
                                    warehouse.storage_config.get("s3.secret-access-key")
                                {
                                    creds.insert("s3.secret-access-key".to_string(), sk.clone());
                                }
                                if let Some(token) =
                                    warehouse.storage_config.get("s3.session-token")
                                {
                                    creds.insert("s3.session-token".to_string(), token.clone());
                                }
                                if !creds.is_empty() {
                                    Some(creds)
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            };

            (
                StatusCode::OK,
                Json(TableResponse::with_credentials(
                    // B16e: this returned `location` - the table *directory* -
                    // where the spec (and `load_table`, correctly) return the
                    // metadata *file*. A client that keeps the returned `Table`
                    // (PyIceberg does) ended up with a `metadata_location` it
                    // could neither read nor refresh from.
                    Some(metadata_location.clone()),
                    metadata,
                    credentials,
                    Some(table_uuid),
                )),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "create_table: failed to register asset");
            // The metadata file was written before registration (B16g); with the
            // asset unregistered it is unreferenced, so clean it up rather than
            // leaving an orphan behind.
            if let Err(cleanup) = store.delete_file(&metadata_location).await {
                tracing::warn!(
                    error = %cleanup,
                    location = %metadata_location,
                    "could not remove the orphaned metadata file after a failed create_asset"
                );
            }
            internal("Failed to create table")
        }
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
        let query_string: String = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        path.push('?');
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
    )
    .await
    {
        return response;
    }

    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return no_such_namespace(&catalog_name),
        Err(e) => {
            tracing::error!(error = %e, "load_table: failed to load catalog");
            return internal("Failed to load catalog");
        }
    };

    // B16a: shared namespace parsing, so a nested namespace resolves the same
    // way here as it does on the commit path.
    let (tbl_name, branch_from_name) = parse_table_identifier(&table);
    let (ns_vec, branch_from_ns) = parse_namespace(&namespace);
    let branch = branch_from_name.or(branch_from_ns);

    let asset = match store
        .get_asset(
            tenant_id,
            &catalog_name,
            branch.clone(),
            ns_vec.clone(),
            tbl_name.clone(),
        )
        .await
    {
        Ok(Some(a)) => a,
        Ok(None) => return no_such_table(&format!("{}.{}", ns_vec.join("."), tbl_name)),
        Err(e) => {
            tracing::error!(error = %e, "load_table: failed to load asset");
            return internal("Failed to load table");
        }
    };

    let scope = PermissionScope::Asset {
        catalog_id: catalog.id,
        namespace: ns_vec.join("."),
        asset_id: asset.id,
    };

    match check_permission(&store, &session, &Action::Read, &scope).await {
        Ok(true) => (),
        Ok(false) => return forbidden("Forbidden"),
        Err(e) => {
            tracing::error!(error = %e, "load_table: permission check failed");
            return internal("Permission check failed");
        }
    }

    let current_metadata_location = asset.properties.get("metadata_location").cloned();

    if let Some(location) = current_metadata_location {
        let metadata_bytes = match store.read_file(&location).await {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::error!(error = %e, "load_table: failed to read metadata file");
                return internal("Failed to read metadata file");
            }
        };

        // Parse metadata in blocking task
        let metadata_vec = metadata_bytes.to_vec();
        let metadata: TableMetadata = match tokio::task::spawn_blocking(move || {
            serde_json::from_slice(&metadata_vec)
        })
        .await
        {
            Ok(Ok(m)) => m,
            Ok(Err(e)) => {
                tracing::error!(error = %e, "update_table: failed to parse metadata");
                return internal("Failed to parse metadata");
            }
            Err(e) => {
                tracing::error!(error = %e, "update_table: metadata parse task panicked");
                return internal("Failed to parse metadata");
            }
        };

        // Credential vending logic...
        let credentials = match store.get_catalog(tenant_id, catalog_name.clone()).await {
            Ok(Some(catalog)) => {
                if let Some(warehouse_name) = catalog.warehouse_name {
                    match store.get_warehouse(tenant_id, warehouse_name).await {
                        Ok(Some(warehouse)) => {
                            let mut creds = HashMap::new();
                            if let Some(ak) = warehouse.storage_config.get("s3.access-key-id") {
                                creds.insert("s3.access-key-id".to_string(), ak.clone());
                            }
                            if let Some(sk) = warehouse.storage_config.get("s3.secret-access-key") {
                                creds.insert("s3.secret-access-key".to_string(), sk.clone());
                            }
                            if let Some(token) = warehouse.storage_config.get("s3.session-token") {
                                creds.insert("s3.session-token".to_string(), token.clone());
                            }
                            if !creds.is_empty() {
                                Some(creds)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        };

        (
            StatusCode::OK,
            Json(TableResponse::with_credentials(
                Some(location),
                metadata,
                credentials,
                Some(asset.id),
            )),
        )
            .into_response()
    } else {
        no_such_table(&format!("{}.{}", ns_vec.join("."), tbl_name))
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
    )
    .await
    {
        return response;
    }

    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return no_such_namespace(&catalog_name),
        Err(e) => {
            tracing::error!(error = %e, "update_table: failed to load catalog");
            return internal("Failed to load catalog");
        }
    };

    // B16a: this used to split the namespace on 0x1F while `create_table` and
    // `load_table` went through `parse_table_identifier` (a *single*-element
    // namespace). A table created in namespace `a\x1Fb` was registered under
    // `["a\x1Fb"]` and looked up here under `["a", "b"]`, so every commit to a
    // nested namespace 404'd and the CAS loop below never ran at all.
    let (table_name, branch_from_name) = parse_table_identifier(&table);
    let (namespace_parts, branch_from_ns) = parse_namespace(&namespace);
    let branch = branch_from_name
        .or(branch_from_ns)
        .unwrap_or_else(|| "main".to_string());

    let asset = match store
        .get_asset(
            tenant_id,
            &catalog_name,
            Some(branch.clone()),
            namespace_parts.clone(),
            table_name.clone(),
        )
        .await
    {
        Ok(Some(a)) => a,
        Ok(None) => return no_such_table(&format!("{}.{}", namespace_parts.join("."), table_name)),
        Err(e) => {
            tracing::error!(error = %e, "update_table: failed to load asset");
            return internal("Failed to load table");
        }
    };

    let scope = PermissionScope::Asset {
        catalog_id: catalog.id,
        namespace: namespace_parts.join("."),
        asset_id: asset.id,
    };

    match check_permission(&store, &session, &Action::Write, &scope).await {
        Ok(true) => (),
        Ok(false) => return forbidden("Forbidden"),
        Err(e) => {
            tracing::error!(error = %e, "update_table: permission check failed");
            return internal("Permission check failed");
        }
    }

    crate::metrics::inc(&crate::metrics::COMMITS_TOTAL);

    let mut retries = 0;
    const MAX_RETRIES: i32 = 5;

    while retries < MAX_RETRIES {
        let current_asset = match store
            .get_asset(
                tenant_id,
                &catalog_name,
                Some(branch.clone()),
                namespace_parts.clone(),
                table_name.clone(),
            )
            .await
        {
            Ok(Some(a)) => a,
            Ok(None) => {
                return no_such_table(&format!("{}.{}", namespace_parts.join("."), table_name))
            }
            Err(e) => {
                tracing::error!(error = %e, "update_table: failed to re-read asset");
                return internal("Failed to load table");
            }
        };

        let current_metadata_location = current_asset.properties.get("metadata_location").cloned();

        let metadata_bytes = if let Some(loc) = &current_metadata_location {
            match store.read_file(loc).await {
                Ok(bytes) => bytes,
                Err(e) => {
                    tracing::error!(error = %e, "update_table: failed to read metadata file");
                    return internal("Failed to read metadata file");
                }
            }
        } else {
            return internal("Table corrupted (no metadata location)");
        };

        // Parse metadata in a blocking task to avoid stalling the executor
        let metadata_vec = metadata_bytes.to_vec();
        let mut metadata: TableMetadata = match tokio::task::spawn_blocking(move || {
            serde_json::from_slice(&metadata_vec)
        })
        .await
        {
            Ok(Ok(m)) => m,
            Ok(Err(e)) => {
                tracing::error!(error = %e, "load_table: failed to parse metadata");
                return internal("Failed to parse metadata");
            }
            Err(e) => {
                tracing::error!(error = %e, "load_table: metadata parse task panicked");
                return internal("Failed to parse metadata");
            }
        };

        // Requirements are evaluated against the metadata we just read, on
        // every attempt. This is what makes the compare-and-swap retry below
        // safe: a writer whose view is stale is rejected instead of having its
        // update replayed onto a branch that moved on (A-1).
        if let Err(e) = commit::check_requirements(&metadata, &payload.requirements, true) {
            crate::metrics::inc(&crate::metrics::COMMITS_CONFLICTED);
            return commit_error_response(e);
        }

        if let Err(e) = commit::apply_updates(&mut metadata, &payload.updates, &branch) {
            return commit_error_response(e);
        }

        // B16b: `last-updated-ms` was only ever assigned inside `add_snapshot`,
        // so a commit of only `set-properties` / `add-schema` / `set-location` /
        // `add-spec` / `set-snapshot-ref` / `remove-snapshots` published a new
        // metadata file carrying an *unchanged* timestamp - and any consumer
        // that orders or dedupes metadata by that field treated the two versions
        // as identical. Every successful set of updates bumps it.
        metadata.last_updated_ms = Utc::now().timestamp_millis();

        // B13: record the metadata file this one supersedes. `metadata-log` was
        // initialised to an empty vec at table creation and never appended to,
        // so metadata time-travel and previous-version cleanup
        // (`write.metadata.previous-versions-max`) had nothing to work with.
        if let Some(previous) = &current_metadata_location {
            let log = metadata.metadata_log.get_or_insert_with(Vec::new);
            log.push(MetadataLogEntry {
                timestamp_ms: metadata.last_updated_ms,
                metadata_file: previous.clone(),
            });
            let max_entries = metadata
                .properties
                .as_ref()
                .and_then(|p| p.get("write.metadata.previous-versions-max"))
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(DEFAULT_PREVIOUS_VERSIONS_MAX);
            if log.len() > max_entries {
                let excess = log.len() - max_entries;
                log.drain(0..excess);
            }
        }

        let new_metadata_location = format!(
            "{}/metadata/00000-{}.metadata.json",
            metadata.location,
            Uuid::new_v4()
        );
        let metadata_json = match serde_json::to_string(&metadata) {
            Ok(json) => json,
            Err(e) => {
                tracing::error!(error = %e, "could not serialise table metadata");
                return internal("Failed to serialise table metadata");
            }
        };

        if store
            .write_file(&new_metadata_location, metadata_json.into_bytes())
            .await
            .is_err()
        {
            return internal("Failed to write new metadata");
        }

        match store
            .update_metadata_location(
                tenant_id,
                &catalog_name,
                Some(branch.clone()),
                namespace_parts.clone(),
                table_name.clone(),
                current_metadata_location.clone(),
                new_metadata_location.clone(),
            )
            .await
        {
            Ok(_) => {
                crate::metrics::inc(&crate::metrics::COMMITS_SUCCEEDED);
                // The audit write is no longer discarded with `let _ = ...`: a
                // dropped record means the commit happened with no trace of who
                // made it (C-18).
                if let Err(e) = store
                    .log_audit_event(
                        tenant_id,
                        pangolin_core::audit::AuditLogEntry::success(
                            tenant_id,
                            Some(session.user_id),
                            session.username.clone(),
                            pangolin_core::audit::AuditAction::CommitTable,
                            pangolin_core::audit::ResourceType::Table,
                            Some(asset.id),
                            format!("{}/{}/{}", catalog_name, namespace, table_name),
                        )
                        .with_metadata(serde_json::json!({
                            "new_metadata_location": new_metadata_location.clone(),
                            "snapshot_id": metadata.current_snapshot_id,
                            "sequence_number": metadata.last_sequence_number,
                        })),
                    )
                    .await
                {
                    crate::metrics::inc(&crate::metrics::AUDIT_WRITE_FAILURES);
                    tracing::error!(error = %e, "failed to write the table-commit audit record");
                }

                return (
                    StatusCode::OK,
                    Json(TableResponse::new(
                        Some(new_metadata_location.clone()),
                        metadata,
                        Some(asset.id),
                    )),
                )
                    .into_response();
            }
            Err(e) => {
                // The compare-and-swap lost: another writer published first.
                // Re-read and re-check requirements on the next pass.
                tracing::debug!(error = %e, attempt = retries, "metadata CAS lost, retrying");
                crate::metrics::inc(&crate::metrics::COMMIT_CAS_RETRIES);
                // B16d: the metadata file was written *before* the CAS, so on a
                // lost CAS it is unreferenced - and the old code just
                // `continue`d, orphaning it. Under contention that leaked up to
                // one file per retry, with up to five left behind on a final
                // give-up, and an orphan is indistinguishable from live metadata
                // from the outside so nothing could reap it later.
                if let Err(cleanup) = store.delete_file(&new_metadata_location).await {
                    tracing::warn!(
                        error = %cleanup,
                        location = %new_metadata_location,
                        "could not remove the metadata file orphaned by a lost CAS"
                    );
                }
                retries += 1;
                continue;
            }
        }
    }

    crate::metrics::inc(&crate::metrics::COMMITS_CONFLICTED);
    iceberg_error(
        StatusCode::CONFLICT,
        "CommitFailedException",
        "Failed to commit after the maximum number of retries",
    )
}

/// Map a commit failure onto the Iceberg REST error envelope.
fn commit_error_response(error: commit::CommitError) -> axum::response::Response {
    match error {
        commit::CommitError::RequirementFailed { .. } => iceberg_error(
            StatusCode::CONFLICT,
            "CommitFailedException",
            &error.to_string(),
        ),
        commit::CommitError::Unsupported { .. } => iceberg_error(
            StatusCode::NOT_IMPLEMENTED,
            "UnsupportedOperationException",
            &error.to_string(),
        ),
        commit::CommitError::Invalid { .. } => iceberg_error(
            StatusCode::BAD_REQUEST,
            "BadRequestException",
            &error.to_string(),
        ),
    }
}

/// Rename a table
///
/// B0c: this handler bound `Extension(session)` but never called
/// `check_permission` - the only table handler that didn't. Any tenant member
/// could move any table into any namespace: an effective delete (the table
/// vanishes from where its readers look for it) and a way to smuggle a table
/// into a namespace where the caller *does* have read rights. It now needs
/// `Write` on the source table and `Create` on the destination namespace.
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
        (status = 403, description = "Forbidden"),
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
    )
    .await
    {
        return response;
    }

    let source_ns = payload.source.namespace;
    let source_name = payload.source.name;
    let dest_ns = payload.destination.namespace;
    let dest_name = payload.destination.name;
    let branch = Some("main".to_string());

    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return no_such_namespace(&catalog_name),
        Err(e) => {
            tracing::error!(error = %e, "rename_table: failed to load catalog");
            return internal("Failed to load catalog");
        }
    };

    let source_asset = match store
        .get_asset(
            tenant_id,
            &catalog_name,
            branch.clone(),
            source_ns.clone(),
            source_name.clone(),
        )
        .await
    {
        Ok(Some(a)) => a,
        Ok(None) => return no_such_table(&format!("{}.{}", source_ns.join("."), source_name)),
        Err(e) => {
            tracing::error!(error = %e, "rename_table: failed to load source asset");
            return internal("Failed to load source table");
        }
    };

    // Write on the source: renaming is a mutation of the table's identity, and
    // from every reader's point of view it is a delete at the old path.
    let source_scope = PermissionScope::Asset {
        catalog_id: catalog.id,
        namespace: source_ns.join("."),
        asset_id: source_asset.id,
    };
    match check_permission(&store, &session, &Action::Write, &source_scope).await {
        Ok(true) => (),
        Ok(false) => return forbidden("Forbidden: no write access to the source table"),
        Err(e) => {
            tracing::error!(error = %e, "rename_table: source permission check failed");
            return internal("Permission check failed");
        }
    }

    // Create on the destination namespace: otherwise a caller with write on one
    // table could plant it anywhere they can read.
    let dest_scope = PermissionScope::Namespace {
        catalog_id: catalog.id,
        namespace: dest_ns.join("."),
    };
    match check_permission(&store, &session, &Action::Create, &dest_scope).await {
        Ok(true) => (),
        Ok(false) => return forbidden("Forbidden: no create access in the destination namespace"),
        Err(e) => {
            tracing::error!(error = %e, "rename_table: destination permission check failed");
            return internal("Permission check failed");
        }
    }

    // The spec returns 409 rather than clobbering an existing destination.
    match store
        .get_asset(
            tenant_id,
            &catalog_name,
            branch.clone(),
            dest_ns.clone(),
            dest_name.clone(),
        )
        .await
    {
        Ok(Some(_)) => {
            return table_already_exists(&format!("{}.{}", dest_ns.join("."), dest_name))
        }
        Ok(None) => {}
        Err(e) => {
            tracing::error!(error = %e, "rename_table: destination existence check failed");
            return internal("Failed to check the destination table");
        }
    }

    match store
        .rename_asset(
            tenant_id,
            &catalog_name,
            branch,
            source_ns.clone(),
            source_name.clone(),
            dest_ns.clone(),
            dest_name.clone(),
        )
        .await
    {
        Ok(_) => {
            let _ = store
                .log_audit_event(
                    tenant_id,
                    pangolin_core::audit::AuditLogEntry::success(
                        tenant_id,
                        Some(session.user_id),
                        session.username.clone(),
                        pangolin_core::audit::AuditAction::RenameTable,
                        pangolin_core::audit::ResourceType::Table,
                        Some(source_asset.id),
                        format!(
                            "{}/{}.{} -> {}.{}",
                            catalog_name,
                            source_ns.join("."),
                            source_name,
                            dest_ns.join("."),
                            dest_name
                        ),
                    ),
                )
                .await;

            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "rename_table: rename failed");
            no_such_table(&format!("{}.{}", source_ns.join("."), source_name))
        }
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
    )
    .await
    {
        return response;
    }

    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return no_such_namespace(&catalog_name),
        Err(e) => {
            tracing::error!(error = %e, "delete_table: failed to load catalog");
            return internal("Failed to load catalog");
        }
    };

    let (table_name, branch_from_name) = parse_table_identifier(&table);
    let (namespace_parts, branch_from_ns) = parse_namespace(&namespace);
    let branch = branch_from_name
        .or(branch_from_ns)
        .or(Some("main".to_string()));

    let asset = match store
        .get_asset(
            tenant_id,
            &catalog_name,
            branch.clone(),
            namespace_parts.clone(),
            table_name.clone(),
        )
        .await
    {
        Ok(Some(a)) => a,
        Ok(None) => return no_such_table(&format!("{}.{}", namespace_parts.join("."), table_name)),
        Err(e) => {
            tracing::error!(error = %e, "delete_table: failed to load asset");
            return internal("Failed to load table");
        }
    };

    let scope = PermissionScope::Asset {
        catalog_id: catalog.id,
        namespace: namespace_parts.join("."),
        asset_id: asset.id,
    };

    match check_permission(&store, &session, &Action::Delete, &scope).await {
        Ok(true) => (),
        Ok(false) => return forbidden("Forbidden"),
        Err(e) => {
            tracing::error!(error = %e, "delete_table: permission check failed");
            return internal("Permission check failed");
        }
    }

    match store
        .delete_asset(
            tenant_id,
            &catalog_name,
            branch,
            namespace_parts,
            table_name,
        )
        .await
    {
        Ok(_) => {
            let _ = store
                .log_audit_event(
                    tenant_id,
                    pangolin_core::audit::AuditLogEntry::success(
                        tenant_id,
                        Some(session.user_id),
                        session.username.clone(),
                        pangolin_core::audit::AuditAction::DropTable,
                        pangolin_core::audit::ResourceType::Table,
                        Some(asset.id),
                        format!("{}/{}/{}", catalog_name, namespace, table),
                    ),
                )
                .await;

            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "delete_table: delete failed");
            internal("Failed to delete table")
        }
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
        Method::HEAD,
        &path,
        None,
        HeaderMap::new(),
    )
    .await
    {
        return response;
    }

    let (namespace_parts, branch_from_ns) = parse_namespace(&namespace);
    let (table_name, branch_from_table) = parse_table_identifier(&table);
    let branch_name = branch_from_table.or(branch_from_ns);

    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!(error = %e, "table_exists: failed to load catalog");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let asset = match store
        .get_asset(
            tenant_id,
            &catalog_name,
            branch_name,
            namespace_parts.clone(),
            table_name,
        )
        .await
    {
        Ok(Some(a)) => a,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!(error = %e, "table_exists: failed to load asset");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Existence is information. Without a check this endpoint is an oracle that
    // reports whether a table the caller cannot read exists - every sibling
    // handler gates on Read, so this one does too.
    let scope = PermissionScope::Asset {
        catalog_id: catalog.id,
        namespace: namespace_parts.join("."),
        asset_id: asset.id,
    };
    match check_permission(&store, &session, &Action::Read, &scope).await {
        Ok(true) => StatusCode::OK.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!(error = %e, "table_exists: permission check failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
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
