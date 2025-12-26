use super::PostgresStore;
use anyhow::Result;
use pangolin_core::model::Tag;
use uuid::Uuid;
use sqlx::Row;

impl PostgresStore {
    // Tag Operations
    pub async fn create_tag(&self, tenant_id: Uuid, catalog_name: &str, tag: Tag) -> Result<()> {
        sqlx::query("INSERT INTO tags (tenant_id, catalog_name, name, commit_id) VALUES ($1, $2, $3, $4)")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(tag.name)
            .bind(tag.commit_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_tag(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Tag>> {
        let row: Option<sqlx::postgres::PgRow> = sqlx::query("SELECT name, commit_id FROM tags WHERE tenant_id = $1 AND catalog_name = $2 AND name = $3")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(Tag {
                name: row.get("name"),
                commit_id: row.get("commit_id"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_tags(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<Tag>> {
        let rows = sqlx::query("SELECT name, commit_id FROM tags WHERE tenant_id = $1 AND catalog_name = $2")
            .bind(tenant_id)
            .bind(catalog_name)
            .fetch_all(&self.pool)
            .await?;

        let mut tags = Vec::new();
        for row in rows {
            tags.push(Tag {
                name: row.get("name"),
                commit_id: row.get("commit_id"),
            });
        }
        Ok(tags)
    }

    pub async fn delete_tag(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<()> {
        sqlx::query("DELETE FROM tags WHERE tenant_id = $1 AND catalog_name = $2 AND name = $3")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
