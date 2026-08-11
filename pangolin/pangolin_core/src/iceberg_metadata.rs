use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;
use uuid::Uuid;

/// Partition field ids are assigned from 1000 upward by the Iceberg spec, so an
/// unpartitioned table's highest *assigned* partition id is 999.
pub const PARTITION_FIELD_ID_START: i32 = 1000;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct TableMetadata {
    pub format_version: i32,
    pub table_uuid: Uuid,
    pub location: String,
    pub last_sequence_number: i64,
    pub last_updated_ms: i64,
    pub last_column_id: i32,
    pub current_schema_id: i32,
    pub schemas: Vec<Schema>,
    /// The default partition spec id.
    ///
    /// The spec field is `default-spec-id`. Under the struct's kebab-case rule
    /// this serialized as `current-partition-spec-id` (B11), which no
    /// spec-conformant reader looks for - metadata Pangolin wrote could not be
    /// read as v2 metadata by an external engine reading the file directly, and
    /// a conformant engine's metadata could not round-trip in. The alias keeps
    /// already-written Pangolin files parseable.
    #[serde(rename = "default-spec-id", alias = "current-partition-spec-id")]
    pub current_partition_spec_id: i32,
    pub partition_specs: Vec<PartitionSpec>,
    /// Highest assigned partition field id.
    ///
    /// Required by the v2 spec and missing entirely before (B12); Java-based
    /// readers reject metadata without it. Defaulted on read so files Pangolin
    /// wrote earlier still parse, and recomputed from `partition_specs` by
    /// [`TableMetadata::recompute_last_partition_id`].
    #[serde(default = "default_last_partition_id")]
    pub last_partition_id: i32,
    pub default_sort_order_id: i32,
    pub sort_orders: Vec<SortOrder>,
    pub properties: Option<HashMap<String, String>>,
    pub current_snapshot_id: Option<i64>,
    pub snapshots: Option<Vec<Snapshot>>,
    pub snapshot_log: Option<Vec<SnapshotLogEntry>>,
    pub metadata_log: Option<Vec<MetadataLogEntry>>,
    /// Named branches and tags, keyed by ref name.
    ///
    /// Required by the Iceberg spec for `set-snapshot-ref` and
    /// `assert-ref-snapshot-id`. `main` conventionally tracks
    /// `current_snapshot_id`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refs: Option<HashMap<String, SnapshotReference>>,
}

fn default_last_partition_id() -> i32 {
    PARTITION_FIELD_ID_START - 1
}

impl TableMetadata {
    /// Recompute `last_partition_id` from the partition specs.
    ///
    /// Called after any change to `partition_specs` so the field stays true;
    /// the spec defines it as the highest partition field id ever assigned, so
    /// it only ever moves up.
    pub fn recompute_last_partition_id(&mut self) {
        let highest = self
            .partition_specs
            .iter()
            .flat_map(|spec| spec.fields.iter())
            .map(|f| f.field_id)
            .max()
            .unwrap_or(PARTITION_FIELD_ID_START - 1);
        self.last_partition_id = self.last_partition_id.max(highest);
    }
}

/// A named branch or tag pointing at a snapshot.
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct SnapshotReference {
    pub snapshot_id: i64,
    /// `"branch"` or `"tag"`.
    #[serde(rename = "type")]
    pub ref_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_snapshots_to_keep: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_snapshot_age_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_ref_age_ms: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct Schema {
    /// Always `"struct"`.
    ///
    /// Spec schemas *are* struct types and conformant writers emit this; it was
    /// missing entirely (B14), so strict readers rejected the schema object.
    #[serde(rename = "type", default = "struct_type_name")]
    pub type_: String,
    /// Defaulted because a `createTable` request body may legitimately omit it -
    /// the server assigns the id. Without the default, deserializing an incoming
    /// schema (which `create_table` now does instead of hand-parsing it, B16f)
    /// would reject a spec-legal request.
    #[serde(default)]
    pub schema_id: i32,
    /// Omitted when absent rather than written as an explicit `null` (B14) -
    /// some strict parsers reject `"identifier-field-ids": null`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identifier_field_ids: Option<Vec<i32>>,
    pub fields: Vec<NestedField>,
}

fn struct_type_name() -> String {
    Schema::STRUCT.to_string()
}

impl Schema {
    /// The only legal value of a schema's `type` field.
    pub const STRUCT: &'static str = "struct";

    /// Highest field id in this schema, including nested fields.
    ///
    /// `last-column-id` must cover nested ids too; computing it from the
    /// top-level fields alone (as `create_table` used to) understates it for any
    /// schema containing a struct, list or map.
    pub fn max_field_id(&self) -> i32 {
        fn walk(t: &Type, acc: &mut i32) {
            match t {
                Type::Primitive(_) => {}
                Type::Struct { fields, .. } => {
                    for f in fields {
                        *acc = (*acc).max(f.id);
                        walk(&f.field_type, acc);
                    }
                }
                Type::List {
                    element_id,
                    element,
                    ..
                } => {
                    *acc = (*acc).max(*element_id);
                    walk(element, acc);
                }
                Type::Map {
                    key_id,
                    key,
                    value_id,
                    value,
                    ..
                } => {
                    *acc = (*acc).max(*key_id).max(*value_id);
                    walk(key, acc);
                    walk(value, acc);
                }
            }
        }

        let mut max = 0;
        for field in &self.fields {
            max = max.max(field.id);
            walk(&field.field_type, &mut max);
        }
        max
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct NestedField {
    pub id: i32,
    pub name: String,
    pub required: bool,
    #[serde(rename = "type")]
    pub field_type: Type,
    /// Omitted when absent rather than serialized as `"doc": null` (B14).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(untagged)]
pub enum Type {
    Primitive(String),
    Struct {
        #[serde(rename = "type")]
        type_name: String, // "struct"
        fields: Vec<NestedField>,
    },
    List {
        #[serde(rename = "type")]
        type_name: String, // "list"
        element_id: i32,
        element_required: bool,
        element: Box<Type>,
    },
    Map {
        #[serde(rename = "type")]
        type_name: String, // "map"
        key_id: i32,
        key: Box<Type>,
        value_id: i32,
        value_required: bool,
        value: Box<Type>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct PartitionSpec {
    pub spec_id: i32,
    pub fields: Vec<PartitionField>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct PartitionField {
    pub source_id: i32,
    pub field_id: i32,
    pub name: String,
    pub transform: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct SortOrder {
    pub order_id: i32,
    pub fields: Vec<SortField>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct SortField {
    pub source_id: i32,
    pub transform: String,
    pub direction: String,
    pub null_order: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct Snapshot {
    pub snapshot_id: i64,
    pub parent_snapshot_id: Option<i64>,
    pub sequence_number: i64,
    pub timestamp_ms: i64,
    pub manifest_list: String,
    pub summary: HashMap<String, String>,
    pub schema_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct SnapshotLogEntry {
    pub timestamp_ms: i64,
    pub snapshot_id: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct MetadataLogEntry {
    pub timestamp_ms: i64,
    pub metadata_file: String,
}
