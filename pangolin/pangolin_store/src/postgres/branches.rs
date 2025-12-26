use super::PostgresStore;
use anyhow::Result;
use pangolin_core::model::{Branch, BranchType};
use uuid::Uuid;
use sqlx::Row;

impl PostgresStore {
    // Branch Operations
    pub async fn create_branch(&self, tenant_id: Uuid, catalog_name: &str, branch: Branch) -> Result<()> {
        sqlx::query("INSERT INTO branches (tenant_id, catalog_name, name, head_commit_id, branch_type, assets) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(branch.name)
            .bind(branch.head_commit_id)
            .bind(format!("{:?}", branch.branch_type))
            .bind(branch.assets)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_branch(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Branch>> {
        let row: Option<sqlx::postgres::PgRow> = sqlx::query("SELECT name, head_commit_id, branch_type, assets FROM branches WHERE tenant_id = $1 AND catalog_name = $2 AND name = $3")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
             let type_str: String = row.get("branch_type");
             let branch_type = match type_str.as_str() {
                 "Ingest" => BranchType::Ingest,
                 "Experimental" => BranchType::Experimental,
                 _ => BranchType::Experimental,
             };

             Ok(Some(Branch {
                 name: row.get("name"),
                 head_commit_id: row.get("head_commit_id"),
                 branch_type,
                 assets: row.get("assets"),
             }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_branches(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<Branch>> {
        let rows = sqlx::query("SELECT name, head_commit_id, branch_type, assets FROM branches WHERE tenant_id = $1 AND catalog_name = $2")
            .bind(tenant_id)
            .bind(catalog_name)
            .fetch_all(&self.pool)
            .await?;

        let mut branches = Vec::new();
        for row in rows {
             let type_str: String = row.get("branch_type");
             let branch_type = match type_str.as_str() {
                 "Ingest" => BranchType::Ingest,
                 "Experimental" => BranchType::Experimental,
                 _ => BranchType::Experimental,
             };

             branches.push(Branch {
                 name: row.get("name"),
                 head_commit_id: row.get("head_commit_id"),
                 branch_type,
                 assets: row.get("assets"),
             });
        }
        Ok(branches)
    }

    pub async fn delete_branch(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<()> {
        let result = sqlx::query("DELETE FROM branches WHERE tenant_id = $1 AND catalog_name = $2 AND name = $3")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(&name)
            .execute(&self.pool)
            .await?;
        
        if result.rows_affected() == 0 {
             return Err(anyhow::anyhow!("Branch not found"));
        }
        
        // Also delete assets associated with this branch
        sqlx::query("DELETE FROM assets WHERE tenant_id = $1 AND catalog_name = $2 AND branch = $3")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(&name)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }

    pub async fn merge_branch(&self, tenant_id: Uuid, catalog_name: &str, target_branch: String, source_branch: String) -> Result<()> {
         // 1. Get Source Branch
        let source_row: sqlx::postgres::PgRow = sqlx::query("SELECT head_commit_id FROM branches WHERE tenant_id = $1 AND catalog_name = $2 AND name = $3")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(&source_branch)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Source branch not found"))?;
            
        let source_head: Option<Uuid> = source_row.get("head_commit_id");

        // 2. Update Target Branch
        let result = sqlx::query("UPDATE branches SET head_commit_id = $1 WHERE tenant_id = $2 AND catalog_name = $3 AND name = $4")
            .bind(source_head)
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(&target_branch)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Target branch not found"));
        }
        Ok(())
    }
}
