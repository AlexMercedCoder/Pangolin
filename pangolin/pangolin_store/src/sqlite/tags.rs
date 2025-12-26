/// Tag operations for SqliteStore
use super::SqliteStore;
use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;
use pangolin_core::model::Tag;

impl SqliteStore {
    pub async fn create_tag(&self, tenant_id: Uuid, catalog_name: &str, tag: Tag) -> Result<()> {
        sqlx::query("INSERT INTO tags (tenant_id, catalog_name, name, commit_id) VALUES (?, ?, ?, ?)")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(tag.name)
            .bind(tag.commit_id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_tag(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Tag>> {
        let row = sqlx::query("SELECT name, commit_id FROM tags WHERE tenant_id = ? AND catalog_name = ? AND name = ?")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(Tag {
                name: row.get("name"),
                commit_id: Uuid::parse_str(&row.get::<String, _>("commit_id"))?,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_tags(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<Tag>> {
        let rows = sqlx::query("SELECT name, commit_id FROM tags WHERE tenant_id = ? AND catalog_name = ?")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .fetch_all(&self.pool)
            .await?;

        let mut tags = Vec::new();
        for row in rows {
            tags.push(Tag {
                name: row.get("name"),
                commit_id: Uuid::parse_str(&row.get::<String, _>("commit_id"))?,
            });
        }
        Ok(tags)
    }

    pub async fn delete_tag(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<()> {
        sqlx::query("DELETE FROM tags WHERE tenant_id = ? AND catalog_name = ? AND name = ?")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(&name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
