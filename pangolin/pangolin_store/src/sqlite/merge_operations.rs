/// Merge Operations implementation for SqliteStore
use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;
use chrono::Utc;
use pangolin_core::model::{MergeOperation, MergeStatus, MergeConflict, ConflictType, ConflictResolution};

use super::SqliteStore;

impl SqliteStore {
    pub async fn create_merge_operation(&self, operation: MergeOperation) -> Result<()> {
        sqlx::query(
            "INSERT INTO merge_operations (id, tenant_id, catalog_name, source_branch, target_branch, base_commit_id, status, created_by, created_at, result_commit_id, completed_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(operation.id.to_string())
        .bind(operation.tenant_id.to_string())
        .bind(operation.catalog_name)
        .bind(operation.source_branch)
        .bind(operation.target_branch)
        .bind(operation.base_commit_id.map(|id| id.to_string()))
        .bind(format!("{:?}", operation.status))
        .bind(operation.created_by.to_string())
        .bind(operation.created_at.timestamp())
        .bind(operation.result_commit_id.map(|id| id.to_string()))
        .bind(operation.completed_at.map(|t| t.timestamp()))
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn get_merge_operation(&self, operation_id: Uuid) -> Result<Option<MergeOperation>> {
        let row = sqlx::query(
            "SELECT id, tenant_id, catalog_name, source_branch, target_branch, base_commit_id, status, created_by, created_at, result_commit_id, completed_at
             FROM merge_operations WHERE id = ?"
        )
        .bind(operation_id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let status_str: String = row.get("status");
                let status = match status_str.as_str() {
                    "Pending" => MergeStatus::Pending,
                    "InProgress" => MergeStatus::InProgress,
                    "Conflicted" => MergeStatus::Conflicted,
                    "Completed" => MergeStatus::Completed,
                    "Aborted" => MergeStatus::Aborted,
                    _ => MergeStatus::Pending,
                };

                Ok(Some(MergeOperation {
                    id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                    tenant_id: Uuid::parse_str(&row.get::<String, _>("tenant_id"))?,
                    catalog_name: row.get("catalog_name"),
                    source_branch: row.get("source_branch"),
                    target_branch: row.get("target_branch"),
                    base_commit_id: row.get::<Option<String>, _>("base_commit_id").map(|s| Uuid::parse_str(&s)).transpose()?,
                    status,
                    created_by: Uuid::parse_str(&row.get::<String, _>("created_by"))?,
                    created_at: Utc.timestamp_opt(row.get("created_at"), 0).unwrap(),
                    result_commit_id: row.get::<Option<String>, _>("result_commit_id").map(|s| Uuid::parse_str(&s)).transpose()?,
                    completed_at: row.get::<Option<i64>, _>("completed_at").map(|t| Utc.timestamp_opt(t, 0).unwrap()),
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn list_merge_operations(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<MergeOperation>> {
        let rows = sqlx::query(
            "SELECT id, tenant_id, catalog_name, source_branch, target_branch, base_commit_id, status, created_by, created_at, result_commit_id, completed_at
             FROM merge_operations WHERE tenant_id = ? AND catalog_name = ?"
        )
        .bind(tenant_id.to_string())
        .bind(catalog_name)
        .fetch_all(&self.pool)
        .await?;

        let mut operations = Vec::new();
        for row in rows {
            let status_str: String = row.get("status");
            let status = match status_str.as_str() {
                "Pending" => MergeStatus::Pending,
                "InProgress" => MergeStatus::InProgress,
                "Conflicted" => MergeStatus::Conflicted,
                "Completed" => MergeStatus::Completed,
                "Aborted" => MergeStatus::Aborted,
                _ => MergeStatus::Pending,
            };

            operations.push(MergeOperation {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                tenant_id: Uuid::parse_str(&row.get::<String, _>("tenant_id"))?,
                catalog_name: row.get("catalog_name"),
                source_branch: row.get("source_branch"),
                target_branch: row.get("target_branch"),
                base_commit_id: row.get::<Option<String>, _>("base_commit_id").map(|s| Uuid::parse_str(&s)).transpose()?,
                status,
                created_by: Uuid::parse_str(&row.get::<String, _>("created_by"))?,
                created_at: Utc.timestamp_opt(row.get("created_at"), 0).unwrap(),
                result_commit_id: row.get::<Option<String>, _>("result_commit_id").map(|s| Uuid::parse_str(&s)).transpose()?,
                completed_at: row.get::<Option<i64>, _>("completed_at").map(|t| Utc.timestamp_opt(t, 0).unwrap()),
            });
        }

        Ok(operations)
    }

    pub async fn update_merge_operation_status(&self, operation_id: Uuid, status: MergeStatus) -> Result<()> {
        sqlx::query("UPDATE merge_operations SET status = ? WHERE id = ?")
            .bind(format!("{:?}", status))
            .bind(operation_id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn complete_merge_operation(&self, operation_id: Uuid, result_commit_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE merge_operations SET status = ?, result_commit_id = ?, completed_at = ? WHERE id = ?")
            .bind("Completed")
            .bind(result_commit_id.to_string())
            .bind(Utc::now().timestamp())
            .bind(operation_id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn abort_merge_operation(&self, operation_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE merge_operations SET status = ?, completed_at = ? WHERE id = ?")
            .bind("Aborted")
            .bind(Utc::now().timestamp())
            .bind(operation_id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn create_merge_conflict(&self, conflict: MergeConflict) -> Result<()> {
        let conflict_type_json = serde_json::to_string(&conflict.conflict_type)?;
        let resolution_json = conflict.resolution.as_ref().map(|r| serde_json::to_string(r)).transpose()?;

        sqlx::query(
            "INSERT INTO merge_conflicts (id, operation_id, conflict_type, asset_id, description, resolution, resolved_by, resolved_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(conflict.id.to_string())
        .bind(conflict.operation_id.to_string())
        .bind(conflict_type_json)
        .bind(conflict.asset_id.map(|id| id.to_string()))
        .bind(conflict.description)
        .bind(resolution_json)
        .bind(conflict.resolved_by.map(|id| id.to_string()))
        .bind(conflict.resolved_at.map(|t| t.timestamp()))
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn get_merge_conflict(&self, conflict_id: Uuid) -> Result<Option<MergeConflict>> {
        let row = sqlx::query(
            "SELECT id, operation_id, conflict_type, asset_id, description, resolution, resolved_by, resolved_at
             FROM merge_conflicts WHERE id = ?"
        )
        .bind(conflict_id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let conflict_type_json: String = row.get("conflict_type");
                let conflict_type: ConflictType = serde_json::from_str(&conflict_type_json)?;
                
                let resolution_json: Option<String> = row.get("resolution");
                let resolution = resolution_json.map(|json| serde_json::from_str(&json)).transpose()?;

                Ok(Some(MergeConflict {
                    id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                    operation_id: Uuid::parse_str(&row.get::<String, _>("operation_id"))?,
                    conflict_type,
                    asset_id: row.get::<Option<String>, _>("asset_id").map(|s| Uuid::parse_str(&s)).transpose()?,
                    description: row.get("description"),
                    resolution,
                    resolved_by: row.get::<Option<String>, _>("resolved_by").map(|s| Uuid::parse_str(&s)).transpose()?,
                    resolved_at: row.get::<Option<i64>, _>("resolved_at").map(|t| Utc.timestamp_opt(t, 0).unwrap()),
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn list_merge_conflicts(&self, operation_id: Uuid) -> Result<Vec<MergeConflict>> {
        let rows = sqlx::query(
            "SELECT id, operation_id, conflict_type, asset_id, description, resolution, resolved_by, resolved_at
             FROM merge_conflicts WHERE operation_id = ?"
        )
        .bind(operation_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        let mut conflicts = Vec::new();
        for row in rows {
            let conflict_type_json: String = row.get("conflict_type");
            let conflict_type: ConflictType = serde_json::from_str(&conflict_type_json)?;
            
            let resolution_json: Option<String> = row.get("resolution");
            let resolution = resolution_json.map(|json| serde_json::from_str(&json)).transpose()?;

            conflicts.push(MergeConflict {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                operation_id: Uuid::parse_str(&row.get::<String, _>("operation_id"))?,
                conflict_type,
                asset_id: row.get::<Option<String>, _>("asset_id").map(|s| Uuid::parse_str(&s)).transpose()?,
                description: row.get("description"),
                resolution,
                resolved_by: row.get::<Option<String>, _>("resolved_by").map(|s| Uuid::parse_str(&s)).transpose()?,
                resolved_at: row.get::<Option<i64>, _>("resolved_at").map(|t| Utc.timestamp_opt(t, 0).unwrap()),
            });
        }

        Ok(conflicts)
    }

    pub async fn resolve_merge_conflict(&self, conflict_id: Uuid, resolution: ConflictResolution) -> Result<()> {
        let resolution_json = serde_json::to_string(&resolution)?;

        sqlx::query("UPDATE merge_conflicts SET resolution = ?, resolved_by = ?, resolved_at = ? WHERE id = ?")
            .bind(resolution_json)
            .bind(resolution.resolved_by.to_string())
            .bind(resolution.resolved_at.timestamp())
            .bind(conflict_id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn add_conflict_to_operation(&self, _operation_id: Uuid, _conflict_id: Uuid) -> Result<()> {
        // Conflicts are already linked via operation_id foreign key
        Ok(())
    }
}
