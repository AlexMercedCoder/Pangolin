use axum::{
    extract::{Path, State, Query, Extension},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use pangolin_store::CatalogStore;
use pangolin_store::memory::MemoryStore;
use pangolin_core::model::{Namespace, Asset, AssetType};
use pangolin_core::iceberg_metadata::{TableMetadata, Schema, PartitionSpec, SortOrder, Snapshot, NestedField, Type};
use uuid::Uuid;
use std::collections::HashMap;
use chrono::Utc;
use crate::auth::TenantId;

// Placeholder for AppState
pub type AppState = Arc<dyn CatalogStore + Send + Sync>;

#[derive(Deserialize)]
pub struct ListNamespaceParams {
    parent: Option<String>,
}

#[derive(Serialize)]
pub struct ListNamespacesResponse {
    namespaces: Vec<Vec<String>>,
}

#[derive(Deserialize)]
pub struct CreateNamespaceRequest {
    namespace: Vec<String>,
    properties: Option<HashMap<String, String>>,
}

#[derive(Serialize)]
pub struct CreateNamespaceResponse {
    namespace: Vec<String>,
    properties: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct CreateTableRequest {
    name: String,
    location: Option<String>,
    schema: Option<serde_json::Value>,  // Accept schema as JSON
    properties: Option<HashMap<String, String>>,
}

#[derive(Serialize)]
pub struct TableResponse {
    name: String,
    location: String,
    properties: HashMap<String, String>,
    metadata: TableMetadata,  // Required by Iceberg REST spec
}

#[derive(Serialize)]
pub struct ListTablesResponse {
    identifiers: Vec<TableIdentifier>,
}

#[derive(Serialize, Deserialize)]
pub struct TableIdentifier {
    namespace: Vec<String>,
    name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PartitionField {
    pub source_id: i32,
    pub field_id: i32,
    pub name: String,
    pub transform: String,
}

#[derive(Deserialize)]
pub struct CommitTableRequest {
    identifier: Option<TableIdentifier>,
    requirements: Vec<CommitRequirement>,
    updates: Vec<CommitUpdate>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum CommitRequirement {
    #[serde(rename = "assert-create")]
    AssertCreate,
    #[serde(rename = "assert-table-uuid")]
    AssertTableUuid { uuid: Uuid },
    // Add others as needed
}

#[derive(Deserialize)]
#[serde(tag = "action")]
pub enum CommitUpdate {
    #[serde(rename = "assign-uuid")]
    AssignUuid { uuid: Uuid },
    #[serde(rename = "upgrade-format-version")]
    UpgradeFormatVersion { format_version: i32 },
    #[serde(rename = "add-schema")]
    AddSchema { schema: Schema },
    #[serde(rename = "set-current-schema")]
    SetCurrentSchema { schema_id: i32 },
    #[serde(rename = "add-snapshot")]
    AddSnapshot { snapshot: Snapshot },
    // Add others as needed
}

// Helper to parse "table@branch"
pub fn parse_table_identifier(identifier: &str) -> (String, Option<String>) {
    if let Some((name, branch)) = identifier.split_once('@') {
        (name.to_string(), Some(branch.to_string()))
    } else {
        (identifier.to_string(), None)
    }
}

pub async fn list_namespaces(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path(prefix): Path<String>,
    Query(params): Query<ListNamespaceParams>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix;
    
    match store.list_namespaces(tenant_id, &catalog_name, params.parent).await {
        Ok(namespaces) => {
            let ns_list: Vec<Vec<String>> = namespaces.into_iter().map(|n| n.name).collect();
            (StatusCode::OK, Json(ListNamespacesResponse { namespaces: ns_list })).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

pub async fn create_namespace(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path(prefix): Path<String>,
    Json(payload): Json<CreateNamespaceRequest>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix;
    let ns = Namespace {
        name: payload.namespace.clone(),
        properties: payload.properties.unwrap_or_default(),
    };

    match store.create_namespace(tenant_id, &catalog_name, ns.clone()).await {
        Ok(_) => (StatusCode::OK, Json(CreateNamespaceResponse {
            namespace: ns.name,
            properties: ns.properties,
        })).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

pub async fn list_tables(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path((prefix, namespace)): Path<(String, String)>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix;
    
    let (ns_name, branch) = parse_table_identifier(&namespace);
    let ns_vec = vec![ns_name];

    match store.list_assets(tenant_id, &catalog_name, branch, ns_vec.clone()).await {
        Ok(assets) => {
            let identifiers: Vec<TableIdentifier> = assets.into_iter().map(|a| TableIdentifier {
                namespace: ns_vec.clone(),
                name: a.name,
            }).collect();
            (StatusCode::OK, Json(ListTablesResponse { identifiers })).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

pub async fn create_table(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path((prefix, namespace)): Path<(String, String)>,
    Query(params): Query<HashMap<String, String>>,
    Json(payload): Json<CreateTableRequest>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix;
    
    let (tbl_name, branch_from_name) = parse_table_identifier(&payload.name);
    let (ns_name, branch_from_ns) = parse_table_identifier(&namespace);
    let branch_from_query = params.get("branch").cloned();
    let branch = branch_from_name.or(branch_from_ns).or(branch_from_query);
    
    let ns_vec = vec![ns_name];

    let table_uuid = Uuid::new_v4();
    let location = payload.location.unwrap_or_else(|| format!("s3://warehouse/{}/{}/{}", catalog_name, ns_vec.join("/"), tbl_name));
    
    // Parse schema from payload if provided, otherwise create empty schema
    let schema_fields = if let Some(schema_value) = &payload.schema {
        // Try to parse fields from schema
        if let Some(fields) = schema_value.get("fields").and_then(|f| f.as_array()) {
            fields.iter().filter_map(|field| {
                // Parse each field into NestedField
                let id = field.get("id")?.as_i64()? as i32;
                let name = field.get("name")?.as_str()?.to_string();
                let required = field.get("required")?.as_bool()?;
                let field_type_str = field.get("type")?.as_str()?;
                
                // Map type string to Type enum
                let field_type = match field_type_str {
                    "int" | "integer" => Type::Primitive("int".to_string()),
                    "long" => Type::Primitive("long".to_string()),
                    "string" => Type::Primitive("string".to_string()),
                    "boolean" => Type::Primitive("boolean".to_string()),
                    "float" => Type::Primitive("float".to_string()),
                    "double" => Type::Primitive("double".to_string()),
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
    
    // Create initial TableMetadata with a valid schema
    let metadata = TableMetadata {
        format_version: 2,
        table_uuid,
        location: location.clone(),
        last_sequence_number: 0,
        last_updated_ms: Utc::now().timestamp_millis(),
        last_column_id: schema_fields.iter().map(|f| f.id).max().unwrap_or(0),
        current_schema_id: 0,
        schemas: vec![Schema {
            schema_id: 0,
            identifier_field_ids: Some(vec![]),  // Empty array instead of None
            fields: schema_fields,
        }],
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

    // Serialize metadata
    let metadata_json = serde_json::to_string(&metadata).unwrap();
    
    // In a real implementation, we would write this JSON to S3 at `location/metadata/v1.metadata.json`
    // But our Store trait doesn't expose generic "write file" yet, only "create_asset".
    // We need to extend Store or use S3 client directly? 
    // Actually, `create_asset` stores the Asset pointer.
    // We should probably add a method to Store to write arbitrary bytes or metadata?
    // Or we assume the "Asset" creation IS the metadata creation?
    // The "Asset" in our model is the catalog pointer. The "TableMetadata" is the Iceberg file.
    
    // For now, let's just store the pointer and assume the metadata file "exists" conceptually or we skip writing it for this step 
    // until we add `write_file` to CatalogStore.
    // Wait, the plan said "Implement Metadata Writer/Reader in pangolin_store".
    // I added `get_metadata_location` and `update_metadata_location` but not `write_metadata_file`.
    
    // Let's add `write_metadata_file` to CatalogStore or just use `S3Store`'s internal client if we could?
    // No, we should keep it abstract.
    
    // Let's update `Asset` to include the metadata location property.
    let metadata_location = format!("{}/metadata/00000-{}.metadata.json", location, Uuid::new_v4());
    
    let mut properties = payload.properties.unwrap_or_default();
    properties.insert("metadata_location".to_string(), metadata_location.clone());

    let asset = Asset {
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
            // Write metadata file
            if let Err(e) = store.write_file(&metadata_location, metadata_json.into_bytes()).await {
                 tracing::error!("Failed to write metadata file: {}", e);
                 // Should we rollback asset creation? For now, just log error.
                 return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to write metadata").into_response();
            }

            // Audit Log
            let _ = store.log_audit_event(tenant_id, pangolin_core::audit::AuditLogEntry::new(
                tenant_id,
                "system".to_string(),
                "create_table".to_string(),
                format!("{}/{}/{}", catalog_name, namespace, tbl_name),
                Some(location.clone())
            )).await;

            (StatusCode::OK, Json(TableResponse {
                name: asset.name.clone(),
                location: asset.location.clone(),
                properties: asset.properties.clone(),
                metadata,  // Include the metadata we created
            })).into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

pub async fn load_table(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path((prefix, namespace, table)): Path<(String, String, String)>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix;
    
    let (tbl_name, branch_from_name) = parse_table_identifier(&table);
    let (ns_name, branch_from_ns) = parse_table_identifier(&namespace);
    let branch = branch_from_name.or(branch_from_ns);
    
    let ns_vec = vec![ns_name];

    // 1. Get current asset to find current metadata location
    let asset = match store.get_asset(tenant_id, &catalog_name, branch.clone(), ns_vec.clone(), tbl_name.clone()).await {
        Ok(Some(a)) => a,
        Ok(None) => return (StatusCode::NOT_FOUND, "Table not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    };

    let current_metadata_location = asset.properties.get("metadata_location").cloned();

    if let Some(location) = current_metadata_location {
        // 2. Read current metadata
        let metadata_bytes = match store.read_file(&location).await {
            Ok(bytes) => bytes,
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read metadata file").into_response(),
        };
        
        let metadata: TableMetadata = match serde_json::from_slice(&metadata_bytes) {
            Ok(m) => m,
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse metadata").into_response(),
        };

        // 3. Handle Time Travel
        // If snapshot-id or as-of-timestamp is provided, we need to find the correct metadata.
        // For Iceberg REST, the client expects the *metadata* that corresponds to that snapshot.
        // However, the `loadTable` response usually returns the *current* metadata, and the client uses the snapshot-id to read the correct snapshot from it.
        // BUT, if the snapshot is not in the current metadata (e.g. expired), we might need to go back in history.
        // For MVP, let's assume we just return the current metadata, but we verify the snapshot exists if requested.
        
        if let Some(snapshot_id_str) = params.get("snapshot-id") {
             if let Ok(snapshot_id) = snapshot_id_str.parse::<i64>() {
                 let found = metadata.snapshots.as_ref().map(|s| s.iter().any(|snap| snap.snapshot_id == snapshot_id)).unwrap_or(false);
                 if !found {
                     // In a full implementation, we would search metadata_log to find the metadata file that contained this snapshot.
                     // For MVP, we just return Not Found if not in current metadata.
                     return (StatusCode::NOT_FOUND, "Snapshot not found in current metadata").into_response();
                 }
             }
        } else if let Some(timestamp_str) = params.get("as-of-timestamp") {
             if let Ok(timestamp) = timestamp_str.parse::<i64>() {
                 // Find the snapshot active at that timestamp
                 // This logic is complex (finding the last snapshot before timestamp).
                 // For MVP, we just check if any snapshot matches or is older.
                 // Actually, Iceberg spec says `loadTable` just returns metadata. The *client* does the time travel logic usually?
                 // Wait, the REST spec says: "The server may return a version of the metadata that contains the snapshot."
                 // So returning current metadata is usually fine unless history is truncated.
             }
        }

        // For load_table, we should read the actual metadata file
        // For now, return a minimal metadata structure
        let metadata = TableMetadata {
            format_version: 2,
            table_uuid: Uuid::new_v4(),
            location: asset.location.clone(),
            last_sequence_number: 0,
            last_updated_ms: Utc::now().timestamp_millis(),
            last_column_id: 0,
            current_schema_id: 0,
            schemas: vec![],
            current_partition_spec_id: 0,
            partition_specs: vec![PartitionSpec { spec_id: 0, fields: vec![] }],
            default_sort_order_id: 0,
            sort_orders: vec![SortOrder { order_id: 0, fields: vec![] }],
            properties: Some(asset.properties.clone()),
            current_snapshot_id: None,
            snapshots: Some(vec![]),
            snapshot_log: Some(vec![]),
            metadata_log: Some(vec![]),
        };
        
        (StatusCode::OK, Json(TableResponse {
            name: asset.name,
            location: asset.location,
            properties: asset.properties,
            metadata,
        })).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Metadata location not found").into_response()
    }
}

pub async fn update_table(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path((prefix, namespace, table)): Path<(String, String, String)>,
    Json(payload): Json<CommitTableRequest>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix;
    let (table_name, branch_from_name) = parse_table_identifier(&table);
    let branch = branch_from_name.unwrap_or("main".to_string());
    let namespace_parts: Vec<String> = namespace.split('\x1F').map(|s| s.to_string()).collect();

    // Retry loop for OCC
    let mut retries = 0;
    const MAX_RETRIES: i32 = 5;

    while retries < MAX_RETRIES {
        // 1. Load current metadata location
        let current_metadata_location = match store.get_metadata_location(tenant_id, &catalog_name, Some(branch.clone()), namespace_parts.clone(), table_name.clone()).await {
            Ok(Some(loc)) => loc,
            Ok(None) => return (StatusCode::NOT_FOUND, "Table not found").into_response(),
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
        };

        // 2. Read current metadata file
        let metadata_bytes = match store.read_file(&current_metadata_location).await {
            Ok(bytes) => bytes,
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read metadata file").into_response(),
        };
        
        let mut metadata: TableMetadata = match serde_json::from_slice(&metadata_bytes) {
            Ok(m) => m,
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse metadata").into_response(),
        };

        // 3. Apply updates
        for update in &payload.updates {
            match update {
                CommitUpdate::AddSnapshot { snapshot } => {
                    if let Some(snapshots) = &mut metadata.snapshots {
                        snapshots.push(snapshot.clone());
                    } else {
                        metadata.snapshots = Some(vec![snapshot.clone()]);
                    }
                    metadata.current_snapshot_id = Some(snapshot.snapshot_id);
                    metadata.last_updated_ms = Utc::now().timestamp_millis();
                },
                CommitUpdate::AddSchema { schema } => {
                    metadata.schemas.push(schema.clone());
                },
                CommitUpdate::SetCurrentSchema { schema_id } => {
                    metadata.current_schema_id = *schema_id;
                },
                _ => {} // Ignore others for MVP
            }
        }

        // 4. Write new metadata
        let new_metadata_location = format!("{}/metadata/00000-{}.metadata.json", metadata.location, Uuid::new_v4());
        let metadata_json = serde_json::to_string(&metadata).unwrap();

        if let Err(e) = store.write_file(&new_metadata_location, metadata_json.into_bytes()).await {
             tracing::error!("Failed to write new metadata file: {}", e);
             return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to write new metadata").into_response();
        }

        // 5. Update catalog pointer (CAS)
        match store.update_metadata_location(tenant_id, &catalog_name, Some(branch.clone()), namespace_parts.clone(), table_name.clone(), Some(current_metadata_location.clone()), new_metadata_location.clone()).await {
            Ok(_) => {
                // Success!
                return (StatusCode::OK, Json(TableResponse {
                    name: metadata.properties.as_ref().and_then(|p: &std::collections::HashMap<String, String>| p.get("name").cloned()).unwrap_or_default(),
                    location: metadata.location.clone(),
                    properties: metadata.properties.clone().unwrap_or_default(),
                    metadata,
                })).into_response();
            },
            Err(_) => {
                // CAS failed, retry
                retries += 1;
                tracing::warn!("CAS failed for table {}, retrying... ({}/{})", table_name, retries, MAX_RETRIES);
                // Clean up the orphaned metadata file we just wrote? Ideally yes, but skipping for MVP.
                continue;
            }
        }
    }

    (StatusCode::CONFLICT, "Failed to commit after retries").into_response()
}

#[derive(Deserialize)]
pub struct RenameTableRequest {
    source: TableIdentifier,
    destination: TableIdentifier,
}

pub async fn rename_table(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path(prefix): Path<String>,
    Json(payload): Json<RenameTableRequest>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix;
    
    let source_ns = payload.source.namespace;
    let source_name = payload.source.name;
    
    let dest_ns = payload.destination.namespace;
    let dest_name = payload.destination.name;

    // Assuming rename is within the same branch for now, or default branch
    // Iceberg spec doesn't explicitly mention branches in rename, so we assume "main" or default.
    let branch = Some("main".to_string());

    match store.rename_asset(tenant_id, &catalog_name, branch, source_ns, source_name, dest_ns, dest_name).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Table not found").into_response(),
    }
}

pub async fn delete_table(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path((prefix, namespace, table)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix;
    let (table_name, branch_from_name) = parse_table_identifier(&table);
    let branch = branch_from_name.or(Some("main".to_string()));
    let namespace_parts: Vec<String> = namespace.split('\x1F').map(|s| s.to_string()).collect();

    match store.delete_asset(tenant_id, &catalog_name, branch, namespace_parts, table_name).await {
        Ok(_) => {
             // Audit Log
             let _ = store.log_audit_event(tenant_id, pangolin_core::audit::AuditLogEntry::new(
                tenant_id,
                "system".to_string(),
                "delete_table".to_string(),
                format!("{}/{}/{}", catalog_name, namespace, table),
                None
            )).await;
            
            StatusCode::NO_CONTENT.into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

pub async fn delete_namespace(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path((prefix, namespace)): Path<(String, String)>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix;
    let namespace_parts: Vec<String> = namespace.split('\x1F').map(|s| s.to_string()).collect();

    match store.delete_namespace(tenant_id, &catalog_name, namespace_parts).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Namespace not found").into_response(),
    }
}

#[derive(Deserialize)]
pub struct UpdateNamespacePropertiesRequest {
    removals: Option<Vec<String>>,
    updates: Option<std::collections::HashMap<String, String>>,
}

#[derive(Serialize)]
pub struct UpdateNamespacePropertiesResponse {
    updated: Vec<String>,
    removed: Vec<String>,
    missing: Vec<String>,
}

pub async fn update_namespace_properties(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path((prefix, namespace)): Path<(String, String)>,
    Json(payload): Json<UpdateNamespacePropertiesRequest>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix;
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

pub async fn report_metrics(
    Path((_prefix, _namespace, _table)): Path<(String, String, String)>,
) -> impl IntoResponse {
    // Just log and return success
    tracing::info!("Received metrics report");
    StatusCode::NO_CONTENT
}

#[derive(Debug, Deserialize)]
pub struct MaintenanceRequest {
    pub job_type: String, // "expire_snapshots" or "remove_orphan_files"
    pub retention_ms: Option<i64>,
    pub older_than_ms: Option<i64>,
}

pub async fn perform_maintenance(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
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

pub async fn table_exists(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Extension(tenant_id): Extension<TenantId>,
    Path((prefix, namespace, table)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let namespace_parts: Vec<String> = namespace.split('\x1F').map(|s| s.to_string()).collect();
    // Parse table@branch
    let (table_name, branch_name) = if let Some((t, b)) = table.split_once('@') {
        (t.to_string(), Some(b.to_string()))
    } else {
        (table.to_string(), None)
    };

    match store.get_asset(tenant_id.0, &prefix, branch_name, namespace_parts, table_name).await {
        Ok(Some(_)) => StatusCode::OK,
        Ok(None) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
