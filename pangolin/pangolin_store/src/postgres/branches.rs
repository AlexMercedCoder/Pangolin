use super::PostgresStore;
use anyhow::Result;
use pangolin_core::model::{Branch, BranchType};
use uuid::Uuid;
use sqlx::Row;

impl PostgresStore {
    // Branch Operations
    pub async fn create_branch(&self, tenant_id: Uuid, catalog_name: &str, branch: Branch) -> Result<()> {
        sqlx::query("INSERT INTO branches (tenant_id, catalog_name, name, head_commit_id, branch_type, assets) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (tenant_id, catalog_name, name) DO UPDATE SET head_commit_id = EXCLUDED.head_commit_id, branch_type = EXCLUDED.branch_type, assets = EXCLUDED.assets")
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

    pub async fn list_branches(&self, tenant_id: Uuid, catalog_name: &str, pagination: Option<crate::PaginationParams>) -> Result<Vec<Branch>> {
        let limit = pagination.map(|p| p.limit.unwrap_or(i64::MAX as usize) as i64).unwrap_or(i64::MAX);
        let offset = pagination.map(|p| p.offset.unwrap_or(0) as i64).unwrap_or(0);

        let rows = sqlx::query("SELECT name, head_commit_id, branch_type, assets FROM branches WHERE tenant_id = $1 AND catalog_name = $2 LIMIT $3 OFFSET $4")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(limit)
            .bind(offset)
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

        // 3. Sync Assets (Fast-Forward Merge)
        // Copy assets from source_branch to target_branch, overwriting target assets if they exist.
        // We use ON CONFLICT based on the PK (tenant_id, catalog_name, branch_name, namespace_path, name).
        sqlx::query(
            "INSERT INTO assets (tenant_id, catalog_name, branch_name, namespace_path, name, asset_type, metadata_location, properties, id)
             SELECT tenant_id, catalog_name, $1, namespace_path, name, asset_type, metadata_location, properties, gen_random_uuid()
             FROM assets
             WHERE tenant_id = $2 AND catalog_name = $3 AND branch_name = $4
             ON CONFLICT (tenant_id, catalog_name, branch_name, namespace_path, name)
             DO UPDATE SET
                metadata_location = EXCLUDED.metadata_location,
                properties = EXCLUDED.properties,
                asset_type = EXCLUDED.asset_type,
                id = EXCLUDED.id"
        )
        .bind(&target_branch)
        .bind(tenant_id)
        .bind(catalog_name)
        .bind(&source_branch)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
