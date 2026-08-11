use super::PostgresStore;
use anyhow::Result;
use pangolin_core::model::{Branch, BranchType};
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    // Branch Operations
    pub async fn create_branch(
        &self,
        tenant_id: Uuid,
        catalog_name: &str,
        branch: Branch,
    ) -> Result<()> {
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

    /// Create a branch and copy assets into it in one transaction (A-24).
    ///
    /// Both statements commit together or neither does. Previously they were
    /// issued independently, so a failure between them left a branch that
    /// existed with some arbitrary prefix of its assets - and the API returned
    /// `200` regardless, because the caller logged the copy error and carried
    /// on.
    pub async fn create_branch_with_assets(
        &self,
        tenant_id: Uuid,
        catalog_name: &str,
        branch: Branch,
        src_branch: &str,
        assets: Option<Vec<String>>,
    ) -> Result<usize> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            "INSERT INTO branches (tenant_id, catalog_name, name, head_commit_id, branch_type, assets) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (tenant_id, catalog_name, name) DO UPDATE SET \
             head_commit_id = EXCLUDED.head_commit_id, branch_type = EXCLUDED.branch_type, \
             assets = EXCLUDED.assets",
        )
        .bind(tenant_id)
        .bind(catalog_name)
        .bind(&branch.name)
        .bind(branch.head_commit_id)
        .bind(format!("{:?}", branch.branch_type))
        .bind(&branch.assets)
        .execute(&mut *tx)
        .await?;

        // `INSERT ... SELECT` rather than reading into the process and writing
        // back: the copy is one statement, so there is no window in which some
        // rows exist and others do not, and a large branch does not have to fit
        // in memory.
        //
        // `gen_random_uuid()` for the new ids because an asset's primary key is
        // unique across branches; reusing the source ids would collide.
        let copied = match &assets {
            None => sqlx::query(
                "INSERT INTO assets (id, tenant_id, catalog_name, namespace_path, name, \
                     branch_name, asset_type, metadata_location, properties) \
                     SELECT gen_random_uuid(), tenant_id, catalog_name, namespace_path, name, \
                     $4, asset_type, metadata_location, properties \
                     FROM assets \
                     WHERE tenant_id = $1 AND catalog_name = $2 AND branch_name = $3",
            )
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(src_branch)
            .bind(&branch.name)
            .execute(&mut *tx)
            .await?
            .rows_affected(),
            Some(names) => {
                if names.is_empty() {
                    0
                } else {
                    // `namespace.table`, so the last segment is the table and
                    // everything before it is the namespace path.
                    let mut wanted: Vec<String> = Vec::with_capacity(names.len());
                    for full in names {
                        let Some((_, table)) = full.rsplit_once('.') else {
                            // A name with no namespace cannot identify an asset;
                            // failing is better than silently copying nothing,
                            // which is what the old loop did with `continue`.
                            tx.rollback().await.ok();
                            return Err(anyhow::anyhow!(
                                "asset name {full:?} is not in namespace.table form"
                            ));
                        };
                        wanted.push(table.to_string());
                    }

                    sqlx::query(
                        "INSERT INTO assets (id, tenant_id, catalog_name, namespace_path, name, \
                         branch_name, asset_type, metadata_location, properties) \
                         SELECT gen_random_uuid(), tenant_id, catalog_name, namespace_path, name, \
                         $4, asset_type, metadata_location, properties \
                         FROM assets \
                         WHERE tenant_id = $1 AND catalog_name = $2 AND branch_name = $3 \
                         AND array_to_string(namespace_path, '.') || '.' || name = ANY($5)",
                    )
                    .bind(tenant_id)
                    .bind(catalog_name)
                    .bind(src_branch)
                    .bind(&branch.name)
                    .bind(names)
                    .execute(&mut *tx)
                    .await?
                    .rows_affected()
                }
            }
        };

        tx.commit().await?;
        Ok(copied as usize)
    }

    pub async fn get_branch(
        &self,
        tenant_id: Uuid,
        catalog_name: &str,
        name: String,
    ) -> Result<Option<Branch>> {
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

    pub async fn list_branches(
        &self,
        tenant_id: Uuid,
        catalog_name: &str,
        pagination: Option<crate::PaginationParams>,
    ) -> Result<Vec<Branch>> {
        let limit = pagination
            .map(|p| p.limit.unwrap_or(i64::MAX as usize) as i64)
            .unwrap_or(i64::MAX);
        let offset = pagination
            .map(|p| p.offset.unwrap_or(0) as i64)
            .unwrap_or(0);

        let rows = sqlx::query("SELECT name, head_commit_id, branch_type, assets FROM branches WHERE tenant_id = $1 AND catalog_name = $2 ORDER BY name LIMIT $3 OFFSET $4")
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

    /// Delete a branch and the assets that live on it.
    ///
    /// Transactional: the branch row and its assets are two statements, and a
    /// failure between them used to leave assets belonging to a branch that no
    /// longer exists, invisible to every listing and impossible to clean up
    /// through the API (A-24).
    pub async fn delete_branch(
        &self,
        tenant_id: Uuid,
        catalog_name: &str,
        name: String,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        let result = sqlx::query(
            "DELETE FROM branches WHERE tenant_id = $1 AND catalog_name = $2 AND name = $3",
        )
        .bind(tenant_id)
        .bind(catalog_name)
        .bind(&name)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            // Rolls back on drop.
            return Err(anyhow::anyhow!("Branch not found"));
        }

        // `branch`, not `branch_name`: the column was renamed by
        // 20251221000000_fix_asset_branching.sql and this query was never
        // updated, so it failed at runtime every time.
        sqlx::query(
            "DELETE FROM assets WHERE tenant_id = $1 AND catalog_name = $2 AND branch_name = $3",
        )
        .bind(tenant_id)
        .bind(catalog_name)
        .bind(&name)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Fast-forward `target_branch` to `source_branch`.
    ///
    /// Transactional. Moving the branch head and copying the assets are
    /// separate statements; a failure between them left the target pointing at
    /// the source's head while still carrying its own assets — a catalog that
    /// claims one state and holds another, with no rollback and no repair tool
    /// (A-24).
    /// Fast-forward `target_branch` to `source_branch`.
    ///
    /// Named differently from `CatalogStore::merge_branch`, and taking the same
    /// (source, target) order, on purpose. There used to be an inherent
    /// `merge_branch` taking (target, source) alongside a trait `merge_branch`
    /// taking (source, target): Rust resolves the inherent method first, so any
    /// caller holding a concrete backend silently merged in the opposite
    /// direction. Two names cannot be confused; two argument orders under one
    /// name can.
    pub async fn merge_branch_into(
        &self,
        tenant_id: Uuid,
        catalog_name: &str,
        source_branch: String,
        target_branch: String,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // 1. Get Source Branch
        let source_row: sqlx::postgres::PgRow = sqlx::query("SELECT head_commit_id FROM branches WHERE tenant_id = $1 AND catalog_name = $2 AND name = $3")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(&source_branch)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Source branch not found"))?;

        let source_head: Option<Uuid> = source_row.get("head_commit_id");

        // 2. Update Target Branch
        let result = sqlx::query("UPDATE branches SET head_commit_id = $1 WHERE tenant_id = $2 AND catalog_name = $3 AND name = $4")
            .bind(source_head)
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(&target_branch)
            .execute(&mut *tx)
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
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}
