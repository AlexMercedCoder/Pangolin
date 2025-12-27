use super::MemoryStore;
use anyhow::Result;
use uuid::Uuid;
use pangolin_core::model::*;
use async_trait::async_trait;

impl MemoryStore {
    pub(crate) async fn create_merge_operation_internal(&self, operation: pangolin_core::model::MergeOperation) -> Result<()> {
            self.merge_operations.insert(operation.id, operation);
            Ok(())
        }
    pub(crate) async fn get_merge_operation_internal(&self, operation_id: Uuid) -> Result<Option<pangolin_core::model::MergeOperation>> {
            Ok(self.merge_operations.get(&operation_id).map(|r| r.value().clone()))
        }
    pub(crate) async fn list_merge_operations_internal(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<pangolin_core::model::MergeOperation>> {
            Ok(self.merge_operations
                .iter()
                .filter(|r| r.value().tenant_id == tenant_id && r.value().catalog_name == catalog_name)
                .map(|r| r.value().clone())
                .collect())
        }
    
    pub(crate) async fn complete_merge_operation_internal(&self, operation_id: Uuid, result_commit_id: Uuid) -> Result<()> {
        if let Some(mut op) = self.merge_operations.get_mut(&operation_id) {
            op.status = MergeStatus::Completed;
            op.result_commit_id = Some(result_commit_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Merge operation not found"))
        }
    }

    pub(crate) async fn abort_merge_operation_internal(&self, operation_id: Uuid) -> Result<()> {
        if let Some(mut op) = self.merge_operations.get_mut(&operation_id) {
            op.status = MergeStatus::Aborted;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Merge operation not found"))
        }
    }

    pub(crate) async fn get_merge_conflict_internal(&self, conflict_id: Uuid) -> Result<Option<MergeConflict>> {
        Ok(self.merge_conflicts.get(&conflict_id).map(|c| c.value().clone()))
    }

    pub(crate) async fn list_merge_conflicts_internal(&self, operation_id: Uuid) -> Result<Vec<MergeConflict>> {
        let conflicts = self.merge_conflicts.iter()
            .filter(|c| c.value().merge_operation_id == operation_id)
            .map(|c| c.value().clone())
            .collect();
        Ok(conflicts)
    }
}
