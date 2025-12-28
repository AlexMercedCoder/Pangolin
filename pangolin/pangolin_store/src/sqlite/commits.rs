/// Commit operations for SqliteStore
use super::SqliteStore;
use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;
use pangolin_core::model::Commit;

impl SqliteStore {
    pub async fn create_commit(&self, tenant_id: Uuid, commit: Commit) -> Result<()> {
        sqlx::query(
            "INSERT INTO commits (id, tenant_id, parent_id, timestamp, author, message, operations) 
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(commit.id.to_string())
        .bind(tenant_id.to_string())
        .bind(commit.parent_id.map(|id| id.to_string()))
        .bind(commit.timestamp) // Assumes i64 timestamp
        .bind(commit.author)
        .bind(commit.message)
        .bind(serde_json::to_string(&commit.operations)?)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_commit(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Commit>> {
        let row = sqlx::query(
            "SELECT id, parent_id, timestamp, author, message, operations 
             FROM commits WHERE tenant_id = ? AND id = ?"
        )
        .bind(tenant_id.to_string())
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Commit {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                parent_id: row.get::<Option<String>, _>("parent_id").map(|s| Uuid::parse_str(&s)).transpose()?,
                timestamp: row.get("timestamp"),
                author: row.get("author"),
                message: row.get("message"),
                operations: serde_json::from_str(&row.get::<String, _>("operations")).unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_commit_ancestry(&self, tenant_id: Uuid, commit_id: Uuid, limit: usize) -> Result<Vec<Commit>> {
        // Recursive CTE to fetch ancestry
        // Note: We use LIMIT in the final select to restrict depth
        let query = "
        WITH RECURSIVE ancestry AS (
            SELECT id, tenant_id, parent_id, timestamp, author, message, operations
            FROM commits
            WHERE tenant_id = ? AND id = ?
            
            UNION ALL
            
            SELECT c.id, c.tenant_id, c.parent_id, c.timestamp, c.author, c.message, c.operations
            FROM commits c
            INNER JOIN ancestry a ON c.id = a.parent_id
        )
        SELECT * FROM ancestry LIMIT ?;
        ";

        let rows = sqlx::query(query)
            .bind(tenant_id.to_string())
            .bind(commit_id.to_string())
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await?;

        let mut commits = Vec::new();
        for row in rows {
            commits.push(Commit {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                parent_id: row.get::<Option<String>, _>("parent_id").map(|s| Uuid::parse_str(&s)).transpose()?,
                timestamp: row.get("timestamp"),
                author: row.get("author"),
                message: row.get("message"),
                operations: serde_json::from_str(&row.get::<String, _>("operations")).unwrap_or_default(),
            });
        }
        
        Ok(commits)
    }
}
