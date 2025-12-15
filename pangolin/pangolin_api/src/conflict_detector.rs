use pangolin_core::model::{Asset, ConflictType, MergeConflict, MergeOperation};
use pangolin_store::CatalogStore;
use std::sync::Arc;
use uuid::Uuid;
use anyhow::Result;

/// Conflict detector for analyzing merge operations
pub struct ConflictDetector {
    store: Arc<dyn CatalogStore + Send + Sync>,
}

impl ConflictDetector {
    pub fn new(store: Arc<dyn CatalogStore + Send + Sync>) -> Self {
        Self { store }
    }

    /// Detect all conflicts between source and target branches
    pub async fn detect_conflicts(
        &self,
        operation: &MergeOperation,
    ) -> Result<Vec<MergeConflict>> {
        let mut conflicts = Vec::new();

        // Get assets from both branches
        let source_assets = self.get_branch_assets(
            operation.tenant_id,
            &operation.catalog_name,
            &operation.source_branch,
        ).await?;

        let target_assets = self.get_branch_assets(
            operation.tenant_id,
            &operation.catalog_name,
            &operation.target_branch,
        ).await?;

        // Detect different types of conflicts
        conflicts.extend(self.detect_schema_conflicts(operation, &source_assets, &target_assets).await?);
        conflicts.extend(self.detect_deletion_conflicts(operation, &source_assets, &target_assets).await?);
        conflicts.extend(self.detect_metadata_conflicts(operation, &source_assets, &target_assets).await?);

        Ok(conflicts)
    }

    /// Get all assets for a specific branch
    async fn get_branch_assets(
        &self,
        tenant_id: Uuid,
        catalog_name: &str,
        branch_name: &str,
    ) -> Result<Vec<Asset>> {
        // Get all namespaces in the catalog
        let namespaces = self.store.list_namespaces(tenant_id, catalog_name, None).await?;
        
        let mut all_assets = Vec::new();
        for namespace in namespaces {
            let assets = self.store.list_assets(
                tenant_id,
                catalog_name,
                Some(branch_name.to_string()),
                namespace.name.clone(),
            ).await?;
            all_assets.extend(assets);
        }
        
        Ok(all_assets)
    }

    /// Detect schema conflicts (tables with different schemas in source vs target)
    async fn detect_schema_conflicts(
        &self,
        operation: &MergeOperation,
        source_assets: &[Asset],
        target_assets: &[Asset],
    ) -> Result<Vec<MergeConflict>> {
        let mut conflicts = Vec::new();

        for source_asset in source_assets {
            // Find matching asset in target
            if let Some(target_asset) = target_assets.iter().find(|a| a.name == source_asset.name) {
                // Compare properties to detect schema changes
                // In a real implementation, you'd parse and compare Iceberg schemas
                if source_asset.properties != target_asset.properties {
                    let conflict = MergeConflict::new(
                        operation.id,
                        ConflictType::SchemaChange {
                            asset_name: source_asset.name.clone(),
                            source_schema: serde_json::to_value(&source_asset.properties)?,
                            target_schema: serde_json::to_value(&target_asset.properties)?,
                        },
                        Some(source_asset.id),
                        format!(
                            "Schema conflict detected for asset '{}': properties differ between branches",
                            source_asset.name
                        ),
                    );
                    conflicts.push(conflict);
                }
            }
        }

        Ok(conflicts)
    }

    /// Detect deletion conflicts (asset deleted in one branch, modified in another)
    async fn detect_deletion_conflicts(
        &self,
        operation: &MergeOperation,
        source_assets: &[Asset],
        target_assets: &[Asset],
    ) -> Result<Vec<MergeConflict>> {
        let mut conflicts = Vec::new();

        // Check for assets deleted in source but present in target
        for target_asset in target_assets {
            if !source_assets.iter().any(|a| a.name == target_asset.name) {
                // Asset exists in target but not in source - might be deleted in source
                // This is a potential deletion conflict
                let conflict = MergeConflict::new(
                    operation.id,
                    ConflictType::DeletionConflict {
                        asset_name: target_asset.name.clone(),
                        deleted_in: "source".to_string(),
                        modified_in: "target".to_string(),
                    },
                    Some(target_asset.id),
                    format!(
                        "Deletion conflict: asset '{}' deleted in source branch but exists in target",
                        target_asset.name
                    ),
                );
                conflicts.push(conflict);
            }
        }

        // Check for assets deleted in target but present in source
        // In a true 3-way merge, we would check if it was in Base. 
        // Lacking Base, we assume "Present in Source, Missing in Target" is an ADDITION, not a conflict.
        // So we skip flagging this as a conflict.
        /*
        for source_asset in source_assets {
            if !target_assets.iter().any(|a| a.name == source_asset.name) {
                let conflict = MergeConflict::new(
                    operation.id,
                    ConflictType::DeletionConflict {
                        asset_name: source_asset.name.clone(),
                        deleted_in: "target".to_string(),
                        modified_in: "source".to_string(),
                    },
                    Some(source_asset.id),
                    format!(
                        "Deletion conflict: asset '{}' deleted in target branch but exists in source",
                        source_asset.name
                    ),
                );
                conflicts.push(conflict);
            }
        }
        */

        Ok(conflicts)
    }

    /// Detect metadata conflicts (conflicting properties/metadata)
    async fn detect_metadata_conflicts(
        &self,
        operation: &MergeOperation,
        source_assets: &[Asset],
        target_assets: &[Asset],
    ) -> Result<Vec<MergeConflict>> {
        let mut conflicts = Vec::new();

        for source_asset in source_assets {
            if let Some(target_asset) = target_assets.iter().find(|a| a.name == source_asset.name) {
                // Check for conflicting properties
                let mut conflicting_props = Vec::new();
                
                for (key, source_value) in &source_asset.properties {
                    if let Some(target_value) = target_asset.properties.get(key) {
                        if source_value != target_value {
                            conflicting_props.push(key.clone());
                        }
                    }
                }

                if !conflicting_props.is_empty() {
                    let conflict = MergeConflict::new(
                        operation.id,
                        ConflictType::MetadataConflict {
                            asset_name: source_asset.name.clone(),
                            conflicting_properties: conflicting_props.clone(),
                        },
                        Some(source_asset.id),
                        format!(
                            "Metadata conflict for asset '{}': conflicting properties: {}",
                            source_asset.name,
                            conflicting_props.join(", ")
                        ),
                    );
                    conflicts.push(conflict);
                }
            }
        }

        Ok(conflicts)
    }

    /// Check if conflicts can be auto-resolved
    pub fn can_auto_resolve(&self, conflict: &MergeConflict) -> bool {
        match &conflict.conflict_type {
            // Metadata conflicts with non-critical properties can be auto-resolved
            ConflictType::MetadataConflict { conflicting_properties, .. } => {
                // Only auto-resolve if conflicts are in non-critical properties
                conflicting_properties.iter().all(|prop| {
                    !prop.starts_with("schema") && !prop.starts_with("partition")
                })
            }
            // Schema and deletion conflicts require manual resolution
            ConflictType::SchemaChange { .. } => false,
            ConflictType::DeletionConflict { .. } => false,
            ConflictType::DataOverlap { .. } => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pangolin_core::model::{AssetType, MergeStatus};
    use pangolin_store::MemoryStore;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_detect_no_conflicts() {
        let store = Arc::new(MemoryStore::new());
        let detector = ConflictDetector::new(store.clone());
        
        let tenant_id = Uuid::new_v4();
        let operation = MergeOperation::new(
            tenant_id,
            "test_catalog".to_string(),
            "source".to_string(),
            "target".to_string(),
            None,
            Uuid::new_v4(),
        );

        let conflicts = detector.detect_conflicts(&operation).await.unwrap();
        assert!(conflicts.is_empty());
    }

    #[tokio::test]
    async fn test_can_auto_resolve() {
        let store = Arc::new(MemoryStore::new());
        let detector = ConflictDetector::new(store);
        
        let operation_id = Uuid::new_v4();
        
        // Metadata conflict with non-critical property
        let metadata_conflict = MergeConflict::new(
            operation_id,
            ConflictType::MetadataConflict {
                asset_name: "test_table".to_string(),
                conflicting_properties: vec!["description".to_string()],
            },
            None,
            "Test conflict".to_string(),
        );
        assert!(detector.can_auto_resolve(&metadata_conflict));
        
        // Schema conflict cannot be auto-resolved
        let schema_conflict = MergeConflict::new(
            operation_id,
            ConflictType::SchemaChange {
                asset_name: "test_table".to_string(),
                source_schema: serde_json::json!({}),
                target_schema: serde_json::json!({}),
            },
            None,
            "Test conflict".to_string(),
        );
        assert!(!detector.can_auto_resolve(&schema_conflict));
    }
}
