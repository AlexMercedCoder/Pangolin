//! Applying Iceberg commit requirements and updates to table metadata.
//!
//! This is the correctness core of the table-commit path, split out of
//! `tables.rs` so it can be tested without a store, a router, or a network.
//!
//! Three defects lived here.
//!
//! **A-1 — commit requirements were silently discarded.** Only
//! `assert-current-schema-id` and `assert-table-uuid` were implemented; the
//! match ended in `_ => {}`. The one that matters most for concurrency is
//! `assert-ref-snapshot-id` — how a writer says *"only commit if the branch
//! still points where I think it does."* Because it was never checked, when
//! writer B's compare-and-swap failed after writer A committed, B retried
//! against A's new metadata and blindly re-applied its own `AddSnapshot`.
//! Nothing rejected the stale commit, producing a snapshot whose
//! `parent_snapshot_id` pointed at a pre-A snapshot spliced onto a branch that
//! had moved on: forked lineage and orphaned data files, with no error ever
//! surfaced.
//!
//! **A-2 — most updates were ignored behind `200 OK`.** Everything except
//! `add-snapshot`, `add-schema` and `set-current-schema` was dropped, and the
//! handler still returned success with a fresh metadata file. Silent success is
//! worse than failure: it cannot be retried or alerted on.
//!
//! **A-3 — `last_sequence_number` was assigned a snapshot ID.** In the spec
//! these are unrelated: the sequence number is a small monotonic counter, the
//! snapshot ID is a random 64-bit value. Sequence numbers govern how
//! position/equality delete files apply to data files, so corrupting them
//! produces incorrect query results on merge-on-read tables.

use std::collections::HashMap;

use chrono::Utc;
use pangolin_core::iceberg_metadata::{
    Snapshot, SnapshotLogEntry, SnapshotReference, TableMetadata,
};

use super::types::{CommitRequirement, CommitUpdate};

/// The reference name used when a commit does not target a named branch.
pub const MAIN_REF: &str = "main";

/// Why a commit was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitError {
    /// A requirement did not hold: the caller's view of the table is stale.
    /// Maps to `409 Conflict` and `CommitFailedException`.
    RequirementFailed { requirement: String, detail: String },
    /// The request asked for something this server cannot do. Maps to
    /// `501 Not Implemented` rather than a false `200 OK`.
    Unsupported { operation: String },
    /// The request was structurally wrong. Maps to `400 Bad Request`.
    Invalid { detail: String },
}

impl std::fmt::Display for CommitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommitError::RequirementFailed {
                requirement,
                detail,
            } => write!(f, "requirement {requirement} failed: {detail}"),
            CommitError::Unsupported { operation } => {
                write!(f, "unsupported commit operation: {operation}")
            }
            CommitError::Invalid { detail } => write!(f, "invalid commit request: {detail}"),
        }
    }
}

/// The snapshot a named ref currently points at.
///
/// `main` falls back to `current_snapshot_id`, since a table written before
/// refs were tracked has no explicit `refs` map but still has a current
/// snapshot.
pub fn ref_snapshot_id(metadata: &TableMetadata, reference: &str) -> Option<i64> {
    if let Some(refs) = &metadata.refs {
        if let Some(r) = refs.get(reference) {
            return Some(r.snapshot_id);
        }
    }
    if reference == MAIN_REF {
        return metadata.current_snapshot_id;
    }
    None
}

/// Check every commit requirement against the current metadata.
///
/// Every requirement is evaluated; an unknown requirement is an error rather
/// than a no-op, because the client is asserting a precondition this server
/// cannot verify.
pub fn check_requirements(
    metadata: &TableMetadata,
    requirements: &[CommitRequirement],
    table_exists: bool,
) -> Result<(), CommitError> {
    for requirement in requirements {
        match requirement {
            CommitRequirement::AssertCreate => {
                if table_exists {
                    return Err(CommitError::RequirementFailed {
                        requirement: "assert-create".into(),
                        detail: "table already exists".into(),
                    });
                }
            }

            CommitRequirement::AssertTableUuid { uuid } => {
                if metadata.table_uuid.to_string() != *uuid {
                    return Err(CommitError::RequirementFailed {
                        requirement: "assert-table-uuid".into(),
                        detail: format!("expected {uuid}, found {}", metadata.table_uuid),
                    });
                }
            }

            // The one that makes optimistic concurrency control work.
            CommitRequirement::AssertRefSnapshotId {
                reference,
                snapshot_id,
            } => {
                let actual = ref_snapshot_id(metadata, reference);
                match (snapshot_id, actual) {
                    // Caller asserts the ref does not exist yet.
                    (None, None) => {}
                    (None, Some(actual)) => {
                        return Err(CommitError::RequirementFailed {
                            requirement: "assert-ref-snapshot-id".into(),
                            detail: format!(
                                "ref {reference:?} was expected not to exist but points at {actual}"
                            ),
                        })
                    }
                    (Some(expected), None) => {
                        return Err(CommitError::RequirementFailed {
                            requirement: "assert-ref-snapshot-id".into(),
                            detail: format!(
                                "ref {reference:?} does not exist, expected {expected}"
                            ),
                        })
                    }
                    (Some(expected), Some(actual)) => {
                        if expected != &actual {
                            return Err(CommitError::RequirementFailed {
                                requirement: "assert-ref-snapshot-id".into(),
                                detail: format!(
                                    "ref {reference:?} points at {actual}, expected {expected}"
                                ),
                            });
                        }
                    }
                }
            }

            CommitRequirement::AssertCurrentSchemaId { current_schema_id } => {
                if let Some(expected) = current_schema_id {
                    if metadata.current_schema_id != *expected {
                        return Err(CommitError::RequirementFailed {
                            requirement: "assert-current-schema-id".into(),
                            detail: format!(
                                "current schema is {}, expected {expected}",
                                metadata.current_schema_id
                            ),
                        });
                    }
                }
            }

            CommitRequirement::AssertDefaultSpecId { default_spec_id } => {
                if metadata.current_partition_spec_id != *default_spec_id {
                    return Err(CommitError::RequirementFailed {
                        requirement: "assert-default-spec-id".into(),
                        detail: format!(
                            "default spec is {}, expected {default_spec_id}",
                            metadata.current_partition_spec_id
                        ),
                    });
                }
            }

            CommitRequirement::AssertDefaultSortOrderId {
                default_sort_order_id,
            } => {
                if metadata.default_sort_order_id != *default_sort_order_id {
                    return Err(CommitError::RequirementFailed {
                        requirement: "assert-default-sort-order-id".into(),
                        detail: format!(
                            "default sort order is {}, expected {default_sort_order_id}",
                            metadata.default_sort_order_id
                        ),
                    });
                }
            }

            CommitRequirement::AssertLastAssignedFieldId {
                last_assigned_field_id,
            } => {
                if metadata.last_column_id != *last_assigned_field_id {
                    return Err(CommitError::RequirementFailed {
                        requirement: "assert-last-assigned-field-id".into(),
                        detail: format!(
                            "last assigned field id is {}, expected {last_assigned_field_id}",
                            metadata.last_column_id
                        ),
                    });
                }
            }

            // Now checkable: `last-partition-id` is a real field on
            // `TableMetadata` (B12). Before it existed this requirement had to
            // be refused, because the alternative was pretending a precondition
            // held when it had never been examined.
            CommitRequirement::AssertLastAssignedPartitionId {
                last_assigned_partition_id,
            } => {
                if metadata.last_partition_id != *last_assigned_partition_id {
                    return Err(CommitError::RequirementFailed {
                        requirement: "assert-last-assigned-partition-id".into(),
                        detail: format!(
                            "last-partition-id is {}, expected {last_assigned_partition_id}",
                            metadata.last_partition_id
                        ),
                    });
                }
            }

            CommitRequirement::Unknown => {
                return Err(CommitError::Unsupported {
                    operation: "unrecognised commit requirement".into(),
                })
            }
        }
    }
    Ok(())
}

/// Apply every update to `metadata` in order.
///
/// On error the caller must discard `metadata`: updates are applied in place
/// and a partially-applied set must never be published.
pub fn apply_updates(
    metadata: &mut TableMetadata,
    updates: &[CommitUpdate],
    branch: &str,
) -> Result<(), CommitError> {
    // B16c: `-1` in `set-current-schema` / `set-default-spec` /
    // `set-default-sort-order` means "the one added *by this commit*". The old
    // code resolved it against `metadata.schemas.last()` etc., which for an
    // existing table is never empty - so a `-1` sent *without* a preceding
    // `add-schema` silently repointed the table at whatever happened to be last
    // in the persisted vector (arbitrary if the vector is not in creation
    // order), instead of hitting the error arm the message claims. Tracking
    // what this commit actually added makes the sentinel mean what it says.
    let mut last_added_schema_id: Option<i32> = None;
    let mut last_added_spec_id: Option<i32> = None;
    let mut last_added_sort_order_id: Option<i32> = None;

    for update in updates {
        match update {
            CommitUpdate::AssignUuid { uuid } => {
                metadata.table_uuid = uuid.parse().map_err(|_| CommitError::Invalid {
                    detail: format!("assign-uuid: {uuid:?} is not a UUID"),
                })?;
            }

            CommitUpdate::UpgradeFormatVersion { format_version } => {
                if *format_version < metadata.format_version {
                    return Err(CommitError::Invalid {
                        detail: format!(
                            "upgrade-format-version cannot downgrade {} to {format_version}",
                            metadata.format_version
                        ),
                    });
                }
                if *format_version > 2 {
                    return Err(CommitError::Unsupported {
                        operation: format!("table format version {format_version}"),
                    });
                }
                metadata.format_version = *format_version;
            }

            CommitUpdate::AddSchema { schema } => {
                let new_schema: pangolin_core::iceberg_metadata::Schema =
                    serde_json::from_value(schema.clone()).map_err(|e| CommitError::Invalid {
                        detail: format!("add-schema: {e}"),
                    })?;
                // Reject a duplicate id rather than pushing a second schema the
                // rest of the metadata cannot tell apart (B16c).
                if metadata
                    .schemas
                    .iter()
                    .any(|s| s.schema_id == new_schema.schema_id)
                {
                    return Err(CommitError::Invalid {
                        detail: format!(
                            "add-schema: schema id {} already exists",
                            new_schema.schema_id
                        ),
                    });
                }
                // `max_field_id` walks nested fields; taking the max over the
                // top-level `fields` alone understates `last-column-id` for any
                // schema containing a struct, list or map.
                metadata.last_column_id = metadata.last_column_id.max(new_schema.max_field_id());
                last_added_schema_id = Some(new_schema.schema_id);
                metadata.schemas.push(new_schema);
            }

            CommitUpdate::SetCurrentSchema { schema_id } => {
                // -1 means "the schema added by this same commit".
                if *schema_id == -1 {
                    match last_added_schema_id {
                        Some(id) => metadata.current_schema_id = id,
                        None => {
                            return Err(CommitError::Invalid {
                                detail: "set-current-schema: -1 with no schema in this commit"
                                    .into(),
                            })
                        }
                    }
                } else {
                    if !metadata.schemas.iter().any(|s| s.schema_id == *schema_id) {
                        return Err(CommitError::Invalid {
                            detail: format!("set-current-schema: no schema with id {schema_id}"),
                        });
                    }
                    metadata.current_schema_id = *schema_id;
                }
            }

            CommitUpdate::AddSnapshot { snapshot } => {
                let snapshot_obj: Snapshot =
                    serde_json::from_value(snapshot.clone()).map_err(|e| CommitError::Invalid {
                        detail: format!("add-snapshot: {e}"),
                    })?;
                add_snapshot(metadata, snapshot_obj, branch)?;
            }

            CommitUpdate::SetSnapshotRef {
                ref_name,
                snapshot_id,
                ref_type,
            } => {
                let known = metadata
                    .snapshots
                    .as_ref()
                    .is_some_and(|s| s.iter().any(|snap| snap.snapshot_id == *snapshot_id));
                if !known {
                    return Err(CommitError::Invalid {
                        detail: format!("set-snapshot-ref: unknown snapshot {snapshot_id}"),
                    });
                }
                metadata.refs.get_or_insert_with(HashMap::new).insert(
                    ref_name.clone(),
                    SnapshotReference {
                        snapshot_id: *snapshot_id,
                        ref_type: ref_type.clone(),
                        min_snapshots_to_keep: None,
                        max_snapshot_age_ms: None,
                        max_ref_age_ms: None,
                    },
                );
                if ref_name == MAIN_REF {
                    metadata.current_snapshot_id = Some(*snapshot_id);
                }
            }

            CommitUpdate::RemoveSnapshotRef { ref_name } => {
                if ref_name == MAIN_REF {
                    return Err(CommitError::Invalid {
                        detail: "remove-snapshot-ref: the main branch cannot be removed".into(),
                    });
                }
                if let Some(refs) = metadata.refs.as_mut() {
                    refs.remove(ref_name);
                }
            }

            CommitUpdate::SetProperties { updates } => {
                let props = metadata.properties.get_or_insert_with(HashMap::new);
                for (k, v) in updates {
                    props.insert(k.clone(), v.clone());
                }
            }

            CommitUpdate::RemoveProperties { removals } => {
                if let Some(props) = metadata.properties.as_mut() {
                    for key in removals {
                        props.remove(key);
                    }
                }
            }

            CommitUpdate::SetLocation { location } => {
                if location.trim().is_empty() {
                    return Err(CommitError::Invalid {
                        detail: "set-location: location must not be empty".into(),
                    });
                }
                metadata.location = location.clone();
            }

            CommitUpdate::AddSpec { spec } => {
                let new_spec: pangolin_core::iceberg_metadata::PartitionSpec =
                    serde_json::from_value(spec.clone()).map_err(|e| CommitError::Invalid {
                        detail: format!("add-spec: {e}"),
                    })?;
                if metadata
                    .partition_specs
                    .iter()
                    .any(|s| s.spec_id == new_spec.spec_id)
                {
                    return Err(CommitError::Invalid {
                        detail: format!("add-spec: spec id {} already exists", new_spec.spec_id),
                    });
                }
                last_added_spec_id = Some(new_spec.spec_id);
                metadata.partition_specs.push(new_spec);
                // Keep the required `last-partition-id` true (B12).
                metadata.recompute_last_partition_id();
            }

            CommitUpdate::SetDefaultSpec { spec_id } => {
                let target = if *spec_id == -1 {
                    last_added_spec_id.ok_or_else(|| CommitError::Invalid {
                        detail: "set-default-spec: -1 with no spec in this commit".into(),
                    })?
                } else {
                    *spec_id
                };
                if !metadata.partition_specs.iter().any(|s| s.spec_id == target) {
                    return Err(CommitError::Invalid {
                        detail: format!("set-default-spec: no spec with id {target}"),
                    });
                }
                metadata.current_partition_spec_id = target;
            }

            CommitUpdate::AddSortOrder { sort_order } => {
                let new_order: pangolin_core::iceberg_metadata::SortOrder =
                    serde_json::from_value(sort_order.clone()).map_err(|e| {
                        CommitError::Invalid {
                            detail: format!("add-sort-order: {e}"),
                        }
                    })?;
                if metadata
                    .sort_orders
                    .iter()
                    .any(|o| o.order_id == new_order.order_id)
                {
                    return Err(CommitError::Invalid {
                        detail: format!(
                            "add-sort-order: sort order id {} already exists",
                            new_order.order_id
                        ),
                    });
                }
                last_added_sort_order_id = Some(new_order.order_id);
                metadata.sort_orders.push(new_order);
            }

            CommitUpdate::SetDefaultSortOrder { sort_order_id } => {
                let target = if *sort_order_id == -1 {
                    last_added_sort_order_id.ok_or_else(|| CommitError::Invalid {
                        detail: "set-default-sort-order: -1 with no sort order in this commit"
                            .into(),
                    })?
                } else {
                    *sort_order_id
                };
                if !metadata.sort_orders.iter().any(|o| o.order_id == target) {
                    return Err(CommitError::Invalid {
                        detail: format!("set-default-sort-order: no sort order with id {target}"),
                    });
                }
                metadata.default_sort_order_id = target;
            }

            CommitUpdate::RemoveSnapshots { snapshot_ids } => {
                // Refuse to strip a snapshot a live ref points at, which would
                // leave the table unreadable from that branch or tag.
                if let Some(refs) = &metadata.refs {
                    for (name, r) in refs {
                        if snapshot_ids.contains(&r.snapshot_id) {
                            return Err(CommitError::Invalid {
                                detail: format!(
                                    "remove-snapshots: {} is still referenced by {name:?}",
                                    r.snapshot_id
                                ),
                            });
                        }
                    }
                }
                if let Some(current) = metadata.current_snapshot_id {
                    if snapshot_ids.contains(&current) {
                        return Err(CommitError::Invalid {
                            detail: format!("remove-snapshots: {current} is the current snapshot"),
                        });
                    }
                }
                if let Some(snapshots) = metadata.snapshots.as_mut() {
                    snapshots.retain(|s| !snapshot_ids.contains(&s.snapshot_id));
                }
                if let Some(log) = metadata.snapshot_log.as_mut() {
                    log.retain(|e| !snapshot_ids.contains(&e.snapshot_id));
                }
            }

            CommitUpdate::Unknown => {
                return Err(CommitError::Unsupported {
                    operation: "unrecognised commit update".into(),
                })
            }
        }
    }
    Ok(())
}

/// Append a snapshot and move the branch to it.
fn add_snapshot(
    metadata: &mut TableMetadata,
    snapshot: Snapshot,
    branch: &str,
) -> Result<(), CommitError> {
    let snapshot_id = snapshot.snapshot_id;
    let timestamp_ms = if snapshot.timestamp_ms > 0 {
        snapshot.timestamp_ms
    } else {
        Utc::now().timestamp_millis()
    };

    // The sequence number is a monotonic counter, *not* a snapshot ID (A-3).
    //
    // B15: the old rule was "honour any client value greater than the current
    // counter". A client could therefore submit `i64::MAX`, after which the next
    // commit computed `last_sequence_number + 1` and overflowed - a panic in
    // debug builds, a wrap in release, and corrupt commit ordering either way.
    // The counter is the server's to advance: a client value is accepted only if
    // it is exactly the next one, and anything else is assigned rather than
    // honoured.
    let next_sequence = metadata
        .last_sequence_number
        .checked_add(1)
        .ok_or_else(|| CommitError::Invalid {
            detail: "add-snapshot: sequence number counter is exhausted".into(),
        })?;
    let sequence_number = if snapshot.sequence_number == next_sequence {
        snapshot.sequence_number
    } else {
        next_sequence
    };

    let mut snapshot = snapshot;
    snapshot.sequence_number = sequence_number;
    snapshot.timestamp_ms = timestamp_ms;

    metadata
        .snapshots
        .get_or_insert_with(Vec::new)
        .push(snapshot);
    metadata.last_sequence_number = sequence_number;
    metadata.last_updated_ms = timestamp_ms;

    // B16: `current_snapshot_id` and the `snapshot_log` describe *main*. Setting
    // them for any branch meant a `dev`-branch commit changed what `main`
    // readers resolve if the metadata document was ever shared across branches
    // or exported. Same for fabricating a `main` ref pointing at the branch's
    // snapshot: that silently published unreviewed work to main.
    if branch == MAIN_REF {
        metadata.current_snapshot_id = Some(snapshot_id);
        metadata
            .snapshot_log
            .get_or_insert_with(Vec::new)
            .push(SnapshotLogEntry {
                timestamp_ms,
                snapshot_id,
            });
    }

    let refs = metadata.refs.get_or_insert_with(HashMap::new);
    refs.insert(
        branch.to_string(),
        SnapshotReference {
            snapshot_id,
            ref_type: "branch".to_string(),
            min_snapshots_to_keep: None,
            max_snapshot_age_ms: None,
            max_ref_age_ms: None,
        },
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pangolin_core::iceberg_metadata::{NestedField, Schema, Type};
    use uuid::Uuid;

    fn base_metadata() -> TableMetadata {
        TableMetadata {
            format_version: 2,
            table_uuid: Uuid::nil(),
            location: "s3://bucket/table".to_string(),
            last_sequence_number: 0,
            last_updated_ms: 0,
            last_column_id: 1,
            current_schema_id: 0,
            schemas: vec![Schema {
                type_: "struct".to_string(),
                schema_id: 0,
                identifier_field_ids: None,
                fields: vec![NestedField {
                    id: 1,
                    name: "id".to_string(),
                    required: true,
                    field_type: Type::Primitive("long".to_string()),
                    doc: None,
                }],
            }],
            current_partition_spec_id: 0,
            partition_specs: vec![],
            last_partition_id: 999,
            default_sort_order_id: 0,
            sort_orders: vec![],
            properties: None,
            current_snapshot_id: None,
            snapshots: None,
            snapshot_log: None,
            metadata_log: None,
            refs: None,
        }
    }

    fn snapshot_json(id: i64, parent: Option<i64>) -> serde_json::Value {
        serde_json::json!({
            "snapshot-id": id,
            "parent-snapshot-id": parent,
            "sequence-number": 0,
            "timestamp-ms": 1_700_000_000_000i64,
            "manifest-list": format!("s3://bucket/table/metadata/snap-{id}.avro"),
            "summary": { "operation": "append" },
            "schema-id": 0,
        })
    }

    // -- A-1: assert-ref-snapshot-id -------------------------------------

    #[test]
    fn assert_ref_snapshot_id_passes_when_the_branch_is_where_the_writer_expects() {
        let mut metadata = base_metadata();
        apply_updates(
            &mut metadata,
            &[CommitUpdate::AddSnapshot {
                snapshot: snapshot_json(111, None),
            }],
            MAIN_REF,
        )
        .unwrap();

        let requirement = CommitRequirement::AssertRefSnapshotId {
            reference: MAIN_REF.to_string(),
            snapshot_id: Some(111),
        };
        assert_eq!(check_requirements(&metadata, &[requirement], true), Ok(()));
    }

    /// The exact lost-update scenario from the audit: writer B prepared its
    /// commit against snapshot 111, writer A committed 222 first, and B's
    /// retry must now be rejected rather than forking the lineage.
    #[test]
    fn assert_ref_snapshot_id_rejects_a_stale_writer() {
        let mut metadata = base_metadata();
        apply_updates(
            &mut metadata,
            &[CommitUpdate::AddSnapshot {
                snapshot: snapshot_json(111, None),
            }],
            MAIN_REF,
        )
        .unwrap();
        // Writer A commits.
        apply_updates(
            &mut metadata,
            &[CommitUpdate::AddSnapshot {
                snapshot: snapshot_json(222, Some(111)),
            }],
            MAIN_REF,
        )
        .unwrap();

        // Writer B still believes main points at 111.
        let stale = CommitRequirement::AssertRefSnapshotId {
            reference: MAIN_REF.to_string(),
            snapshot_id: Some(111),
        };
        let err = check_requirements(&metadata, &[stale], true).unwrap_err();
        assert!(
            matches!(err, CommitError::RequirementFailed { ref requirement, .. } if requirement == "assert-ref-snapshot-id"),
            "expected a requirement failure, got {err:?}"
        );
    }

    #[test]
    fn assert_ref_snapshot_id_handles_missing_and_unexpected_refs() {
        let metadata = base_metadata();
        // Ref does not exist and the writer expects one.
        assert!(check_requirements(
            &metadata,
            &[CommitRequirement::AssertRefSnapshotId {
                reference: "feature".into(),
                snapshot_id: Some(1),
            }],
            true
        )
        .is_err());
        // Ref does not exist and the writer expects that.
        assert_eq!(
            check_requirements(
                &metadata,
                &[CommitRequirement::AssertRefSnapshotId {
                    reference: "feature".into(),
                    snapshot_id: None,
                }],
                true
            ),
            Ok(())
        );
    }

    #[test]
    fn unknown_requirements_are_refused_rather_than_ignored() {
        let metadata = base_metadata();
        let err = check_requirements(&metadata, &[CommitRequirement::Unknown], true).unwrap_err();
        assert!(matches!(err, CommitError::Unsupported { .. }));
    }

    #[test]
    fn assert_create_fails_for_an_existing_table() {
        let metadata = base_metadata();
        assert!(check_requirements(&metadata, &[CommitRequirement::AssertCreate], true).is_err());
        assert_eq!(
            check_requirements(&metadata, &[CommitRequirement::AssertCreate], false),
            Ok(())
        );
    }

    #[test]
    fn other_requirements_are_actually_checked() {
        let metadata = base_metadata();
        assert!(check_requirements(
            &metadata,
            &[CommitRequirement::AssertCurrentSchemaId {
                current_schema_id: Some(9)
            }],
            true
        )
        .is_err());
        assert!(check_requirements(
            &metadata,
            &[CommitRequirement::AssertDefaultSpecId { default_spec_id: 7 }],
            true
        )
        .is_err());
        assert!(check_requirements(
            &metadata,
            &[CommitRequirement::AssertLastAssignedFieldId {
                last_assigned_field_id: 99
            }],
            true
        )
        .is_err());
        assert!(check_requirements(
            &metadata,
            &[CommitRequirement::AssertTableUuid {
                uuid: Uuid::new_v4().to_string()
            }],
            true
        )
        .is_err());
    }

    // -- A-2: updates are applied or refused, never silently dropped -----

    #[test]
    fn set_properties_is_applied() {
        let mut metadata = base_metadata();
        apply_updates(
            &mut metadata,
            &[CommitUpdate::SetProperties {
                updates: HashMap::from([(
                    "write.format.default".to_string(),
                    "parquet".to_string(),
                )]),
            }],
            MAIN_REF,
        )
        .unwrap();
        assert_eq!(
            metadata
                .properties
                .as_ref()
                .unwrap()
                .get("write.format.default"),
            Some(&"parquet".to_string())
        );
    }

    #[test]
    fn remove_properties_is_applied() {
        let mut metadata = base_metadata();
        metadata.properties = Some(HashMap::from([("a".to_string(), "1".to_string())]));
        apply_updates(
            &mut metadata,
            &[CommitUpdate::RemoveProperties {
                removals: vec!["a".to_string()],
            }],
            MAIN_REF,
        )
        .unwrap();
        assert!(metadata.properties.as_ref().unwrap().is_empty());
    }

    #[test]
    fn set_location_is_applied_and_validated() {
        let mut metadata = base_metadata();
        apply_updates(
            &mut metadata,
            &[CommitUpdate::SetLocation {
                location: "s3://other/table".to_string(),
            }],
            MAIN_REF,
        )
        .unwrap();
        assert_eq!(metadata.location, "s3://other/table");

        assert!(apply_updates(
            &mut metadata,
            &[CommitUpdate::SetLocation {
                location: "  ".to_string()
            }],
            MAIN_REF
        )
        .is_err());
    }

    #[test]
    fn set_snapshot_ref_creates_a_tag_and_rejects_unknown_snapshots() {
        let mut metadata = base_metadata();
        apply_updates(
            &mut metadata,
            &[CommitUpdate::AddSnapshot {
                snapshot: snapshot_json(555, None),
            }],
            MAIN_REF,
        )
        .unwrap();
        apply_updates(
            &mut metadata,
            &[CommitUpdate::SetSnapshotRef {
                ref_name: "release-1".to_string(),
                snapshot_id: 555,
                ref_type: "tag".to_string(),
            }],
            MAIN_REF,
        )
        .unwrap();
        assert_eq!(ref_snapshot_id(&metadata, "release-1"), Some(555));

        assert!(apply_updates(
            &mut metadata,
            &[CommitUpdate::SetSnapshotRef {
                ref_name: "bogus".to_string(),
                snapshot_id: 999,
                ref_type: "tag".to_string(),
            }],
            MAIN_REF
        )
        .is_err());
    }

    #[test]
    fn remove_snapshots_refuses_to_orphan_a_live_ref() {
        let mut metadata = base_metadata();
        apply_updates(
            &mut metadata,
            &[CommitUpdate::AddSnapshot {
                snapshot: snapshot_json(1, None),
            }],
            MAIN_REF,
        )
        .unwrap();
        let err = apply_updates(
            &mut metadata,
            &[CommitUpdate::RemoveSnapshots {
                snapshot_ids: vec![1],
            }],
            MAIN_REF,
        )
        .unwrap_err();
        assert!(matches!(err, CommitError::Invalid { .. }));
    }

    #[test]
    fn remove_snapshots_drops_an_unreferenced_snapshot() {
        let mut metadata = base_metadata();
        apply_updates(
            &mut metadata,
            &[
                CommitUpdate::AddSnapshot {
                    snapshot: snapshot_json(1, None),
                },
                CommitUpdate::AddSnapshot {
                    snapshot: snapshot_json(2, Some(1)),
                },
            ],
            MAIN_REF,
        )
        .unwrap();
        apply_updates(
            &mut metadata,
            &[CommitUpdate::RemoveSnapshots {
                snapshot_ids: vec![1],
            }],
            MAIN_REF,
        )
        .unwrap();
        assert_eq!(metadata.snapshots.as_ref().unwrap().len(), 1);
    }

    /// Regression test for A-2: the shape that used to return `200 OK` having
    /// done nothing at all.
    #[test]
    fn unknown_updates_are_refused_rather_than_silently_dropped() {
        let mut metadata = base_metadata();
        let err = apply_updates(&mut metadata, &[CommitUpdate::Unknown], MAIN_REF).unwrap_err();
        assert!(matches!(err, CommitError::Unsupported { .. }));
    }

    #[test]
    fn an_unrecognised_update_action_deserialises_to_unknown() {
        let parsed: CommitUpdate =
            serde_json::from_value(serde_json::json!({ "action": "some-future-action" })).unwrap();
        assert!(matches!(parsed, CommitUpdate::Unknown));
    }

    #[test]
    fn set_current_schema_rejects_an_unknown_schema_id() {
        let mut metadata = base_metadata();
        assert!(apply_updates(
            &mut metadata,
            &[CommitUpdate::SetCurrentSchema { schema_id: 42 }],
            MAIN_REF
        )
        .is_err());
    }

    // -- A-3: sequence numbers ------------------------------------------

    #[test]
    fn sequence_numbers_are_a_monotonic_counter_not_snapshot_ids() {
        let mut metadata = base_metadata();
        // Snapshot IDs are large random values; sequence numbers must not
        // follow them.
        for (i, id) in [7_446_468_398_231_234_000i64, 1_223_344_556_677_889_900]
            .iter()
            .enumerate()
        {
            apply_updates(
                &mut metadata,
                &[CommitUpdate::AddSnapshot {
                    snapshot: snapshot_json(*id, None),
                }],
                MAIN_REF,
            )
            .unwrap();
            assert_eq!(
                metadata.last_sequence_number,
                (i + 1) as i64,
                "sequence number must increment by one per commit"
            );
        }
        let snapshots = metadata.snapshots.as_ref().unwrap();
        assert_eq!(snapshots[0].sequence_number, 1);
        assert_eq!(snapshots[1].sequence_number, 2);
    }

    /// B15: the counter is the server's. A client value is honoured only when it
    /// is exactly the next one; anything else is replaced by the next value
    /// rather than letting the client jump the counter to, say, `i64::MAX` and
    /// overflow the *following* commit.
    #[test]
    fn a_client_supplied_sequence_number_cannot_jump_the_counter() {
        let mut metadata = base_metadata();
        let mut snap = snapshot_json(1, None);
        snap["sequence-number"] = serde_json::json!(5);
        apply_updates(
            &mut metadata,
            &[CommitUpdate::AddSnapshot { snapshot: snap }],
            MAIN_REF,
        )
        .unwrap();
        assert_eq!(metadata.last_sequence_number, 1);

        // The exact next value is accepted.
        let mut snap = snapshot_json(2, None);
        snap["sequence-number"] = serde_json::json!(2);
        apply_updates(
            &mut metadata,
            &[CommitUpdate::AddSnapshot { snapshot: snap }],
            MAIN_REF,
        )
        .unwrap();
        assert_eq!(metadata.last_sequence_number, 2);
    }

    /// B15 regression: `i64::MAX` used to be honoured verbatim, so the next
    /// commit's `last_sequence_number + 1` overflowed.
    #[test]
    fn an_absurd_client_sequence_number_does_not_overflow_the_next_commit() {
        let mut metadata = base_metadata();
        let mut snap = snapshot_json(1, None);
        snap["sequence-number"] = serde_json::json!(i64::MAX);
        apply_updates(
            &mut metadata,
            &[CommitUpdate::AddSnapshot { snapshot: snap }],
            MAIN_REF,
        )
        .unwrap();
        assert_eq!(metadata.last_sequence_number, 1);

        // The follow-up commit is ordinary arithmetic, not an overflow.
        apply_updates(
            &mut metadata,
            &[CommitUpdate::AddSnapshot {
                snapshot: snapshot_json(2, None),
            }],
            MAIN_REF,
        )
        .unwrap();
        assert_eq!(metadata.last_sequence_number, 2);
    }

    #[test]
    fn adding_a_snapshot_on_main_records_the_snapshot_log_and_moves_the_branch() {
        let mut metadata = base_metadata();
        apply_updates(
            &mut metadata,
            &[CommitUpdate::AddSnapshot {
                snapshot: snapshot_json(99, None),
            }],
            MAIN_REF,
        )
        .unwrap();
        assert_eq!(metadata.current_snapshot_id, Some(99));
        assert_eq!(ref_snapshot_id(&metadata, MAIN_REF), Some(99));
        assert_eq!(metadata.snapshot_log.as_ref().unwrap().len(), 1);
    }

    /// B16: a commit to a feature branch must move *only* that branch. It used
    /// to set `current_snapshot_id` for any branch and fabricate a `main` ref
    /// pointing at the branch's snapshot, so a `dev` commit changed what `main`
    /// readers resolve.
    #[test]
    fn committing_to_a_feature_branch_leaves_main_alone() {
        let mut metadata = base_metadata();
        let main_before = ref_snapshot_id(&metadata, MAIN_REF);

        apply_updates(
            &mut metadata,
            &[CommitUpdate::AddSnapshot {
                snapshot: snapshot_json(99, None),
            }],
            "feature",
        )
        .unwrap();

        assert_eq!(ref_snapshot_id(&metadata, "feature"), Some(99));
        assert_eq!(
            ref_snapshot_id(&metadata, MAIN_REF),
            main_before,
            "a feature-branch commit must not move main"
        );
        assert_eq!(
            metadata.current_snapshot_id, main_before,
            "current-snapshot-id describes main, not the committed branch"
        );
        assert!(
            metadata
                .snapshot_log
                .as_ref()
                .is_none_or(|log| log.is_empty()),
            "the snapshot log describes main's history"
        );
    }

    #[test]
    fn metadata_round_trips_through_json_with_refs() {
        let mut metadata = base_metadata();
        apply_updates(
            &mut metadata,
            &[CommitUpdate::AddSnapshot {
                snapshot: snapshot_json(3, None),
            }],
            MAIN_REF,
        )
        .unwrap();
        let json = serde_json::to_string(&metadata).unwrap();
        let back: TableMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(back.refs, metadata.refs);
        assert_eq!(back.last_sequence_number, metadata.last_sequence_number);
    }
}
