/// Branch operations for SqliteStore
use super::SqliteStore;
use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;
use pangolin_core::model::{Branch, BranchType};

impl SqliteStore {
    pub async fn create_branch(&self, tenant_id: Uuid, catalog_name: &str, branch: Branch) -> Result<()> {
        sqlx::query("INSERT INTO branches (tenant_id, catalog_name, name, head_commit_id, branch_type, assets) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(branch.name)
            .bind(branch.head_commit_id.map(|u| u.to_string()))
            .bind(format!("{:?}", branch.branch_type))
            .bind(serde_json::to_string(&branch.assets)?)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_branch(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Branch>> {
        let row = sqlx::query("SELECT name, head_commit_id, branch_type, assets FROM branches WHERE tenant_id = ? AND catalog_name = ? AND name = ?")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let branch_type_str: String = row.get("branch_type");
            let branch_type = match branch_type_str.as_str() {
                "Ingest" => BranchType::Ingest,
                _ => BranchType::Experimental,
            };

            Ok(Some(Branch {
                name: row.get("name"),
                head_commit_id: row.get::<Option<String>, _>("head_commit_id").map(|s| Uuid::parse_str(&s)).transpose()?,
                branch_type,
                assets: serde_json::from_str(&row.get::<String, _>("assets")).unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_branches(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<Branch>> {
        let rows = sqlx::query("SELECT name, head_commit_id, branch_type, assets FROM branches WHERE tenant_id = ? AND catalog_name = ?")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .fetch_all(&self.pool)
            .await?;

        let mut branches = Vec::new();
        for row in rows {
            let branch_type_str: String = row.get("branch_type");
            let branch_type = match branch_type_str.as_str() {
                "Ingest" => BranchType::Ingest,
                _ => BranchType::Experimental,
            };

            branches.push(Branch {
                name: row.get("name"),
                head_commit_id: row.get::<Option<String>, _>("head_commit_id").map(|s| Uuid::parse_str(&s)).transpose()?,
                branch_type,
                assets: serde_json::from_str(&row.get::<String, _>("assets")).unwrap_or_default(),
            });
        }
        Ok(branches)
    }

    pub async fn delete_branch(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<()> {
        let result = sqlx::query("DELETE FROM branches WHERE tenant_id = ? AND catalog_name = ? AND name = ?")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(&name)
            .execute(&self.pool)
            .await?;
        
        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Branch '{}' not found", name));
        }
        
        // Also delete assets associated with this branch
        sqlx::query("DELETE FROM assets WHERE tenant_id = ? AND catalog_name = ? AND branch = ?")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(&name)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }

    pub async fn merge_branch(&self, tenant_id: Uuid, catalog_name: &str, target_branch: String, source_branch: String) -> Result<()> {
         // 1. Get Source Branch
        let source_row = sqlx::query("SELECT head_commit_id FROM branches WHERE tenant_id = ? AND catalog_name = ? AND name = ?")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(&source_branch)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Source branch not found"))?;
            
        let source_head: Option<String> = source_row.get("head_commit_id");

        // 2. Update Target Branch
        let result = sqlx::query("UPDATE branches SET head_commit_id = ? WHERE tenant_id = ? AND catalog_name = ? AND name = ?")
            .bind(source_head)
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(&target_branch)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Target branch not found"));
        }

        // 3. Sync Assets (Fast-Forward Merge)
        // Copy assets from source_branch to target_branch, overwriting target assets if they exist.
        // using ON CONFLICT based on idx_assets_isolation (tenant_id, catalog_name, branch_name, namespace_path, name)
        sqlx::query(
            "INSERT INTO assets (tenant_id, catalog_name, branch_name, namespace_path, name, asset_type, metadata_location, properties, id)
             SELECT tenant_id, catalog_name, ?, namespace_path, name, asset_type, metadata_location, properties, (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(6))))
             FROM assets
             WHERE tenant_id = ? AND catalog_name = ? AND branch_name = ?
             ON CONFLICT (tenant_id, catalog_name, branch_name, namespace_path, name)
             DO UPDATE SET
                metadata_location = excluded.metadata_location,
                properties = excluded.properties,
                asset_type = excluded.asset_type,
                id = excluded.id"
        )
        .bind(&target_branch)
        .bind(tenant_id.to_string())
        .bind(catalog_name)
        .bind(&source_branch)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
