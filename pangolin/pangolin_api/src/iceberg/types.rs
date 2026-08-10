use pangolin_core::iceberg_metadata::TableMetadata;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, ToSchema)]
pub struct CatalogConfig {
    pub defaults: HashMap<String, String>,
    pub overrides: HashMap<String, String>,
}

#[derive(Serialize, ToSchema)]
pub struct ListNamespacesResponse {
    pub namespaces: Vec<Vec<String>>,
    /// Continuation token; absent on the final page (B16i).
    #[serde(rename = "next-page-token", skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct CreateTableRequest {
    pub name: String,
    pub location: Option<String>,
    pub schema: Option<serde_json::Value>, // Accept schema as JSON
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
    pub fn new(
        metadata_location: Option<String>,
        metadata: TableMetadata,
        asset_id: Option<uuid::Uuid>,
    ) -> Self {
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
        config.entry("s3.endpoint".to_string()).or_insert_with(|| {
            std::env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string())
        });
        config.entry("s3.region".to_string()).or_insert_with(|| {
            std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string())
        });

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
    /// Continuation token; absent on the final page (B16i).
    #[serde(
        rename = "next-page-token",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub next_page_token: Option<String>,
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
#[serde(deny_unknown_fields)]
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
    #[serde(rename = "assert-last-assigned-field-id")]
    AssertLastAssignedFieldId {
        #[serde(rename = "last-assigned-field-id")]
        last_assigned_field_id: i32,
    },
    #[serde(rename = "assert-default-spec-id")]
    AssertDefaultSpecId {
        #[serde(rename = "default-spec-id")]
        default_spec_id: i32,
    },
    #[serde(rename = "assert-last-assigned-partition-id")]
    AssertLastAssignedPartitionId {
        #[serde(rename = "last-assigned-partition-id")]
        last_assigned_partition_id: i32,
    },
    #[serde(rename = "assert-default-sort-order-id")]
    AssertDefaultSortOrderId {
        #[serde(rename = "default-sort-order-id")]
        default_sort_order_id: i32,
    },
    /// A requirement type this server does not implement.
    ///
    /// Modelled explicitly rather than swallowed by a catch-all match arm: an
    /// unrecognised requirement must fail the commit, because the client is
    /// asserting a precondition the server cannot check (A-1).
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(tag = "action")]
pub enum CommitUpdate {
    #[serde(rename = "assign-uuid")]
    AssignUuid { uuid: String },
    #[serde(rename = "upgrade-format-version")]
    UpgradeFormatVersion {
        #[serde(rename = "format-version")]
        format_version: i32,
    },
    #[serde(rename = "add-schema")]
    AddSchema { schema: serde_json::Value },
    #[serde(rename = "set-current-schema")]
    SetCurrentSchema {
        #[serde(rename = "schema-id")]
        schema_id: i32,
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
    SetProperties { updates: HashMap<String, String> },
    #[serde(rename = "remove-properties")]
    RemoveProperties { removals: Vec<String> },
    #[serde(rename = "set-location")]
    SetLocation { location: String },
    #[serde(rename = "add-spec")]
    AddSpec { spec: serde_json::Value },
    #[serde(rename = "set-default-spec")]
    SetDefaultSpec {
        #[serde(rename = "spec-id")]
        spec_id: i32,
    },
    #[serde(rename = "add-sort-order")]
    AddSortOrder {
        #[serde(rename = "sort-order")]
        sort_order: serde_json::Value,
    },
    #[serde(rename = "set-default-sort-order")]
    SetDefaultSortOrder {
        #[serde(rename = "sort-order-id")]
        sort_order_id: i32,
    },
    #[serde(rename = "remove-snapshots")]
    RemoveSnapshots {
        #[serde(rename = "snapshot-ids")]
        snapshot_ids: Vec<i64>,
    },
    #[serde(rename = "remove-snapshot-ref")]
    RemoveSnapshotRef {
        #[serde(rename = "ref-name")]
        ref_name: String,
    },
    /// Any update type this server does not know about.
    ///
    /// The previous `_ => {}` arm meant a client running
    /// `ALTER TABLE ... SET TBLPROPERTIES`, evolving a partition spec or
    /// expiring snapshots received `200 OK` for an operation that never
    /// happened (A-2). Unknown updates are now rejected.
    #[serde(other)]
    Unknown,
}

// Helper to parse "table@branch"
pub fn parse_table_identifier(identifier: &str) -> (String, Option<String>) {
    if let Some((name, branch)) = identifier.split_once('@') {
        (name.to_string(), Some(branch.to_string()))
    } else {
        (identifier.to_string(), None)
    }
}

/// The unit separator the Iceberg REST spec uses to encode a multi-level
/// namespace inside a single path segment.
pub const NAMESPACE_SEPARATOR: char = '\u{1F}';

/// Parse a namespace path segment into its levels plus an optional branch.
///
/// This is the single parser for namespace path segments (B16a). Handlers used
/// to disagree: `list_tables`/`create_table`/`load_table` went through
/// [`parse_table_identifier`], which yields a *single-element* namespace, while
/// `update_table`/`delete_table`/`table_exists` split on `0x1F` and yielded the
/// real multi-element path. So a table created in namespace `a\x1Fb` was
/// registered under `["a\x1Fb"]` but looked up under `["a", "b"]` on commit -
/// a guaranteed `404 Table not found`, with the CAS loop never running.
///
/// The `@branch` suffix is stripped first so a `ns@branch` form works
/// everywhere, not just on the handlers that happened to call
/// `parse_table_identifier`.
pub fn parse_namespace(namespace: &str) -> (Vec<String>, Option<String>) {
    let (path, branch) = match namespace.split_once('@') {
        Some((path, branch)) if !branch.is_empty() => (path, Some(branch.to_string())),
        _ => (namespace, None),
    };

    let levels: Vec<String> = path
        .split(NAMESPACE_SEPARATOR)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    (levels, branch)
}

/// Spec-shaped pagination query parameters.
///
/// The Iceberg REST spec paginates with `pageToken`/`pageSize`; Pangolin only
/// understood `limit`/`offset`, so a spec client's paging parameters were
/// silently ignored and it had no way to detect a truncated listing (B16i).
/// Both spellings are accepted, with the spec ones taking precedence.
#[derive(Deserialize, IntoParams, Default)]
pub struct IcebergPageParams {
    #[serde(rename = "pageToken")]
    pub page_token: Option<String>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<u32>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Default page size when a client asks for pagination without naming one.
pub const DEFAULT_PAGE_SIZE: u32 = 100;

impl IcebergPageParams {
    /// Resolve to `(offset, limit)`.
    ///
    /// The page token is an opaque encoding of the offset, per the spec's
    /// "clients must treat the token as opaque" rule; the encoding here is just
    /// a prefixed decimal so it stays debuggable, and an unparseable token
    /// degrades to offset 0 rather than erroring the listing.
    pub fn resolve(&self) -> (u32, u32) {
        let limit = self
            .page_size
            .or(self.limit)
            .filter(|l| *l > 0)
            .unwrap_or(DEFAULT_PAGE_SIZE);

        let offset = self
            .page_token
            .as_deref()
            .and_then(decode_page_token)
            .or(self.offset)
            .unwrap_or(0);

        (offset, limit)
    }
}

/// Encode an offset as an opaque continuation token.
pub fn encode_page_token(offset: u32) -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    URL_SAFE_NO_PAD.encode(format!("o:{}", offset))
}

/// Decode a continuation token produced by [`encode_page_token`].
pub fn decode_page_token(token: &str) -> Option<u32> {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    let decoded = URL_SAFE_NO_PAD.decode(token).ok()?;
    let decoded = String::from_utf8(decoded).ok()?;
    decoded.strip_prefix("o:")?.parse().ok()
}

/// Compute the `next-page-token` for a listing.
///
/// Returns `None` when the page came back short, which is how a client knows it
/// has reached the end.
pub fn next_page_token(returned: usize, offset: u32, limit: u32) -> Option<String> {
    if returned as u32 == limit {
        Some(encode_page_token(offset + limit))
    } else {
        None
    }
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct RenameTableRequest {
    pub source: TableIdentifier,
    pub destination: TableIdentifier,
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(deny_unknown_fields)]
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
