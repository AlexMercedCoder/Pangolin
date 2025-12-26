use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, IntoParams};
use std::collections::HashMap;
use pangolin_core::iceberg_metadata::{TableMetadata, Snapshot};

#[derive(Serialize, ToSchema)]
pub struct CatalogConfig {
    pub defaults: HashMap<String, String>,
    pub overrides: HashMap<String, String>,
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
    /// Internal Pangolin asset ID for linking to business metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<uuid::Uuid>,
    #[serde(rename = "metadata-location")]
    pub metadata_location: Option<String>,
    pub metadata: TableMetadata,
    // Config tells PyIceberg how to access the table's data
    // Including credential vending configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<HashMap<String, String>>,
}

impl TableResponse {
    pub fn new(metadata_location: Option<String>, metadata: TableMetadata, asset_id: Option<uuid::Uuid>) -> Self {
        Self::with_credentials(metadata_location, metadata, None, asset_id)
    }
    
    pub fn with_credentials(
        metadata_location: Option<String>, 
        metadata: TableMetadata,
        credentials: Option<HashMap<String, String>>,
        asset_id: Option<uuid::Uuid>,
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
            id: asset_id,
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

#[derive(Deserialize, Serialize, ToSchema)]
pub struct RenameTableRequest {
    pub source: TableIdentifier,
    pub destination: TableIdentifier,
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
