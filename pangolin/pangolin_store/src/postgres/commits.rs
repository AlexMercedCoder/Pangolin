use super::PostgresStore;
use anyhow::Result;
use pangolin_core::model::Commit;
use uuid::Uuid;
use sqlx::Row;
use chrono::{TimeZone, Utc};

impl PostgresStore {
    // Commit Operations
    pub async fn create_commit(&self, tenant_id: Uuid, commit: Commit) -> Result<()> {
        let timestamp_dt = Utc.timestamp_millis_opt(commit.timestamp).single().unwrap_or(Utc::now());
        
        sqlx::query("INSERT INTO commits (tenant_id, id, parent_id, timestamp, author, message, operations) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(tenant_id)
            .bind(commit.id)
            .bind(commit.parent_id)
            .bind(timestamp_dt)
            .bind(commit.author)
            .bind(commit.message)
            .bind(serde_json::to_value(commit.operations)?)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_commit(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Commit>> {
        let row: Option<sqlx::postgres::PgRow> = sqlx::query("SELECT id, parent_id, timestamp, author, message, operations FROM commits WHERE tenant_id = $1 AND id = $2")
            .bind(tenant_id)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let ts: chrono::DateTime<Utc> = row.get("timestamp");
            
            Ok(Some(Commit {
                id: row.get("id"),
                parent_id: row.get("parent_id"),
                timestamp: ts.timestamp_millis(),
                author: row.get("author"),
                message: row.get("message"),
                operations: serde_json::from_value(row.get("operations"))?,
            }))
        } else {
            Ok(None)
        }
    }
    pub async fn get_commit_ancestry(&self, tenant_id: Uuid, commit_id: Uuid, limit: usize) -> Result<Vec<Commit>> {
        let query = "
        WITH RECURSIVE ancestry AS (
            SELECT id, tenant_id, parent_id, timestamp, author, message, operations
            FROM commits
            WHERE tenant_id = $1 AND id = $2
            
            UNION ALL
            
            SELECT c.id, c.tenant_id, c.parent_id, c.timestamp, c.author, c.message, c.operations
            FROM commits c
            INNER JOIN ancestry a ON c.id = a.parent_id
        )
        SELECT * FROM ancestry LIMIT $3;
        ";

        let rows = sqlx::query(query)
            .bind(tenant_id)
            .bind(commit_id)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await?;

        let mut commits = Vec::new();
        for row in rows {
            let ts: chrono::DateTime<Utc> = row.get("timestamp");
            
            commits.push(Commit {
                id: row.get("id"),
                parent_id: row.get("parent_id"),
                timestamp: ts.timestamp_millis(),
                author: row.get("author"),
                message: row.get("message"),
                operations: serde_json::from_value(row.get("operations"))?,
            });
        }

        Ok(commits)
    }
}
