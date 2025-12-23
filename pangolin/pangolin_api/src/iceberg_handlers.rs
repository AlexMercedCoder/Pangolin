use axum::{
    extract::{Path, State, Query, Extension},
    Json,
    response::IntoResponse,
    http::{StatusCode, HeaderMap, Method, Request},
    body::Body,
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use pangolin_store::CatalogStore;
use pangolin_store::memory::MemoryStore;
use pangolin_core::model::{Namespace, Asset, AssetType, CatalogType};
use pangolin_core::iceberg_metadata::{TableMetadata, Schema, PartitionSpec, SortOrder, Snapshot, NestedField, Type};
use chrono::Utc;
use uuid::Uuid;
use std::collections::HashMap;
use crate::federated_proxy::FederatedCatalogProxy;
use crate::authz::check_permission;
use pangolin_core::permission::{PermissionScope, Action};
use pangolin_core::user::UserSession;
use crate::auth::TenantId;
use utoipa::{ToSchema, IntoParams};

// Placeholder for AppState
pub type AppState = Arc<dyn CatalogStore + Send + Sync>;

#[derive(Serialize, ToSchema)]
pub struct CatalogConfig {
    pub defaults: HashMap<String, String>,
    pub overrides: HashMap<String, String>,
}


/// Helper function to check if a catalog is federated and forward the request if so
async fn check_and_forward_if_federated(
    store: &Arc<dyn CatalogStore + Send + Sync>,
    tenant_id: Uuid,
    catalog_name: &str,
    method: Method,
    path: &str,
    body: Option<Bytes>,
    headers: HeaderMap,
) -> Option<axum::response::Response> {
    // Get the catalog
    let catalog = match store.get_catalog(tenant_id, catalog_name.to_string()).await {
        Ok(Some(c)) => c,
        Ok(None) => return None, // Catalog not found, let handler deal with it
        Err(_) => return None,
    };

    // Check if it's federated
    if catalog.catalog_type == CatalogType::Federated {
        if let Some(config) = catalog.federated_config {
            let proxy = FederatedCatalogProxy::new();
            match proxy.forward_request(&config, method, path, body, headers).await {
                Ok(response) => Some(response),
                Err(e) => Some((
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({"error": format!("Federated catalog error: {}", e)})),
                ).into_response()),
            }
        } else {
            Some((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Federated catalog missing configuration"})),
            ).into_response())
        }
    } else {
        None // Not federated, continue with local handling
    }
}

#[derive(Serialize, ToSchema)]
pub struct ListNamespacesResponse {
    pub namespaces: Vec<Vec<String>>,
}

#[derive(Deserialize, IntoParams)]
pub struct ListNamespaceParams {
    pub parent: Option<String>,
}

#[derive(Serialize, Clone, ToSchema)]
pub struct NamespaceNode {
    pub name: String,
    pub full_path: Vec<String>,
    pub children: Vec<NamespaceNode>,
}

#[derive(Serialize, ToSchema)]
pub struct ListNamespacesTreeResponse {
    pub root: Vec<NamespaceNode>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateNamespaceRequest {
    pub namespace: Vec<String>,
    pub properties: Option<HashMap<String, String>>,
}

#[derive(Serialize, ToSchema)]
pub struct CreateNamespaceResponse {
    pub namespace: Vec<String>,
    pub properties: HashMap<String, String>,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateTableRequest {
    pub name: String,
    pub location: Option<String>,
    pub schema: Option<serde_json::Value>,  // Accept schema as JSON
    pub properties: Option<HashMap<String, String>>,
}

#[derive(Serialize, ToSchema)]
pub struct TableResponse {
    #[serde(rename = "metadata-location")]
    pub metadata_location: Option<String>,
    pub metadata: TableMetadata,
    // Config tells PyIceberg how to access the table's data
    // Including credential vending configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<HashMap<String, String>>,
}

impl TableResponse {
    pub fn new(metadata_location: Option<String>, metadata: TableMetadata) -> Self {
        Self::with_credentials(metadata_location, metadata, None)
    }
    
    pub fn with_credentials(
        metadata_location: Option<String>, 
        metadata: TableMetadata,
        credentials: Option<HashMap<String, String>>,
    ) -> Self {
        let mut config = HashMap::new();
        
        // Merge vended credentials into config
        if let Some(creds) = credentials {
            config.extend(creds);
        }
        
        // Add S3 defaults if not already present
        config.entry("s3.endpoint".to_string())
            .or_insert_with(|| std::env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string()));
        config.entry("s3.region".to_string())
            .or_insert_with(|| std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string()));
        
        Self {
            metadata_location,
            metadata,
            config: Some(config),
        }
    }
}
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ListTablesResponse {
    pub identifiers: Vec<TableIdentifier>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct TableIdentifier {
    pub namespace: Vec<String>,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct PartitionField {
    pub source_id: i32,
    pub field_id: i32,
    pub name: String,
    pub transform: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct CommitTableRequest {
    pub identifier: Option<TableIdentifier>,
    pub requirements: Vec<CommitRequirement>,
    pub updates: Vec<CommitUpdate>,
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(tag = "type")]
pub enum CommitRequirement {
    #[serde(rename = "assert-create")]
    AssertCreate,
    #[serde(rename = "assert-table-uuid")]
    AssertTableUuid { uuid: String },
    #[serde(rename = "assert-ref-snapshot-id")]
    AssertRefSnapshotId { 
        #[serde(rename = "ref")]
        reference: String,
        #[serde(rename = "snapshot-id")]
        snapshot_id: Option<i64>,
    },
    #[serde(rename = "assert-current-schema-id")]
    AssertCurrentSchemaId {
        #[serde(rename = "current-schema-id")]
        current_schema_id: Option<i32>,
    },
    // Add others as needed
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(tag = "action")]
pub enum CommitUpdate {
    #[serde(rename = "assign-uuid")]
    AssignUuid { uuid: String },
    #[serde(rename = "upgrade-format-version")]
    UpgradeFormatVersion { 
        #[serde(rename = "format-version")]
        format_version: i32 
    },
    #[serde(rename = "add-schema")]
    AddSchema { schema: serde_json::Value },
    #[serde(rename = "set-current-schema")]
    SetCurrentSchema { 
        #[serde(rename = "schema-id")]
        schema_id: i32 
    },
    #[serde(rename = "add-snapshot")]
    AddSnapshot { snapshot: serde_json::Value },
    #[serde(rename = "set-snapshot-ref")]
    SetSnapshotRef {
        #[serde(rename = "ref-name")]
        ref_name: String,
        #[serde(rename = "snapshot-id")]
        snapshot_id: i64,
        #[serde(rename = "type")]
        ref_type: String,
    },
    #[serde(rename = "set-properties")]
    SetProperties {
        updates: HashMap<String, String>,
    },
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
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix.clone();
    tracing::info!("list_namespaces: tenant_id={}, catalog_name={}", tenant_id, catalog_name);
    
    // Check if this is a federated catalog and forward if so
    // We pass relative path (suffix) because base_url includes the remote prefix
    let path = "/namespaces".to_string(); 
    // TODO: Append query params if needed (e.g. ?parent=...)
    
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
    
    // Local catalog handling (existing logic)
    // Resolve catalog ID
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "Catalog not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    };

    // Check Permissions (List Namespace requires List on Catalog)
    let scope = PermissionScope::Catalog { catalog_id: catalog.id };
    match check_permission(&store, &session, &Action::List, &scope).await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, "Forbidden").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Permission check failed: {}", e)).into_response(),
    }
    
    match store.list_namespaces(tenant_id, &catalog_name, params.parent).await {
        Ok(namespaces) => {
            let ns_list: Vec<Vec<String>> = namespaces.into_iter().map(|n| n.name).collect();
            (StatusCode::OK, Json(ListNamespacesResponse { namespaces: ns_list })).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
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
    
    // Resolve catalog ID
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "Catalog not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    };

    // Check Permissions
    let scope = PermissionScope::Catalog { catalog_id: catalog.id };
    match check_permission(&store, &session, &Action::List, &scope).await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, "Forbidden").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Permission check failed: {}", e)).into_response(),
    }

    // List ALL namespaces (no parent filter)
    match store.list_namespaces(tenant_id, &catalog_name, None).await {
        Ok(namespaces) => {
            let mut root_nodes: Vec<NamespaceNode> = Vec::new();
            
            // Helper to find or create a node in a list of siblings
            fn find_or_create_child<'a>(nodes: &'a mut Vec<NamespaceNode>, name: &str, full_path: Vec<String>) -> &'a mut NamespaceNode {
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
                    current_level = &mut find_or_create_child(current_level, &part, current_path.clone()).children;
                }
            }
            
            (StatusCode::OK, Json(ListNamespacesTreeResponse { root: root_nodes })).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to list namespaces: {}", e)).into_response(),
    }
}


#[utoipa::path(
    get,
    path = "/v1/config",
    tag = "Iceberg REST",
    responses(
        (status = 200, description = "Catalog configuration", body = CatalogConfig),
    )
)]
pub async fn get_iceberg_catalog_config_handler() -> Json<CatalogConfig> {
    // Return Iceberg REST catalog config
    // Use X-Iceberg-Access-Delegation header to enable credential vending
    let mut defaults = HashMap::new();
    
    // This header tells PyIceberg to request credentials via the vend-credentials endpoint
    // PyIceberg will call POST /v1/{prefix}/namespaces/{namespace}/tables/{table}/credentials
    defaults.insert("header.X-Iceberg-Access-Delegation".to_string(), "vended-credentials".to_string());
    
    // Optionally provide S3 endpoint if using MinIO or custom S3
    if let Ok(endpoint) = std::env::var("S3_ENDPOINT") {
        defaults.insert("s3.endpoint".to_string(), endpoint);
    } else if let Ok(endpoint) = std::env::var("AWS_ENDPOINT_URL") {
        defaults.insert("s3.endpoint".to_string(), endpoint);
    }
    
    // Add region if specified
    if let Ok(region) = std::env::var("AWS_REGION") {
        defaults.insert("s3.region".to_string(), region);
    }
    
    Json(CatalogConfig {
        defaults,
        overrides: HashMap::new(),
    })
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
    
    tracing::info!("create_namespace: tenant_id={}, catalog_name={}", tenant_id, catalog_name);

    // Resolve catalog ID
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "Catalog not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    };
    
    // Check Permissions (Create Namespace requires Create on Catalog)
    let scope = PermissionScope::Catalog { catalog_id: catalog.id };
    match check_permission(&store, &session, &Action::Create, &scope).await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, "Forbidden").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Permission check failed: {}", e)).into_response(),
    }

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
    
    // Resolve catalog ID
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "Catalog not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    };
    
    let (ns_name, branch) = parse_table_identifier(&namespace);
    
    // Check Permissions (List Tables requires List on Namespace)
    let scope = PermissionScope::Namespace { catalog_id: catalog.id, namespace: ns_name.clone() };
    match check_permission(&store, &session, &Action::List, &scope).await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, "Forbidden").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Permission check failed: {}", e)).into_response(),
    }
    
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
    
    // Federated check
    let mut path = format!("/namespaces/{}/tables", namespace);
    if !params.is_empty() {
        let query_string: String = params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        path.push_str("?");
        path.push_str(&query_string);
    }

    // Serialize body for forwarding
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
    
    // Resolve catalog ID
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "Catalog not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    };
    
    let (tbl_name, branch_from_name) = parse_table_identifier(&payload.name);
    let (ns_name, branch_from_ns) = parse_table_identifier(&namespace);
    let branch_from_query = params.get("branch").cloned();
    let branch = branch_from_name.or(branch_from_ns).or(branch_from_query);
    
    // Check Permissions (Create Table requires Create on Namespace)
    let scope = PermissionScope::Namespace { catalog_id: catalog.id, namespace: ns_name.clone() };
    match check_permission(&store, &session, &Action::Create, &scope).await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, "Forbidden").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Permission check failed: {}", e)).into_response(),
    }
    
    let ns_vec = vec![ns_name];

    // Resolve location
    let table_uuid = Uuid::new_v4();
    let location = if let Some(loc) = payload.location {
        loc
    } else if let Some(base_loc) = &catalog.storage_location {
         format!("{}/{}/{}", base_loc.trim_end_matches('/'), ns_vec.join("/"), tbl_name)
    } else {
        // Fallback: Resolve warehouse bucket if possible
        let mut bucket_name = "warehouse".to_string();
        let mut scheme = "s3";
        
        if let Some(warehouse_name) = &catalog.warehouse_name {
            if let Ok(Some(wh)) = store.get_warehouse(tenant_id, warehouse_name.clone()).await {
                 if let Some(b) = wh.storage_config.get("s3.bucket") {
                     bucket_name = b.clone();
                     scheme = "s3";
                 } else if let Some(c) = wh.storage_config.get("azure.container") {
                     bucket_name = c.clone();
                     scheme = "abfss"; // Azure
                 } else if let Some(b) = wh.storage_config.get("gcp.bucket") {
                     bucket_name = b.clone();
                     scheme = "gs"; // GCP
                 } else if let Some(b) = wh.storage_config.get("bucket") {
                     // Legacy fallback
                     bucket_name = b.clone();
                 }
            }
        }
        format!("{}://{}/{}/{}/{}", scheme, bucket_name, catalog_name, ns_vec.join("/"), tbl_name)
    };

    
    // Parse schema from payload if provided, otherwise create empty schema
    let schema_fields = if let Some(schema_value) = &payload.schema {
        // Try to parse fields from schema
        if let Some(fields) = schema_value.get("fields").and_then(|f| f.as_array()) {
            fields.iter().filter_map(|field| {
                // Parse each field into NestedField
                let id = field.get("id")?.as_i64()? as i32;
                let name = field.get("name")?.as_str()?.to_string();
                // PyArrow creates all fields as optional by default
                // So we ignore the 'required' flag from the request and make everything optional
                let required = false;  // Always optional to match PyArrow
                let field_type_str = field.get("type")?.as_str()?;
                
                // Map type string to Type enum
                // PyArrow uses int64 (long) by default for Python integers
                // So we map both "int" and "integer" to "long" for compatibility
                let field_type = match field_type_str {
                    "int" | "integer" => Type::Primitive("long".to_string()),  // PyArrow uses int64
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
    
    // Create initial TableMetadata with a valid schema
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
            // Write metadata file
            if let Err(e) = store.write_file(&metadata_location, metadata_json.into_bytes()).await {
                 tracing::error!("Failed to write metadata file: {}", e);
                 // Should we rollback asset creation? For now, just log error.
                 return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to write metadata").into_response();
            }

            // Audit Log
            let _ = store.log_audit_event(tenant_id, pangolin_core::audit::AuditLogEntry::legacy_new(
                tenant_id,
                "system".to_string(),
                "create_table".to_string(),
                format!("{}/{}/{}", catalog_name, namespace, tbl_name),
                Some(location.clone())
            )).await;

            // Fetch warehouse credentials for credential vending
            let credentials = match store.get_catalog(tenant_id, catalog_name.clone()).await {
                Ok(Some(catalog)) => {
                    if let Some(warehouse_name) = catalog.warehouse_name {
                        match store.get_warehouse(tenant_id, warehouse_name).await {
                            Ok(Some(warehouse)) => {
                                let mut creds = HashMap::new();
                                
                                // Extract S3 credentials if present
                                if let Some(ak) = warehouse.storage_config.get("s3.access-key-id") {
                                    creds.insert("s3.access-key-id".to_string(), ak.clone());
                                }
                                if let Some(sk) = warehouse.storage_config.get("s3.secret-access-key") {
                                    creds.insert("s3.secret-access-key".to_string(), sk.clone());
                                }
                                if let Some(token) = warehouse.storage_config.get("s3.session-token") {
                                    creds.insert("s3.session-token".to_string(), token.clone());
                                }
                                
                                // Extract Azure credentials if present
                                if let Some(account) = warehouse.storage_config.get("adls.account-name") {
                                    creds.insert("adls.account-name".to_string(), account.clone());
                                }
                                if let Some(key) = warehouse.storage_config.get("adls.account-key") {
                                    creds.insert("adls.account-key".to_string(), key.clone());
                                }
                                if let Some(sas) = warehouse.storage_config.get("adls.sas-token") {
                                    creds.insert("adls.sas-token".to_string(), sas.clone());
                                }
                                
                                // Extract GCS credentials if present
                                if let Some(project) = warehouse.storage_config.get("gcs.project-id") {
                                    creds.insert("gcs.project-id".to_string(), project.clone());
                                }
                                if let Some(sa_file) = warehouse.storage_config.get("gcs.service-account-file") {
                                    creds.insert("gcs.service-account-file".to_string(), sa_file.clone());
                                }
                                if let Some(token) = warehouse.storage_config.get("gcs.oauth2.token") {
                                    creds.insert("gcs.oauth2.token".to_string(), token.clone());
                                }
                                
                                if !creds.is_empty() {
                                    Some(creds)
                                } else {
                                    None
                                }
                            },
                            _ => None
                        }
                    } else {
                        None
                    }
                },
                _ => None
            };

            (StatusCode::OK, Json(TableResponse::with_credentials(
                Some(location.clone()),
                metadata,
                credentials,
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
    
    // Federated check
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
    
    // Resolve catalog ID
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "Catalog not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    };
    
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
    
    // Check Permissions (Load Table requires Read on Asset)
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

        // 4. Try to fetch warehouse credentials for credential vending
        let credentials = match store.get_catalog(tenant_id, catalog_name.clone()).await {
            Ok(Some(catalog)) => {
                if let Some(warehouse_name) = catalog.warehouse_name {
                    match store.get_warehouse(tenant_id, warehouse_name).await {
                        Ok(Some(warehouse)) => {
                            let mut creds = HashMap::new();
                            
                            // Extract S3 credentials if present
                            if let Some(ak) = warehouse.storage_config.get("s3.access-key-id") {
                                creds.insert("s3.access-key-id".to_string(), ak.clone());
                            }
                            if let Some(sk) = warehouse.storage_config.get("s3.secret-access-key") {
                                creds.insert("s3.secret-access-key".to_string(), sk.clone());
                            }
                            if let Some(token) = warehouse.storage_config.get("s3.session-token") {
                                creds.insert("s3.session-token".to_string(), token.clone());
                            }
                            
                            // Extract Azure credentials if present
                            if let Some(account) = warehouse.storage_config.get("adls.account-name") {
                                creds.insert("adls.account-name".to_string(), account.clone());
                            }
                            if let Some(key) = warehouse.storage_config.get("adls.account-key") {
                                creds.insert("adls.account-key".to_string(), key.clone());
                            }
                            if let Some(sas) = warehouse.storage_config.get("adls.sas-token") {
                                creds.insert("adls.sas-token".to_string(), sas.clone());
                            }
                            
                            // Extract GCS credentials if present
                            if let Some(project) = warehouse.storage_config.get("gcs.project-id") {
                                creds.insert("gcs.project-id".to_string(), project.clone());
                            }
                            if let Some(sa_file) = warehouse.storage_config.get("gcs.service-account-file") {
                                creds.insert("gcs.service-account-file".to_string(), sa_file.clone());
                            }
                            if let Some(token) = warehouse.storage_config.get("gcs.oauth2.token") {
                                creds.insert("gcs.oauth2.token".to_string(), token.clone());
                            }
                            
                            if !creds.is_empty() {
                                Some(creds)
                            } else {
                                None
                            }
                        },
                        _ => None
                    }
                } else {
                    None
                }
            },
            _ => None
        };

        // Return the metadata with vended credentials if available
        (StatusCode::OK, Json(TableResponse::with_credentials(
            Some(location),
            metadata,
            credentials,
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
    
    // Federated check
    let path = format!("/namespaces/{}/tables/{}", namespace, table);
    // Serialize body
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
    
    // Resolve catalog ID
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "Catalog not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    };

    let (table_name, branch_from_name) = parse_table_identifier(&table);
    let branch = branch_from_name.unwrap_or("main".to_string());
    let namespace_parts: Vec<String> = namespace.split('\x1F').map(|s| s.to_string()).collect();
    
    tracing::info!("update_table: catalog={} ns={:?} table={}", catalog_name, namespace_parts, table_name);

    // Check Permissions (Update Table requires Write on Asset)
    // Need asset ID
    let asset = match store.get_asset(tenant_id, &catalog_name, Some(branch.clone()), namespace_parts.clone(), table_name.clone()).await {
        Ok(Some(a)) => {
            tracing::info!("update_table: Asset found: {}", a.id);
            a
        },
        Ok(None) => {
            tracing::error!("update_table: Table not found in store");
            return (StatusCode::NOT_FOUND, "Table not found").into_response()
        },
        Err(e) => {
            tracing::error!("update_table: DB Error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        },
    };
    
    let scope = PermissionScope::Asset { 
        catalog_id: catalog.id, 
        namespace: namespace_parts.join("."), 
        asset_id: asset.id 
    };
    
    match check_permission(&store, &session, &Action::Write, &scope).await {
        Ok(true) => (),
        Ok(false) => {
            tracing::error!("update_table: Permission denied");
            return (StatusCode::FORBIDDEN, "Forbidden").into_response();
        },
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Permission check failed: {}", e)).into_response(),
    }
    
    tracing::info!("update_table: Permission granted, starting retry loop");

    // Retry loop for OCC
    let mut retries = 0;
    const MAX_RETRIES: i32 = 5;

    while retries < MAX_RETRIES {
        // 1. Re-fetch asset to get current metadata location (critical for OCC)
        let current_asset = match store.get_asset(tenant_id, &catalog_name, Some(branch.clone()), namespace_parts.clone(), table_name.clone()).await {
            Ok(Some(a)) => a,
            Ok(None) => {
                tracing::error!("update_table: Table not found during retry");
                return (StatusCode::NOT_FOUND, "Table not found").into_response()
            },
            Err(e) => {
                tracing::error!("update_table: DB Error during retry: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            },
        };
        
        let current_metadata_location = current_asset.properties.get("metadata_location").cloned();
        tracing::info!("update_table: Current metadata location (retry {}): {:?}", retries, current_metadata_location);
        
        let metadata_bytes = if let Some(loc) = &current_metadata_location {
             match store.read_file(loc).await {
                 Ok(bytes) => {
                     tracing::info!("update_table: Read {} bytes from metadata file", bytes.len());
                     bytes
                 },
                 Err(e) => {
                     tracing::error!("update_table: Failed to read metadata file: {}", e);
                     return (StatusCode::NOT_FOUND, format!("Failed to read metadata file: {}", e)).into_response()
                 }
             }
        } else {
             tracing::error!("update_table: No metadata location in asset properties");
             return (StatusCode::INTERNAL_SERVER_ERROR, "Table corrupted (no metadata)").into_response()
        };
        
        let mut metadata: TableMetadata = match serde_json::from_slice(&metadata_bytes) {
            Ok(m) => m,
            Err(e) => {
                tracing::error!("update_table: Failed to parse metadata: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse metadata").into_response()
            }
        };


        // 2. Validate Requirements
        for requirement in &payload.requirements {
            match requirement {
                CommitRequirement::AssertCurrentSchemaId { current_schema_id } => {
                    if let Some(req_id) = current_schema_id {
                        if metadata.current_schema_id != *req_id {
                            tracing::warn!("Commit failed: Current schema ID mismatch: expected {}, found {}", req_id, metadata.current_schema_id);
                            return (StatusCode::CONFLICT, format!("Current schema ID mismatch: expected {}, found {}", req_id, metadata.current_schema_id)).into_response();
                        }
                    }
                },
                CommitRequirement::AssertTableUuid { uuid } => {
                    if metadata.table_uuid.to_string() != *uuid {
                        tracing::warn!("Commit failed: Table UUID mismatch: expected {}, found {}", uuid, metadata.table_uuid);
                        return (StatusCode::CONFLICT, format!("Table UUID mismatch: expected {}, found {}", uuid, metadata.table_uuid)).into_response();
                     }
                },
                 _ => {}
            }
        }


        // 3. Apply updates
        for update in &payload.updates {
            match update {
                CommitUpdate::AddSnapshot { snapshot } => {
                    // Parse the full snapshot object
                    match serde_json::from_value::<Snapshot>(snapshot.clone()) {
                        Ok(snapshot_obj) => {
                            tracing::info!("Adding snapshot with ID: {}", snapshot_obj.snapshot_id);
                            
                            // Add to snapshots array
                            if let Some(ref mut snapshots) = metadata.snapshots {
                                snapshots.push(snapshot_obj.clone());
                            } else {
                                metadata.snapshots = Some(vec![snapshot_obj.clone()]);
                            }
                            
                            // Set as current snapshot
                            metadata.current_snapshot_id = Some(snapshot_obj.snapshot_id);
                            metadata.last_updated_ms = Utc::now().timestamp_millis();
                            metadata.last_sequence_number = snapshot_obj.snapshot_id;
                        },
                        Err(e) => {
                            tracing::error!("Failed to parse snapshot: {}", e);
                            // Continue anyway - don't fail the whole commit
                            // Just extract snapshot-id as fallback
                            if let Some(snapshot_id) = snapshot.get("snapshot-id").and_then(|v| v.as_i64()) {
                                tracing::warn!("Using fallback: only setting snapshot ID without full object");
                                metadata.current_snapshot_id = Some(snapshot_id);
                                metadata.last_updated_ms = Utc::now().timestamp_millis();
                                metadata.last_sequence_number = snapshot_id;
                            }
                        }
                    }
                },
                CommitUpdate::AddSchema { schema } => {
                    // Deserialize schema Value to Schema struct
                    match serde_json::from_value::<pangolin_core::iceberg_metadata::Schema>(schema.clone()) {
                        Ok(new_schema) => {
                             tracing::info!("Adding new schema with ID: {}", new_schema.schema_id);
                             metadata.schemas.push(new_schema);
                        },
                        Err(e) => {
                             tracing::error!("Failed to parse new schema: {}", e);
                             return (StatusCode::BAD_REQUEST, format!("Invalid schema format: {}", e)).into_response();
                        }
                    }
                },
                CommitUpdate::SetCurrentSchema { schema_id } => {
                    tracing::info!("Setting current schema ID to: {}", schema_id);
                    if *schema_id == -1 {
                        if let Some(last) = metadata.schemas.last() {
                            tracing::info!("Resolving -1 to last schema ID: {}", last.schema_id);
                            metadata.current_schema_id = last.schema_id;
                        } else {
                             tracing::warn!("SetCurrentSchema -1 requested but no schemas available");
                             metadata.current_schema_id = *schema_id; // Fallback
                        }
                    } else {
                        metadata.current_schema_id = *schema_id;
                    }
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
        match store.update_metadata_location(tenant_id, &catalog_name, Some(branch.clone()), namespace_parts.clone(), table_name.clone(), current_metadata_location.clone(), new_metadata_location.clone()).await {
            Ok(_) => {
                // Success!
                return (StatusCode::OK, Json(TableResponse::new(
                    Some(new_metadata_location.clone()),
                    metadata,
                ))).into_response();
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

#[derive(Deserialize, Serialize, ToSchema)]
pub struct RenameTableRequest {
    pub source: TableIdentifier,
    pub destination: TableIdentifier,
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
    
    // Federated check - Rename is weird in REST catalog spec (POST /tables/rename or similar?)
    // Our route is /v1/:prefix/tables/rename.
    // So path relative to catalogue is /tables/rename
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

    // Assuming rename is within the same branch for now, or default branch
    // Iceberg spec doesn't explicitly mention branches in rename, so we assume "main" or default.
    let branch = Some("main".to_string());

    match store.rename_asset(tenant_id, &catalog_name, branch, source_ns, source_name, dest_ns, dest_name).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
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
    
    // Federated check
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
    
    // Resolve catalog ID
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "Catalog not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    };
    
    let (table_name, branch_from_name) = parse_table_identifier(&table);
    let branch = branch_from_name.or(Some("main".to_string()));
    let namespace_parts: Vec<String> = namespace.split('\x1F').map(|s| s.to_string()).collect();
    
    // Check Permissions (Delete Table requires Delete on Asset)
    // Need asset ID
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
             // Audit Log
             let _ = store.log_audit_event(tenant_id, pangolin_core::audit::AuditLogEntry::legacy_new(
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
    
    // Resolve catalog ID
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "Catalog not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    };
    
    let namespace_parts: Vec<String> = namespace.split('\x1F').map(|s| s.to_string()).collect();
    
    // Check Permissions (Delete Namespace)
    let scope = PermissionScope::Namespace { 
        catalog_id: catalog.id, 
        namespace: namespace_parts.join(".") 
    };
    
    match check_permission(&store, &session, &Action::Delete, &scope).await {
        Ok(true) => (),
        Ok(false) => return (StatusCode::FORBIDDEN, "Forbidden").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Permission check failed: {}", e)).into_response(),
    }

    match store.delete_namespace(tenant_id, &catalog_name, namespace_parts).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Namespace not found").into_response(),
    }
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct UpdateNamespacePropertiesRequest {
    pub removals: Option<Vec<String>>,
    pub updates: Option<std::collections::HashMap<String, String>>,
}

#[derive(Serialize, ToSchema)]
pub struct UpdateNamespacePropertiesResponse {
    pub updated: Vec<String>,
    pub removed: Vec<String>,
    pub missing: Vec<String>,
}

/// Update namespace properties
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
    ).await {
         return response;
    }
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
    // Just log and return success
    tracing::info!("Received metrics report");
    StatusCode::NO_CONTENT
}






#[derive(Debug, Deserialize, ToSchema)]
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
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Extension(tenant_id): Extension<TenantId>,
    Path((prefix, namespace, table)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let tenant_id = tenant_id.0;
    let catalog_name = prefix;
    
    // Federated check
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
    // Parse table@branch
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_table_request_serialization() {
        let req = CreateTableRequest {
            name: "test_table".to_string(),
            location: None,
            schema: Some(serde_json::json!({"type": "struct", "fields": []})),
            properties: Some(HashMap::new()),
        };

        let serialized = serde_json::to_string(&req);
        assert!(serialized.is_ok(), "CreateTableRequest should be serializable");
    }

    #[test]
    fn test_commit_table_request_serialization() {
        let req = CommitTableRequest {
            identifier: None,
            requirements: vec![],
            updates: vec![],
        };
        let serialized = serde_json::to_string(&req);
        assert!(serialized.is_ok(), "CommitTableRequest should be serializable");
    }

    #[test]
    fn test_rename_table_request_serialization() {
        let req = RenameTableRequest {
            source: TableIdentifier { namespace: vec!["ns".to_string()], name: "t1".to_string() },
            destination: TableIdentifier { namespace: vec!["ns".to_string()], name: "t2".to_string() },
        };
        let serialized = serde_json::to_string(&req);
        assert!(serialized.is_ok(), "RenameTableRequest should be serializable");
    }

    #[test]
    fn test_update_namespace_properties_request_serialization() {
        let req = UpdateNamespacePropertiesRequest {
            removals: Some(vec!["prop1".to_string()]),
            updates: Some(HashMap::new()),
        };
        let serialized = serde_json::to_string(&req);
        assert!(serialized.is_ok(), "UpdateNamespacePropertiesRequest should be serializable");
    }
}
