use pangolin_api::iceberg_handlers::TableResponse;
use pangolin_core::iceberg_metadata::{Snapshot, TableMetadata};
use serde_json::json;

#[test]
fn test_add_snapshot_deserializes_full_object() {
    // Test that AddSnapshot properly deserializes and stores the full snapshot object
    let snapshot_json = json!({
        "snapshot-id": 123456789,
        "parent-snapshot-id": null,
        "sequence-number": 1,
        "timestamp-ms": 1702483200000i64,
        "manifest-list": "s3://bucket/table/metadata/snap-123456789-1-abc.avro",
        "summary": {
            "operation": "append"
        }
    });
    
    // Create initial metadata with empty snapshots
    let mut metadata = TableMetadata {
        format_version: 2,
        table_uuid: uuid::Uuid::new_v4(),
        location: "s3://bucket/table".to_string(),
        last_sequence_number: 0,
        last_updated_ms: 1702483200000,
        last_column_id: 0,
        schemas: vec![],
        current_schema_id: 0,
        current_partition_spec_id: 0,
        partition_specs: vec![],
        default_sort_order_id: 0,
        sort_orders: vec![],
        properties: None,
        current_snapshot_id: None,
        snapshots: Some(vec![]),
        snapshot_log: Some(vec![]),
        metadata_log: Some(vec![]),
    };
    
    // Simulate the AddSnapshot update
    match serde_json::from_value::<Snapshot>(snapshot_json.clone()) {
        Ok(snapshot_obj) => {
            // Add to snapshots array
            if let Some(ref mut snapshots) = metadata.snapshots {
                snapshots.push(snapshot_obj.clone());
            }
            
            // Set as current snapshot
            metadata.current_snapshot_id = Some(snapshot_obj.snapshot_id);
            
            // Verify the snapshot was added
            assert_eq!(metadata.snapshots.as_ref().unwrap().len(), 1);
            assert_eq!(metadata.current_snapshot_id, Some(123456789));
            assert_eq!(metadata.snapshots.as_ref().unwrap()[0].snapshot_id, 123456789);
        },
        Err(e) => {
            panic!("Failed to deserialize snapshot: {}", e);
        }
    }
}

#[test]
fn test_add_snapshot_handles_multiple_snapshots() {
    // Test that multiple snapshots can be added and tracked
    let mut metadata = TableMetadata {
        format_version: 2,
        table_uuid: uuid::Uuid::new_v4(),
        location: "s3://bucket/table".to_string(),
        last_sequence_number: 0,
        last_updated_ms: 1702483200000,
        last_column_id: 0,
        schemas: vec![],
        current_schema_id: 0,
        current_partition_spec_id: 0,
        partition_specs: vec![],
        default_sort_order_id: 0,
        sort_orders: vec![],
        properties: None,
        current_snapshot_id: None,
        snapshots: Some(vec![]),
        snapshot_log: Some(vec![]),
        metadata_log: Some(vec![]),
    };
    
    // Add first snapshot
    let snapshot1_json = json!({
        "snapshot-id": 111,
        "parent-snapshot-id": null,
        "sequence-number": 1,
        "timestamp-ms": 1702483200000i64,
        "manifest-list": "s3://bucket/table/metadata/snap-111-1-abc.avro",
        "summary": {"operation": "append"}
    });
    
    let snapshot1: Snapshot = serde_json::from_value(snapshot1_json).unwrap();
    if let Some(ref mut snapshots) = metadata.snapshots {
        snapshots.push(snapshot1.clone());
    }
    metadata.current_snapshot_id = Some(snapshot1.snapshot_id);
    
    // Add second snapshot
    let snapshot2_json = json!({
        "snapshot-id": 222,
        "parent-snapshot-id": 111,
        "sequence-number": 2,
        "timestamp-ms": 1702483300000i64,
        "manifest-list": "s3://bucket/table/metadata/snap-222-2-def.avro",
        "summary": {"operation": "append"}
    });
    
    let snapshot2: Snapshot = serde_json::from_value(snapshot2_json).unwrap();
    if let Some(ref mut snapshots) = metadata.snapshots {
        snapshots.push(snapshot2.clone());
    }
    metadata.current_snapshot_id = Some(snapshot2.snapshot_id);
    
    // Verify both snapshots are tracked
    assert_eq!(metadata.snapshots.as_ref().unwrap().len(), 2);
    assert_eq!(metadata.current_snapshot_id, Some(222));
    assert_eq!(metadata.snapshots.as_ref().unwrap()[0].snapshot_id, 111);
    assert_eq!(metadata.snapshots.as_ref().unwrap()[1].snapshot_id, 222);
}

#[test]
fn test_table_response_includes_config() {
    // Test that TableResponse includes S3 endpoint configuration
    let metadata = TableMetadata {
        format_version: 2,
        table_uuid: uuid::Uuid::new_v4(),
        location: "s3://bucket/table".to_string(),
        last_sequence_number: 0,
        last_updated_ms: 1702483200000,
        last_column_id: 0,
        schemas: vec![],
        current_schema_id: 0,
        current_partition_spec_id: 0,
        partition_specs: vec![],
        default_sort_order_id: 0,
        sort_orders: vec![],
        properties: None,
        current_snapshot_id: None,
        snapshots: Some(vec![]),
        snapshot_log: Some(vec![]),
        metadata_log: Some(vec![]),
    };
    
    let response = TableResponse::new(
        Some("s3://bucket/table/metadata/v1.metadata.json".to_string()),
        metadata
    );
    
    // Verify config is present
    assert!(response.config.is_some());
    let config = response.config.unwrap();
    
    // Verify S3 endpoint is configured
    assert!(config.contains_key("s3.endpoint"));
    assert!(config.contains_key("s3.region"));
}
