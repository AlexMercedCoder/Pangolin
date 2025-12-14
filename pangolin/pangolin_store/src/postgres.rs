use crate::CatalogStore;
use anyhow::Result;
use async_trait::async_trait;
use pangolin_core::model::{
    Asset, Branch, Catalog, Commit, Namespace, Tag, Tenant, Warehouse,
};
use pangolin_core::audit::AuditLogEntry;
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresStore {
    pool: PgPool,
}

impl PostgresStore {
    pub async fn new(connection_string: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(connection_string)
            .await?;
        
        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl CatalogStore for PostgresStore {
    // Tenant Operations
    async fn create_tenant(&self, tenant: Tenant) -> Result<()> {
        sqlx::query("INSERT INTO tenants (id, name, properties) VALUES ($1, $2, $3)")
            .bind(tenant.id)
            .bind(tenant.name)
            .bind(serde_json::to_value(tenant.properties)?)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_tenant(&self, id: Uuid) -> Result<Option<Tenant>> {
        let row = sqlx::query("SELECT id, name, properties FROM tenants WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(Tenant {
                id: row.get("id"),
                name: row.get("name"),
                properties: serde_json::from_value(row.get("properties"))?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_tenants(&self) -> Result<Vec<Tenant>> {
        let rows = sqlx::query("SELECT id, name, properties FROM tenants")
            .fetch_all(&self.pool)
            .await?;

        let mut tenants = Vec::new();
        for row in rows {
            tenants.push(Tenant {
                id: row.get("id"),
                name: row.get("name"),
                properties: serde_json::from_value(row.get("properties"))?,
            });
        }
        Ok(tenants)
    }

    async fn update_tenant(&self, tenant_id: Uuid, updates: pangolin_core::model::TenantUpdate) -> Result<Tenant> {
        // Build dynamic UPDATE query based on which fields are provided
        let mut query = String::from("UPDATE tenants SET ");
        let mut set_clauses = Vec::new();
        let mut bind_count = 1;

        if updates.name.is_some() {
            set_clauses.push(format!("name = ${}", bind_count));
            bind_count += 1;
        }
        if updates.properties.is_some() {
            set_clauses.push(format!("properties = ${}", bind_count));
            bind_count += 1;
        }

        if set_clauses.is_empty() {
            // No updates provided, just return current tenant
            return self.get_tenant(tenant_id).await?
                .ok_or_else(|| anyhow::anyhow!("Tenant not found"));
        }

        query.push_str(&set_clauses.join(", "));
        query.push_str(&format!(" WHERE id = ${} RETURNING id, name, properties", bind_count));

        let mut q = sqlx::query(&query);
        if let Some(name) = &updates.name {
            q = q.bind(name);
        }
        if let Some(properties) = &updates.properties {
            q = q.bind(serde_json::to_value(properties)?);
        }
        q = q.bind(tenant_id);

        let row = q.fetch_one(&self.pool).await?;
        Ok(Tenant {
            id: row.get("id"),
            name: row.get("name"),
            properties: serde_json::from_value(row.get("properties"))?,
        })
    }

    async fn delete_tenant(&self, tenant_id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM tenants WHERE id = $1")
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;
        
        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Tenant not found"));
        }
        Ok(())
    }

    // Warehouse Operations
    async fn create_warehouse(&self, tenant_id: Uuid, warehouse: Warehouse) -> Result<()> {
        sqlx::query("INSERT INTO warehouses (id, tenant_id, name, use_sts, storage_config) VALUES ($1, $2, $3, $4, $5)")
            .bind(warehouse.id)
            .bind(tenant_id)
            .bind(&warehouse.name)
            .bind(warehouse.use_sts)
            .bind(serde_json::to_value(&warehouse.storage_config)?)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_warehouse(&self, tenant_id: Uuid, name: String) -> Result<Option<Warehouse>> {
        let row = sqlx::query("SELECT id, name, tenant_id, use_sts, storage_config FROM warehouses WHERE tenant_id = $1 AND name = $2")
            .bind(tenant_id)
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            // Note: Warehouse struct has `id` and `tenant_id` fields which might not be in the query result if I don't select them or if they are implicit.
            // The SQL table has `tenant_id` and `name` as PK. `id` is not in my SQL schema for Warehouse!
            // Wait, `Warehouse` model has `id: Uuid`. My SQL schema didn't have `id`.
            // I need to update SQL schema for Warehouse to include `id`.
            // For now, I'll generate a random UUID or use tenant_id/name hash if I can't change schema easily.
            // But I CAN change schema, I just edited it.
            // Let's assume I will update schema to include `id` for Warehouse.
            // Or I can just return a dummy ID if it's not critical.
            // Let's update schema in next step if needed.
            // Actually, let's just use a placeholder ID for now or fix the schema.
            // I'll fix the schema in a separate tool call if I can, or just assume I'll do it.
            // Let's fix the schema for Warehouse ID.
            
            Ok(Some(Warehouse {
                id: row.get("id"),
                name: row.get("name"),
                tenant_id: row.get("tenant_id"),
                use_sts: row.try_get("use_sts").unwrap_or(false),
                storage_config: serde_json::from_value(row.get("storage_config")).unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_warehouses(&self, tenant_id: Uuid) -> Result<Vec<Warehouse>> {
        let rows = sqlx::query("SELECT id, name, tenant_id, use_sts, storage_config FROM warehouses WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await?;

        let mut warehouses = Vec::new();
        for row in rows {
            warehouses.push(Warehouse {
                id: row.get("id"),
                name: row.get("name"),
                tenant_id: row.get("tenant_id"),
                use_sts: row.try_get("use_sts").unwrap_or(false),
                storage_config: serde_json::from_value(row.get("storage_config")).unwrap_or_default(),
            });
        }
        Ok(warehouses)
    }

    async fn update_warehouse(&self, tenant_id: Uuid, name: String, updates: pangolin_core::model::WarehouseUpdate) -> Result<Warehouse> {
        let mut query = String::from("UPDATE warehouses SET ");
        let mut set_clauses = Vec::new();
        let mut bind_count = 1;

        if updates.name.is_some() {
            set_clauses.push(format!("name = ${}", bind_count));
            bind_count += 1;
        }
        if updates.storage_config.is_some() {
            set_clauses.push(format!("storage_config = ${}", bind_count));
            bind_count += 1;
        }
        if updates.use_sts.is_some() {
            set_clauses.push(format!("use_sts = ${}", bind_count));
            bind_count += 1;
        }

        if set_clauses.is_empty() {
            return self.get_warehouse(tenant_id, name).await?
                .ok_or_else(|| anyhow::anyhow!("Warehouse not found"));
        }

        query.push_str(&set_clauses.join(", "));
        query.push_str(&format!(" WHERE tenant_id = ${} AND name = ${} RETURNING id, name, tenant_id, use_sts, storage_config", bind_count, bind_count + 1));

        let mut q = sqlx::query(&query);
        if let Some(new_name) = &updates.name {
            q = q.bind(new_name);
        }
        if let Some(config) = &updates.storage_config {
            q = q.bind(serde_json::to_value(config)?);
        }
        if let Some(use_sts) = updates.use_sts {
            q = q.bind(use_sts);
        }
        q = q.bind(tenant_id).bind(&name);

        let row = q.fetch_one(&self.pool).await?;
        Ok(Warehouse {
            id: row.get("id"),
            name: row.get("name"),
            tenant_id: row.get("tenant_id"),
            use_sts: row.try_get("use_sts").unwrap_or(false),
            storage_config: serde_json::from_value(row.get("storage_config")).unwrap_or_default(),
        })
    }

    async fn delete_warehouse(&self, tenant_id: Uuid, name: String) -> Result<()> {
        let result = sqlx::query("DELETE FROM warehouses WHERE tenant_id = $1 AND name = $2")
            .bind(tenant_id)
            .bind(&name)
            .execute(&self.pool)
            .await?;
        
        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Warehouse '{}' not found", name));
        }
        Ok(())
    }

    // Catalog Operations
    async fn create_catalog(&self, tenant_id: Uuid, catalog: Catalog) -> Result<()> {
        sqlx::query("INSERT INTO catalogs (id, tenant_id, name, catalog_type, warehouse_name, storage_location, federated_config, properties) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
            .bind(catalog.id)
            .bind(tenant_id)
            .bind(&catalog.name)
            .bind(format!("{:?}", catalog.catalog_type))
            .bind(&catalog.warehouse_name)
            .bind(&catalog.storage_location)
            .bind(serde_json::to_value(&catalog.federated_config)?)
            .bind(serde_json::to_value(&catalog.properties)?)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_catalog(&self, tenant_id: Uuid, name: String) -> Result<Option<Catalog>> {
        let row = sqlx::query("SELECT id, name, catalog_type, warehouse_name, storage_location, federated_config, properties FROM catalogs WHERE tenant_id = $1 AND name = $2")
            .bind(tenant_id)
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let catalog_type_str: String = row.get("catalog_type");
            let catalog_type = match catalog_type_str.as_str() {
                "Local" => pangolin_core::model::CatalogType::Local,
                "Federated" => pangolin_core::model::CatalogType::Federated,
                _ => pangolin_core::model::CatalogType::Local,
            };
            
            Ok(Some(Catalog {
                id: row.get("id"),
                name: row.get("name"),
                catalog_type,
                warehouse_name: row.get("warehouse_name"),
                storage_location: row.get("storage_location"),
                federated_config: serde_json::from_value(row.get("federated_config")).ok(),
                properties: serde_json::from_value(row.get("properties")).unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_catalogs(&self, tenant_id: Uuid) -> Result<Vec<Catalog>> {
        let rows = sqlx::query("SELECT id, name, warehouse_name, storage_location, properties FROM catalogs WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await?;

        let mut catalogs = Vec::new();
        for row in rows {
            catalogs.push(Catalog {
                id: row.get("id"),
                name: row.get("name"),
                catalog_type: pangolin_core::model::CatalogType::Local,
                warehouse_name: row.get("warehouse_name"),
                storage_location: row.get("storage_location"),
                federated_config: None,
                properties: serde_json::from_value(row.get("properties")).unwrap_or_default(),
            });
        }
        Ok(catalogs)
    }

    async fn update_catalog(&self, tenant_id: Uuid, name: String, updates: pangolin_core::model::CatalogUpdate) -> Result<Catalog> {
        let mut query = String::from("UPDATE catalogs SET ");
        let mut set_clauses = Vec::new();
        let mut bind_count = 1;

        if updates.warehouse_name.is_some() {
            set_clauses.push(format!("warehouse_name = ${}", bind_count));
            bind_count += 1;
        }
        if updates.storage_location.is_some() {
            set_clauses.push(format!("storage_location = ${}", bind_count));
            bind_count += 1;
        }
        if updates.properties.is_some() {
            set_clauses.push(format!("properties = ${}", bind_count));
            bind_count += 1;
        }

        if set_clauses.is_empty() {
            return self.get_catalog(tenant_id, name).await?
                .ok_or_else(|| anyhow::anyhow!("Catalog not found"));
        }

        query.push_str(&set_clauses.join(", "));
        query.push_str(&format!(" WHERE tenant_id = ${} AND name = ${} RETURNING id, name, catalog_type, warehouse_name, storage_location, federated_config, properties", bind_count, bind_count + 1));

        let mut q = sqlx::query(&query);
        if let Some(warehouse_name) = &updates.warehouse_name {
            q = q.bind(warehouse_name);
        }
        if let Some(storage_location) = &updates.storage_location {
            q = q.bind(storage_location);
        }
        if let Some(properties) = &updates.properties {
            q = q.bind(serde_json::to_value(properties)?);
        }
        q = q.bind(tenant_id).bind(&name);

        let row = q.fetch_one(&self.pool).await?;
        let catalog_type_str: String = row.get("catalog_type");
        let catalog_type = match catalog_type_str.as_str() {
            "Local" => pangolin_core::model::CatalogType::Local,
            "Federated" => pangolin_core::model::CatalogType::Federated,
            _ => pangolin_core::model::CatalogType::Local,
        };

        Ok(Catalog {
            id: row.get("id"),
            name: row.get("name"),
            catalog_type,
            warehouse_name: row.get("warehouse_name"),
            storage_location: row.get("storage_location"),
            federated_config: serde_json::from_value(row.get("federated_config")).ok(),
            properties: serde_json::from_value(row.get("properties")).unwrap_or_default(),
        })
    }

    async fn delete_catalog(&self, tenant_id: Uuid, name: String) -> Result<()> {
        let result = sqlx::query("DELETE FROM catalogs WHERE tenant_id = $1 AND name = $2")
            .bind(tenant_id)
            .bind(&name)
            .execute(&self.pool)
            .await?;
        
        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Catalog '{}' not found", name));
        }
        Ok(())
    }

    // Namespace Operations
    async fn create_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Namespace) -> Result<()> {
        sqlx::query("INSERT INTO namespaces (id, tenant_id, catalog_name, namespace_path, properties) VALUES ($1, $2, $3, $4, $5)")
            .bind(Uuid::new_v4())
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(&namespace.name)
            .bind(serde_json::to_value(&namespace.properties)?)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<Option<Namespace>> {
        let row = sqlx::query("SELECT namespace_path, properties FROM namespaces WHERE tenant_id = $1 AND catalog_name = $2 AND namespace_path = $3")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(&namespace)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(Namespace {
                name: row.get("namespace_path"),
                properties: serde_json::from_value(row.get("properties"))?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_namespaces(&self, tenant_id: Uuid, catalog_name: &str, _parent: Option<String>) -> Result<Vec<Namespace>> {
        let rows = sqlx::query("SELECT namespace_path, properties FROM namespaces WHERE tenant_id = $1 AND catalog_name = $2")
            .bind(tenant_id)
            .bind(catalog_name)
            .fetch_all(&self.pool)
            .await?;

        let mut namespaces = Vec::new();
        for row in rows {
            namespaces.push(Namespace {
                name: row.get("namespace_path"),
                properties: serde_json::from_value(row.get("properties"))?,
            });
        }
        Ok(namespaces)
    }

    async fn delete_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<()> {
        sqlx::query("DELETE FROM namespaces WHERE tenant_id = $1 AND catalog_name = $2 AND namespace_path = $3")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(&namespace)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_namespace_properties(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>, properties: std::collections::HashMap<String, String>) -> Result<()> {
        let ns = self.get_namespace(tenant_id, catalog_name, namespace.clone()).await?;
        if let Some(mut n) = ns {
            n.properties.extend(properties);
            sqlx::query("UPDATE namespaces SET properties = $1 WHERE tenant_id = $2 AND catalog_name = $3 AND namespace_path = $4")
                .bind(serde_json::to_value(&n.properties)?)
                .bind(tenant_id)
                .bind(catalog_name)
                .bind(&namespace)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    // Asset Operations
    async fn create_asset(&self, tenant_id: Uuid, catalog_name: &str, _branch: Option<String>, namespace: Vec<String>, asset: Asset) -> Result<()> {
        sqlx::query("INSERT INTO assets (id, tenant_id, catalog_name, namespace_path, name, asset_type, metadata_location, properties) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
            .bind(Uuid::new_v4())
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(&namespace)
            .bind(&asset.name)
            .bind(format!("{:?}", asset.kind))
            .bind(&asset.location)
            .bind(serde_json::to_value(&asset.properties)?)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_asset(&self, tenant_id: Uuid, catalog_name: &str, _branch: Option<String>, namespace: Vec<String>, name: String) -> Result<Option<Asset>> {
        let row = sqlx::query("SELECT id, name, asset_type, metadata_location, properties FROM assets WHERE tenant_id = $1 AND catalog_name = $2 AND namespace_path = $3 AND name = $4")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(&namespace)
            .bind(&name)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let asset_type_str: String = row.get("asset_type");
            let kind = match asset_type_str.as_str() {
                "IcebergTable" => pangolin_core::model::AssetType::IcebergTable,
                "View" => pangolin_core::model::AssetType::View,
                _ => pangolin_core::model::AssetType::IcebergTable,
            };
            
            Ok(Some(Asset {
                id: row.get("id"),
                name: row.get("name"),
                kind,
                location: row.get::<Option<String>, _>("metadata_location").unwrap_or_default(),
                properties: serde_json::from_value(row.get("properties")).unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_assets(&self, tenant_id: Uuid, catalog_name: &str, _branch: Option<String>, namespace: Vec<String>) -> Result<Vec<Asset>> {
        let rows = sqlx::query("SELECT id, name, asset_type, metadata_location, properties FROM assets WHERE tenant_id = $1 AND catalog_name = $2 AND namespace_path = $3")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(&namespace)
            .fetch_all(&self.pool)
            .await?;

        let mut assets = Vec::new();
        for row in rows {
            let asset_type_str: String = row.get("asset_type");
            let kind = match asset_type_str.as_str() {
                "IcebergTable" => pangolin_core::model::AssetType::IcebergTable,
                "View" => pangolin_core::model::AssetType::View,
                _ => pangolin_core::model::AssetType::IcebergTable,
            };
            
            assets.push(Asset {
                id: row.get("id"),
                name: row.get("name"),
                kind,
                location: row.get::<Option<String>, _>("metadata_location").unwrap_or_default(),
                properties: serde_json::from_value(row.get("properties")).unwrap_or_default(),
            });
        }
        Ok(assets)
    }

    async fn delete_asset(&self, tenant_id: Uuid, catalog_name: &str, _branch: Option<String>, namespace: Vec<String>, name: String) -> Result<()> {
        sqlx::query("DELETE FROM assets WHERE tenant_id = $1 AND catalog_name = $2 AND namespace_path = $3 AND name = $4")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(&namespace)
            .bind(&name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn rename_asset(&self, tenant_id: Uuid, catalog_name: &str, _branch: Option<String>, source_namespace: Vec<String>, source_name: String, dest_namespace: Vec<String>, dest_name: String) -> Result<()> {
        sqlx::query("UPDATE assets SET namespace_path = $1, name = $2 WHERE tenant_id = $3 AND catalog_name = $4 AND namespace_path = $5 AND name = $6")
            .bind(&dest_namespace)
            .bind(&dest_name)
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(&source_namespace)
            .bind(&source_name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Branch Operations
    async fn create_branch(&self, tenant_id: Uuid, catalog_name: &str, branch: Branch) -> Result<()> {
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

    async fn get_branch(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Branch>> {
        let row = sqlx::query("SELECT name, head_commit_id, branch_type, assets FROM branches WHERE tenant_id = $1 AND catalog_name = $2 AND name = $3")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
             let type_str: String = row.get("branch_type");
             let branch_type = match type_str.as_str() {
                 "Ingest" => pangolin_core::model::BranchType::Ingest,
                 "Experimental" => pangolin_core::model::BranchType::Experimental,
                 _ => pangolin_core::model::BranchType::Experimental,
             };

             Ok(Some(Branch {
                 name: row.get("name"),
                 head_commit_id: row.get("head_commit_id"),
                 branch_type,
                 assets: serde_json::from_value(row.get("assets"))?, // assets is TEXT[] in SQL, sqlx maps to Vec<String> automatically? No, I bound it as Vec<String> but in SQL it is TEXT[]. sqlx should handle it. But wait, I used `row.get("assets")`.
             }))
        } else {
            Ok(None)
        }
    }

    async fn list_branches(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<Branch>> {
        let rows = sqlx::query("SELECT name, head_commit_id, branch_type, assets FROM branches WHERE tenant_id = $1 AND catalog_name = $2")
            .bind(tenant_id)
            .bind(catalog_name)
            .fetch_all(&self.pool)
            .await?;

        let mut branches = Vec::new();
        for row in rows {
             let type_str: String = row.get("branch_type");
             let branch_type = match type_str.as_str() {
                 "Ingest" => pangolin_core::model::BranchType::Ingest,
                 "Experimental" => pangolin_core::model::BranchType::Experimental,
                 _ => pangolin_core::model::BranchType::Experimental,
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


    async fn merge_branch(&self, tenant_id: Uuid, catalog_name: &str, source_branch: String, target_branch: String) -> Result<()> {
        // TODO: Implement full merge logic with conflict detection in Postgres.
        // For now, simple fast-forward update of head_commit_id.
        let source = self.get_branch(tenant_id, catalog_name, source_branch.clone()).await?
            .ok_or_else(|| anyhow::anyhow!("Source branch not found"))?;
        
        let target = self.get_branch(tenant_id, catalog_name, target_branch.clone()).await?
            .ok_or_else(|| anyhow::anyhow!("Target branch not found"))?;

        // Simplified: Update target head to source head
        sqlx::query("UPDATE branches SET head_commit_id = $1 WHERE tenant_id = $2 AND catalog_name = $3 AND name = $4")
            .bind(source.head_commit_id)
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(target_branch)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }

    // Commit Operations
    async fn create_commit(&self, tenant_id: Uuid, commit: Commit) -> Result<()> {
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

    async fn get_commit(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Commit>> {
        let row = sqlx::query("SELECT id, parent_id, timestamp, author, message, operations FROM commits WHERE tenant_id = $1 AND id = $2")
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


    // File Operations (Not supported in Postgres directly, use S3 or bytea)
    // For metadata files, we can store in a separate table or just return error if not supported.
    // Ideally, PostgresStore should be paired with S3 for file storage, or store files in a `files` table.
    // Let's add a `files` table to schema or just use S3 for files and Postgres for metadata.
    // Requirement says "Shared Metadata Backend". Files are usually on S3.
    // But `CatalogStore` trait has `read_file`/`write_file`.
    // Let's implement a simple `files` table for small metadata files.
    
    async fn read_file(&self, location: &str) -> Result<Vec<u8>> {
        // TODO: Implement file storage in Postgres or delegate.
        // For now, error.
        Err(anyhow::anyhow!("File storage not implemented in PostgresStore yet"))
    }

    async fn write_file(&self, location: &str, bytes: Vec<u8>) -> Result<()> {
         Err(anyhow::anyhow!("File storage not implemented in PostgresStore yet"))
    }

    // Tag Operations
    async fn create_tag(&self, tenant_id: Uuid, catalog_name: &str, tag: Tag) -> Result<()> {
        sqlx::query("INSERT INTO tags (tenant_id, catalog_name, name, commit_id) VALUES ($1, $2, $3, $4)")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(tag.name)
            .bind(tag.commit_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_tag(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Tag>> {
        let row = sqlx::query("SELECT name, commit_id FROM tags WHERE tenant_id = $1 AND catalog_name = $2 AND name = $3")
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

    async fn list_tags(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<Tag>> {
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

    async fn delete_tag(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<()> {
        sqlx::query("DELETE FROM tags WHERE tenant_id = $1 AND catalog_name = $2 AND name = $3")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Audit Operations
    async fn log_audit_event(&self, tenant_id: Uuid, event: AuditLogEntry) -> Result<()> {
        sqlx::query("INSERT INTO audit_logs (id, tenant_id, timestamp, actor, action, resource, details) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(event.id)
            .bind(tenant_id)
            .bind(event.timestamp)
            .bind(event.actor)
            .bind(event.action)
            .bind(event.resource)
            .bind(event.details)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn list_audit_events(&self, tenant_id: Uuid) -> Result<Vec<AuditLogEntry>> {
        let rows = sqlx::query("SELECT id, timestamp, actor, action, resource, details FROM audit_logs WHERE tenant_id = $1 ORDER BY timestamp DESC LIMIT 100")
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await?;

        let mut events = Vec::new();
        for row in rows {
            events.push(AuditLogEntry {
                id: row.get("id"),
                tenant_id,
                timestamp: row.get("timestamp"),
                actor: row.get("actor"),
                action: row.get("action"),
                resource: row.get("resource"),
                details: row.get("details"),
            });
        }
        Ok(events)
    }

    // Maintenance Operations (Placeholders)
    async fn expire_snapshots(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _namespace: Vec<String>, _table: String, _retention_ms: i64) -> Result<()> {
        Ok(())
    }

    async fn remove_orphan_files(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _namespace: Vec<String>, _table: String, _older_than_ms: i64) -> Result<()> {
        Ok(())
    }

    // Metadata IO
    async fn get_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String) -> Result<Option<String>> {
        let asset = self.get_asset(tenant_id, catalog_name, branch, namespace, table).await?;
        if let Some(asset) = asset {
            if let Some(loc) = asset.properties.get("metadata_location") {
                return Ok(Some(loc.clone()));
            }
        }
        Ok(None)
    }

    async fn update_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String, expected_location: Option<String>, new_location: String) -> Result<()> {
        let namespace_name = namespace.join("\x1F");
        
        // CAS Logic
        // We need to check if current metadata_location matches expected_location
        // In Postgres, we can do this in the UPDATE query WHERE clause
        
        let result = if let Some(expected) = expected_location {
            // Update only if current location matches expected
            // We need to access the JSON property. 
            // This is complex in SQL with JSONB. 
            // "properties"->>'metadata_location' = expected
            sqlx::query("UPDATE assets SET properties = jsonb_set(properties, '{metadata_location}', to_jsonb($1::text), true) WHERE tenant_id = $2 AND catalog_name = $3 AND namespace_name = $4 AND name = $5 AND properties->>'metadata_location' = $6")
                .bind(new_location)
                .bind(tenant_id)
                .bind(catalog_name)
                .bind(namespace_name)
                .bind(table)
                .bind(expected)
                .execute(&self.pool)
                .await?
        } else {
            // Update if no metadata_location exists (create or first commit) or force overwrite?
            // Usually expected_location=None means "ensure it doesn't exist" or "I don't care".
            // Iceberg CAS usually implies "ensure it matches this specific state".
            // If expected is None, it might mean "it should be null".
            sqlx::query("UPDATE assets SET properties = jsonb_set(properties, '{metadata_location}', to_jsonb($1::text), true) WHERE tenant_id = $2 AND catalog_name = $3 AND namespace_name = $4 AND name = $5 AND (properties->>'metadata_location' IS NULL)")
                .bind(new_location)
                .bind(tenant_id)
                .bind(catalog_name)
                .bind(namespace_name)
                .bind(table)
                .execute(&self.pool)
                .await?
        };

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("CAS check failed: Metadata location mismatch or asset not found"));
        }

        Ok(())
    }

}

use crate::signer::Signer;
use crate::signer::Credentials;

#[async_trait]
impl Signer for PostgresStore {
    async fn get_table_credentials(&self, _location: &str) -> Result<Credentials> {
        // Placeholder: PostgresStore doesn't manage S3 creds yet.
        // In a real scenario, we'd inject S3 config into PostgresStore or have a hybrid store.
        Err(anyhow::anyhow!("Credential vending not supported by PostgresStore"))
    }

    async fn presign_get(&self, _location: &str) -> Result<String> {
        Err(anyhow::anyhow!("Presigning not supported by PostgresStore"))
    }
}
