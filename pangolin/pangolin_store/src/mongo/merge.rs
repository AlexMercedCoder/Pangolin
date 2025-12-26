use super::MongoStore;
use anyhow::Result;
use mongodb::bson::{doc, Document};
use pangolin_core::model::{MergeOperation, MergeConflict, MergeStatus, ConflictResolution};
use uuid::Uuid;
use futures::stream::TryStreamExt;

impl MongoStore {
    pub async fn create_merge_operation(&self, operation: MergeOperation) -> Result<()> {
        let doc = mongodb::bson::to_document(&operation)?;
        self.merge_operations().insert_one(doc).await?;
        Ok(())
    }

    pub async fn get_merge_operation(&self, operation_id: Uuid) -> Result<Option<MergeOperation>> {
        let filter = doc! { "_id": operation_id.to_string() };
        if let Some(doc) = self.merge_operations().find_one(filter).await? {
            Ok(Some(mongodb::bson::from_document(doc)?))
        } else {
            Ok(None)
        }
    }

    pub async fn list_merge_operations(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<MergeOperation>> {
        let filter = doc! {
            "tenant_id": tenant_id.to_string(),
            "catalog_name": catalog_name
        };
        let mut cursor = self.merge_operations().find(filter).await?;
        let mut operations = Vec::new();
        while cursor.advance().await? {
            operations.push(mongodb::bson::from_document(cursor.deserialize_current()?)?);
        }
        Ok(operations)
    }

    pub async fn update_merge_operation(&self, op: MergeOperation) -> Result<()> {
        let filter = doc! { "_id": op.id.to_string() };
        let doc = mongodb::bson::to_document(&op)?;
        self.merge_operations().replace_one(filter, doc).await?;
        Ok(())
    }

    pub async fn delete_merge_operation(&self, id: Uuid) -> Result<()> {
        let filter = doc! { "_id": id.to_string() };
        self.merge_operations().delete_one(filter).await?;
        Ok(())
    }

    pub async fn update_merge_operation_status(&self, operation_id: Uuid, status: MergeStatus) -> Result<()> {
        let filter = doc! { "_id": operation_id.to_string() };
        let status_str = format!("{:?}", status);
        let update = doc! { "$set": { "status": status_str } };
        self.merge_operations().update_one(filter, update).await?;
        Ok(())
    }

    pub async fn complete_merge_operation(&self, operation_id: Uuid, result_commit_id: Uuid) -> Result<()> {
        let filter = doc! { "_id": operation_id.to_string() };
        let update = doc! {
            "$set": {
                "status": "Completed",
                "result_commit_id": result_commit_id.to_string(),
                "completed_at": chrono::Utc::now().to_rfc3339()
            }
        };
        self.merge_operations().update_one(filter, update).await?;
        Ok(())
    }

    pub async fn abort_merge_operation(&self, operation_id: Uuid) -> Result<()> {
        let filter = doc! { "_id": operation_id.to_string() };
        let update = doc! {
            "$set": {
                "status": "Aborted",
                "completed_at": chrono::Utc::now().to_rfc3339()
            }
        };
        self.merge_operations().update_one(filter, update).await?;
        Ok(())
    }

    // Merge Conflict Methods
    pub async fn create_merge_conflict(&self, conflict: MergeConflict) -> Result<()> {
        let doc = mongodb::bson::to_document(&conflict)?;
        self.merge_conflicts().insert_one(doc).await?;
        Ok(())
    }

    pub async fn get_merge_conflict(&self, conflict_id: Uuid) -> Result<Option<MergeConflict>> {
        let filter = doc! { "_id": conflict_id.to_string() };
        if let Some(doc) = self.merge_conflicts().find_one(filter).await? {
            Ok(Some(mongodb::bson::from_document(doc)?))
        } else {
            Ok(None)
        }
    }

    pub async fn list_merge_conflicts(&self, operation_id: Uuid) -> Result<Vec<MergeConflict>> {
        let filter = doc! { "merge_operation_id": operation_id.to_string() };
        let mut cursor = self.merge_conflicts().find(filter).await?;
        let mut conflicts = Vec::new();
        while cursor.advance().await? {
            conflicts.push(mongodb::bson::from_document(cursor.deserialize_current()?)?);
        }
        Ok(conflicts)
    }

    pub async fn resolve_merge_conflict(&self, conflict_id: Uuid, resolution: ConflictResolution) -> Result<()> {
        let filter = doc! { "_id": conflict_id.to_string() };
        let resolution_doc = mongodb::bson::to_document(&resolution)?;
        let update = doc! { "$set": { "resolution": resolution_doc } };
        self.merge_conflicts().update_one(filter, update).await?;
        Ok(())
    }

    pub async fn delete_merge_conflict(&self, id: Uuid) -> Result<()> {
        let filter = doc! { "_id": id.to_string() };
        self.merge_conflicts().delete_one(filter).await?;
        Ok(())
    }

    pub async fn add_conflict_to_operation(&self, operation_id: Uuid, conflict_id: Uuid) -> Result<()> {
        let filter = doc! { "_id": operation_id.to_string() };
        let update = doc! { "$addToSet": { "conflicts": conflict_id.to_string() } };
        self.merge_operations().update_one(filter, update).await?;
        Ok(())
    }
}
