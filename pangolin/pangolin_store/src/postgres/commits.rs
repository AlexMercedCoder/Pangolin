use super::PostgresStore;
use anyhow::Result;
use pangolin_core::model::Commit;
use uuid::Uuid;
use sqlx::Row;

impl PostgresStore {
    // Commit Operations
    pub async fn create_commit(&self, tenant_id: Uuid, commit: Commit) -> Result<()> {
        sqlx::query("INSERT INTO commits (tenant_id, id, parent_id, timestamp, author, message, operations) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(tenant_id)
            .bind(commit.id)
            .bind(commit.parent_id)
            .bind(commit.timestamp)
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
            Ok(Some(Commit {
                id: row.get("id"),
                parent_id: row.get("parent_id"),
                timestamp: row.get("timestamp"),
                author: row.get("author"),
                message: row.get("message"),
                operations: serde_json::from_value(row.get("operations"))?,
            }))
        } else {
            Ok(None)
        }
    }
}
