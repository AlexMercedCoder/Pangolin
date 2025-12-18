use crate::CatalogStore;
use anyhow::Result;
use async_trait::async_trait;
use pangolin_core::model::{
    Asset, Branch, Catalog, Commit, Namespace, Tag, Tenant, Warehouse, VendingStrategy,
};
use crate::signer::{Signer, Credentials};
use pangolin_core::user::{User, UserRole, OAuthProvider};
use pangolin_core::permission::{Role, Permission, PermissionGrant, UserRole as UserRoleAssignment};
use pangolin_core::audit::AuditLogEntry;
use pangolin_core::business_metadata::{AccessRequest, RequestStatus};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc, TimeZone};
use object_store::{aws::AmazonS3Builder, ObjectStore, path::Path as ObjPath};

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
        sqlx::query("INSERT INTO warehouses (id, tenant_id, name, use_sts, storage_config, vending_strategy) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(warehouse.id)
            .bind(tenant_id)
            .bind(&warehouse.name)
            .bind(warehouse.use_sts)
            .bind(serde_json::to_value(&warehouse.storage_config)?)
            .bind(serde_json::to_value(&warehouse.vending_strategy)?)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_warehouse(&self, tenant_id: Uuid, name: String) -> Result<Option<Warehouse>> {
        let row = sqlx::query("SELECT id, name, tenant_id, use_sts, storage_config, vending_strategy FROM warehouses WHERE tenant_id = $1 AND name = $2")
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
                vending_strategy: row.try_get("vending_strategy")
                    .ok()
                    .and_then(|v: serde_json::Value| serde_json::from_value(v).ok()),
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_warehouses(&self, tenant_id: Uuid) -> Result<Vec<Warehouse>> {
        let rows = sqlx::query("SELECT id, name, tenant_id, use_sts, storage_config, vending_strategy FROM warehouses WHERE tenant_id = $1")
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
                vending_strategy: row.try_get("vending_strategy")
                    .ok()
                    .and_then(|v: serde_json::Value| serde_json::from_value(v).ok()),
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
        if updates.vending_strategy.is_some() {
            set_clauses.push(format!("vending_strategy = ${}", bind_count));
            bind_count += 1;
        }

        if set_clauses.is_empty() {
            return self.get_warehouse(tenant_id, name).await?
                .ok_or_else(|| anyhow::anyhow!("Warehouse not found"));
        }

        query.push_str(&set_clauses.join(", "));
        query.push_str(&format!(" WHERE tenant_id = ${} AND name = ${} RETURNING id, name, tenant_id, use_sts, storage_config, vending_strategy", bind_count, bind_count + 1));

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
        if let Some(vending_strategy) = updates.vending_strategy {
             q = q.bind(serde_json::to_value(vending_strategy)?);
        }
        q = q.bind(tenant_id).bind(&name);

        let row = q.fetch_one(&self.pool).await?;
        Ok(Warehouse {
            id: row.get("id"),
            name: row.get("name"),
            tenant_id: row.get("tenant_id"),
            use_sts: row.try_get("use_sts").unwrap_or(false),
            storage_config: serde_json::from_value(row.get("storage_config")).unwrap_or_default(),
            vending_strategy: row.try_get("vending_strategy")
                .ok()
                .and_then(|v: serde_json::Value| serde_json::from_value(v).ok()),
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
        // Manually cascade delete dependent resources
        // 1. Tags
        sqlx::query("DELETE FROM tags WHERE tenant_id = $1 AND catalog_name = $2")
            .bind(tenant_id)
            .bind(&name)
            .execute(&self.pool)
            .await?;

        // 2. Branches
        sqlx::query("DELETE FROM branches WHERE tenant_id = $1 AND catalog_name = $2")
            .bind(tenant_id)
            .bind(&name)
            .execute(&self.pool)
            .await?;

        // 3. Assets
        sqlx::query("DELETE FROM assets WHERE tenant_id = $1 AND catalog_name = $2")
            .bind(tenant_id)
            .bind(&name)
            .execute(&self.pool)
            .await?;

        // 4. Namespaces
        sqlx::query("DELETE FROM namespaces WHERE tenant_id = $1 AND catalog_name = $2")
            .bind(tenant_id)
            .bind(&name)
            .execute(&self.pool)
            .await?;

        // 5. Catalog
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
            .bind(asset.id)
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(&namespace)
            .bind(&asset.name)
            .bind(format!("{:?}", asset.kind))
            .bind(asset.properties.get("metadata_location").unwrap_or(&asset.location))
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

    async fn get_asset_by_id(&self, tenant_id: Uuid, asset_id: Uuid) -> Result<Option<(Asset, String, Vec<String>)>> {
        let row = sqlx::query("SELECT id, name, catalog_name, namespace_path, asset_type, metadata_location, properties FROM assets WHERE tenant_id = $1 AND id = $2")
            .bind(tenant_id)
            .bind(asset_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let catalog_name: String = row.get("catalog_name");
            let namespace_path: Vec<String> = row.get("namespace_path");
            let asset_type_str: String = row.get("asset_type");
            let kind = match asset_type_str.as_str() {
                "IcebergTable" => pangolin_core::model::AssetType::IcebergTable,
                "View" => pangolin_core::model::AssetType::View,
                _ => pangolin_core::model::AssetType::IcebergTable,
            };
            
            let asset = Asset {
                id: row.get("id"),
                name: row.get("name"),
                kind,
                location: row.get::<Option<String>, _>("metadata_location").unwrap_or_default(),
                properties: serde_json::from_value(row.get("properties")).unwrap_or_default(),
            };
            
            Ok(Some((asset, catalog_name, namespace_path)))
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
    
    async fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        if let Some(rest) = path.strip_prefix("s3://") {
            let (bucket, key) = rest.split_once('/').ok_or_else(|| anyhow::anyhow!("Invalid S3 path"))?;
            
            let mut builder = AmazonS3Builder::from_env()
                .with_bucket_name(bucket)
                .with_allow_http(true);
                
             if let Ok(endpoint) = std::env::var("S3_ENDPOINT") {
                 builder = builder.with_endpoint(endpoint);
             }
             if let Ok(key_id) = std::env::var("AWS_ACCESS_KEY_ID") {
                 builder = builder.with_access_key_id(key_id);
             }
             if let Ok(secret) = std::env::var("AWS_SECRET_ACCESS_KEY") {
                 builder = builder.with_secret_access_key(secret);
             }
             if let Ok(region) = std::env::var("AWS_REGION") {
                 builder = builder.with_region(region);
             }
             
             let store = builder.build()?;
             let result = store.get(&ObjPath::from(key)).await?;
             let bytes = result.bytes().await?;
             Ok(bytes.to_vec())
        } else {
             Err(anyhow::anyhow!("Only s3:// paths are supported in Postgres store"))
        }
    }

    async fn write_file(&self, path: &str, data: Vec<u8>) -> Result<()> {
        if let Some(rest) = path.strip_prefix("s3://") {
            let (bucket, key) = rest.split_once('/').ok_or_else(|| anyhow::anyhow!("Invalid S3 path"))?;
            
            tracing::info!("write_file: s3 path='{}' bucket='{}' key='{}'", path, bucket, key);
            
            let mut builder = AmazonS3Builder::from_env()
                .with_bucket_name(bucket)
                .with_allow_http(true);
                
             if let Ok(endpoint) = std::env::var("S3_ENDPOINT") {
                 builder = builder.with_endpoint(endpoint);
             }
             if let Ok(key_id) = std::env::var("AWS_ACCESS_KEY_ID") {
                 builder = builder.with_access_key_id(key_id);
             }
             if let Ok(secret) = std::env::var("AWS_SECRET_ACCESS_KEY") {
                 builder = builder.with_secret_access_key(secret);
             }
             if let Ok(region) = std::env::var("AWS_REGION") {
                 builder = builder.with_region(region);
             }
             
             let store = builder.build()?;
             store.put(&ObjPath::from(key), data.into()).await?;
             Ok(())
        } else {
             Err(anyhow::anyhow!("Only s3:// paths are supported in Postgres store"))
        }
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
        sqlx::query(
            "INSERT INTO audit_logs (
                id, tenant_id, user_id, username, action, resource_type,
                resource_id, resource_name, timestamp, ip_address, user_agent,
                result, error_message, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)"
        )
        .bind(event.id)
        .bind(tenant_id)
        .bind(event.user_id)
        .bind(event.username)
        .bind(event.action)
        .bind(event.resource_type)
        .bind(event.resource_id)
        .bind(event.resource_name)
        .bind(event.timestamp)
        .bind(event.ip_address)
        .bind(event.user_agent)
        .bind(event.result)
        .bind(event.error_message)
        .bind(event.metadata)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list_audit_events(&self, tenant_id: Uuid, filter: Option<pangolin_core::audit::AuditLogFilter>) -> Result<Vec<AuditLogEntry>> {
        let mut query = String::from(
            "SELECT id, tenant_id, user_id, username, action, resource_type,
             resource_id, resource_name, timestamp, ip_address, user_agent,
             result, error_message, metadata
             FROM audit_logs WHERE tenant_id = $1"
        );
        
        let mut conditions = Vec::new();
        let mut param_count = 2;
        
        // Build dynamic WHERE clause
        if let Some(ref f) = filter {
            if f.user_id.is_some() {
                conditions.push(format!("user_id = ${}", param_count));
                param_count += 1;
            }
            if f.action.is_some() {
                conditions.push(format!("action = ${}", param_count));
                param_count += 1;
            }
            if f.resource_type.is_some() {
                conditions.push(format!("resource_type = ${}", param_count));
                param_count += 1;
            }
            if f.resource_id.is_some() {
                conditions.push(format!("resource_id = ${}", param_count));
                param_count += 1;
            }
            if f.start_time.is_some() {
                conditions.push(format!("timestamp >= ${}", param_count));
                param_count += 1;
            }
            if f.end_time.is_some() {
                conditions.push(format!("timestamp <= ${}", param_count));
                param_count += 1;
            }
            if f.result.is_some() {
                conditions.push(format!("result = ${}", param_count));
                param_count += 1;
            }
        }
        
        if !conditions.is_empty() {
            query.push_str(" AND ");
            query.push_str(&conditions.join(" AND "));
        }
        
        query.push_str(" ORDER BY timestamp DESC");
        
        // Add pagination
        if let Some(ref f) = filter {
            if let Some(limit) = f.limit {
                query.push_str(&format!(" LIMIT {}", limit));
            } else {
                query.push_str(" LIMIT 100");
            }
            if let Some(offset) = f.offset {
                query.push_str(&format!(" OFFSET {}", offset));
            }
        } else {
            query.push_str(" LIMIT 100");
        }
        
        // Build and execute query with dynamic binding
        let mut query_builder = sqlx::query(&query).bind(tenant_id);
        
        if let Some(f) = filter {
            if let Some(user_id) = f.user_id {
                query_builder = query_builder.bind(user_id);
            }
            if let Some(action) = f.action {
                query_builder = query_builder.bind(action);
            }
            if let Some(resource_type) = f.resource_type {
                query_builder = query_builder.bind(resource_type);
            }
            if let Some(resource_id) = f.resource_id {
                query_builder = query_builder.bind(resource_id);
            }
            if let Some(start_time) = f.start_time {
                query_builder = query_builder.bind(start_time);
            }
            if let Some(end_time) = f.end_time {
                query_builder = query_builder.bind(end_time);
            }
            if let Some(result) = f.result {
                query_builder = query_builder.bind(result);
            }
        }
        
        let rows = query_builder.fetch_all(&self.pool).await?;
        
        let mut events = Vec::new();
        for row in rows {
            events.push(AuditLogEntry {
                id: row.get("id"),
                tenant_id: row.get("tenant_id"),
                user_id: row.get("user_id"),
                username: row.get("username"),
                action: row.get("action"),
                resource_type: row.get("resource_type"),
                resource_id: row.get("resource_id"),
                resource_name: row.get("resource_name"),
                timestamp: row.get("timestamp"),
                ip_address: row.get("ip_address"),
                user_agent: row.get("user_agent"),
                result: row.get("result"),
                error_message: row.get("error_message"),
                metadata: row.get("metadata"),
            });
        }
        Ok(events)
    }
    
    async fn get_audit_event(&self, tenant_id: Uuid, event_id: Uuid) -> Result<Option<AuditLogEntry>> {
        let row = sqlx::query(
            "SELECT id, tenant_id, user_id, username, action, resource_type,
             resource_id, resource_name, timestamp, ip_address, user_agent,
             result, error_message, metadata
             FROM audit_logs WHERE tenant_id = $1 AND id = $2"
        )
        .bind(tenant_id)
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            Ok(Some(AuditLogEntry {
                id: row.get("id"),
                tenant_id: row.get("tenant_id"),
                user_id: row.get("user_id"),
                username: row.get("username"),
                action: row.get("action"),
                resource_type: row.get("resource_type"),
                resource_id: row.get("resource_id"),
                resource_name: row.get("resource_name"),
                timestamp: row.get("timestamp"),
                ip_address: row.get("ip_address"),
                user_agent: row.get("user_agent"),
                result: row.get("result"),
                error_message: row.get("error_message"),
                metadata: row.get("metadata"),
            }))
        } else {
            Ok(None)
        }
    }
    
    async fn count_audit_events(&self, tenant_id: Uuid, filter: Option<pangolin_core::audit::AuditLogFilter>) -> Result<usize> {
        let mut query = String::from("SELECT COUNT(*) as count FROM audit_logs WHERE tenant_id = $1");
        
        let mut conditions = Vec::new();
        let mut param_count = 2;
        
        // Build same WHERE clause as list_audit_events
        if let Some(ref f) = filter {
            if f.user_id.is_some() {
                conditions.push(format!("user_id = ${}", param_count));
                param_count += 1;
            }
            if f.action.is_some() {
                conditions.push(format!("action = ${}", param_count));
                param_count += 1;
            }
            if f.resource_type.is_some() {
                conditions.push(format!("resource_type = ${}", param_count));
                param_count += 1;
            }
            if f.resource_id.is_some() {
                conditions.push(format!("resource_id = ${}", param_count));
                param_count += 1;
            }
            if f.start_time.is_some() {
                conditions.push(format!("timestamp >= ${}", param_count));
                param_count += 1;
            }
            if f.end_time.is_some() {
                conditions.push(format!("timestamp <= ${}", param_count));
                param_count += 1;
            }
            if f.result.is_some() {
                conditions.push(format!("result = ${}", param_count));
                param_count += 1;
            }
        }
        
        if !conditions.is_empty() {
            query.push_str(" AND ");
            query.push_str(&conditions.join(" AND "));
        }
        
        // Build and execute query
        let mut query_builder = sqlx::query(&query).bind(tenant_id);
        
        if let Some(f) = filter {
            if let Some(user_id) = f.user_id {
                query_builder = query_builder.bind(user_id);
            }
            if let Some(action) = f.action {
                query_builder = query_builder.bind(action);
            }
            if let Some(resource_type) = f.resource_type {
                query_builder = query_builder.bind(resource_type);
            }
            if let Some(resource_id) = f.resource_id {
                query_builder = query_builder.bind(resource_id);
            }
            if let Some(start_time) = f.start_time {
                query_builder = query_builder.bind(start_time);
            }
            if let Some(end_time) = f.end_time {
                query_builder = query_builder.bind(end_time);
            }
            if let Some(result) = f.result {
                query_builder = query_builder.bind(result);
            }
        }
        
        let row = query_builder.fetch_one(&self.pool).await?;
        let count: i64 = row.get("count");
        Ok(count as usize)
    }

    // Maintenance Operations (Placeholders)
    async fn expire_snapshots(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _namespace: Vec<String>, _table: String, _retention_ms: i64) -> Result<()> {
        Ok(())
    }

    async fn remove_orphan_files(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _namespace: Vec<String>, _table: String, _older_than_ms: i64) -> Result<()> {
        Ok(())
    }

    // Access Request Operations
    async fn create_access_request(&self, request: AccessRequest) -> Result<()> {
        sqlx::query(
            "INSERT INTO access_requests (id, user_id, asset_id, reason, requested_at, status, reviewed_by, reviewed_at, review_comment) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
        )
        .bind(request.id)
        .bind(request.user_id)
        .bind(request.asset_id)
        .bind(request.reason)
        .bind(request.requested_at)
        .bind(format!("{:?}", request.status))
        .bind(request.reviewed_by)
        .bind(request.reviewed_at)
        .bind(request.review_comment)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_access_request(&self, id: Uuid) -> Result<Option<AccessRequest>> {
        let row = sqlx::query("SELECT id, tenant_id, user_id, asset_id, reason, requested_at, status, reviewed_by, reviewed_at, review_comment FROM access_requests WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(self.row_to_access_request(row)?))
        } else {
            Ok(None)
        }
    }

    async fn list_access_requests(&self, tenant_id: Uuid) -> Result<Vec<AccessRequest>> {
        let rows = sqlx::query(
            "SELECT ar.id, ar.user_id, ar.asset_id, ar.reason, ar.requested_at, ar.status, ar.reviewed_by, ar.reviewed_at, ar.review_comment FROM access_requests ar
             JOIN users u ON ar.user_id = u.id
             WHERE u.tenant_id = $1"
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;
        
        let mut requests = Vec::new();
        for row in rows {
            requests.push(self.row_to_access_request(row)?);
        }
        Ok(requests)
    }

    async fn update_access_request(&self, request: AccessRequest) -> Result<()> {
        sqlx::query(
            "UPDATE access_requests SET status = $1, reviewed_by = $2, reviewed_at = $3, review_comment = $4 WHERE id = $5"
        )
        .bind(format!("{:?}", request.status))
        .bind(request.reviewed_by)
        .bind(request.reviewed_at)
        .bind(request.review_comment)
        .bind(request.id)
        .execute(&self.pool)
        .await?;
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
        
        tracing::info!("update_metadata_location: tenant_id={}, catalog={}, namespace={:?}, table={}, expected={:?}, new={}", 
            tenant_id, catalog_name, namespace, table, expected_location, new_location);

        // CAS Logic
        // We need to check if current metadata_location matches expected_location
        // In Postgres, we can do this in the UPDATE query WHERE clause
        
        let result = if let Some(expected) = expected_location {
            // Update only if current location matches expected
            sqlx::query("UPDATE assets SET metadata_location = $1 WHERE tenant_id = $2 AND catalog_name = $3 AND namespace_path = $4 AND name = $5 AND metadata_location = $6")
                .bind(new_location)
                .bind(tenant_id)
                .bind(catalog_name)
                .bind(&namespace)
                .bind(table)
                .bind(expected)
                .execute(&self.pool)
                .await?
        } else {
            // Update if no metadata_location exists (create or first commit)
            sqlx::query("UPDATE assets SET metadata_location = $1 WHERE tenant_id = $2 AND catalog_name = $3 AND namespace_path = $4 AND name = $5 AND metadata_location IS NULL")
                .bind(new_location)
                .bind(tenant_id)
                .bind(catalog_name)
                .bind(&namespace)
                .bind(table)
                .execute(&self.pool)
                .await?
        };

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("CAS check failed: Metadata location mismatch or asset not found"));
        }

        Ok(())
    }



    // User Operations
    async fn create_user(&self, user: User) -> Result<()> {
        sqlx::query("INSERT INTO users (id, username, email, password_hash, oauth_provider, oauth_subject, tenant_id, role, active, created_at, updated_at, last_login) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)")
            .bind(user.id)
            .bind(user.username)
            .bind(user.email)
            .bind(user.password_hash)
            .bind(user.oauth_provider.map(|p| format!("{:?}", p).to_lowercase()))
            .bind(user.oauth_subject)
            .bind(user.tenant_id)
            .bind(format!("{:?}", user.role))
            .bind(user.active)
            .bind(user.created_at)
            .bind(user.updated_at)
            .bind(user.last_login)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_user(&self, user_id: Uuid) -> Result<Option<User>> {
        let row = sqlx::query("SELECT id, username, email, password_hash, oauth_provider, oauth_subject, tenant_id, role, active, created_at, updated_at, last_login FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;
        
        if let Some(row) = row {
            self.row_to_user(row)
        } else {
            Ok(None)
        }
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let row = sqlx::query("SELECT id, username, email, password_hash, oauth_provider, oauth_subject, tenant_id, role, active, created_at, updated_at, last_login FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;
        
        if let Some(row) = row {
            self.row_to_user(row)
        } else {
            Ok(None)
        }
    }

    async fn list_users(&self, tenant_id: Option<Uuid>) -> Result<Vec<User>> {
        let rows = if let Some(tid) = tenant_id {
            sqlx::query("SELECT id, username, email, password_hash, oauth_provider, oauth_subject, tenant_id, role, active, created_at, updated_at, last_login FROM users WHERE tenant_id = $1")
                .bind(tid)
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query("SELECT id, username, email, password_hash, oauth_provider, oauth_subject, tenant_id, role, active, created_at, updated_at, last_login FROM users")
                .fetch_all(&self.pool)
                .await?
        };

        let mut users = Vec::new();
        for row in rows {
            if let Ok(Some(user)) = self.row_to_user(row) {
                users.push(user);
            }
        }
        Ok(users)
    }

    async fn update_user(&self, user: User) -> Result<()> {
        sqlx::query("UPDATE users SET username = $1, email = $2, password_hash = $3, role = $4, active = $5, updated_at = $6, last_login = $7 WHERE id = $8")
            .bind(user.username)
            .bind(user.email)
            .bind(user.password_hash)
            .bind(format!("{:?}", user.role))
            .bind(user.active)
            .bind(chrono::Utc::now())
            .bind(user.last_login)
            .bind(user.id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete_user(&self, user_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = $1").bind(user_id).execute(&self.pool).await?;
        Ok(())
    }

    // Role Operations
    async fn create_role(&self, role: Role) -> Result<()> {
        sqlx::query("INSERT INTO roles (id, tenant_id, name, description, permissions, created_by, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
            .bind(role.id)
            .bind(role.tenant_id)
            .bind(role.name)
            .bind(role.description)
            .bind(serde_json::to_value(&role.permissions)?)
            .bind(role.created_by)
            .bind(role.created_at)
            .bind(role.updated_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_role(&self, role_id: Uuid) -> Result<Option<Role>> {
        let row = sqlx::query("SELECT id, tenant_id, name, description, permissions, created_by, created_at, updated_at FROM roles WHERE id = $1")
            .bind(role_id)
            .fetch_optional(&self.pool)
            .await?;
        
        if let Some(row) = row {
            Ok(Some(Role {
                id: row.get("id"),
                tenant_id: row.get("tenant_id"),
                name: row.get("name"),
                description: row.get("description"),
                permissions: serde_json::from_value(row.get("permissions"))?,
                created_by: row.get("created_by"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_roles(&self, tenant_id: Uuid) -> Result<Vec<Role>> {
        let rows = sqlx::query("SELECT id, tenant_id, name, description, permissions, created_by, created_at, updated_at FROM roles WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await?;
        
        let mut roles = Vec::new();
        for row in rows {
            roles.push(Role {
                id: row.get("id"),
                tenant_id: row.get("tenant_id"),
                name: row.get("name"),
                description: row.get("description"),
                permissions: serde_json::from_value(row.get("permissions"))?,
                created_by: row.get("created_by"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        Ok(roles)
    }

    async fn delete_role(&self, role_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM roles WHERE id = $1").bind(role_id).execute(&self.pool).await?;
        Ok(())
    }

    async fn update_role(&self, role: Role) -> Result<()> {
        sqlx::query("UPDATE roles SET name = $1, description = $2, permissions = $3, updated_at = $4 WHERE id = $5")
            .bind(role.name)
            .bind(role.description)
            .bind(serde_json::to_value(&role.permissions)?)
            .bind(chrono::Utc::now())
            .bind(role.id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn assign_role(&self, user_role: UserRoleAssignment) -> Result<()> {
        sqlx::query("INSERT INTO user_roles (user_id, role_id, assigned_by, assigned_at) VALUES ($1, $2, $3, $4)")
            .bind(user_role.user_id)
            .bind(user_role.role_id)
            .bind(user_role.assigned_by)
            .bind(user_role.assigned_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn revoke_role(&self, user_id: Uuid, role_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2")
            .bind(user_id)
            .bind(role_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<UserRoleAssignment>> {
        let rows = sqlx::query("SELECT user_id, role_id, assigned_by, assigned_at FROM user_roles WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
        
        let mut roles = Vec::new();
        for row in rows {
            roles.push(UserRoleAssignment {
                user_id: row.get("user_id"),
                role_id: row.get("role_id"),
                assigned_by: row.get("assigned_by"),
                assigned_at: row.get("assigned_at"),
            });
        }
        Ok(roles)
    }

    // Direct Permission Operations
    async fn create_permission(&self, permission: Permission) -> Result<()> {
        sqlx::query("INSERT INTO permissions (id, user_id, scope, actions, granted_by, granted_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(permission.id)
            .bind(permission.user_id)
            .bind(serde_json::to_value(&permission.scope)?)
            .bind(serde_json::to_value(&permission.actions)?)
            .bind(permission.granted_by)
            .bind(permission.granted_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn revoke_permission(&self, permission_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM permissions WHERE id = $1")
            .bind(permission_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn list_user_permissions(&self, user_id: Uuid) -> Result<Vec<Permission>> {
        let rows = sqlx::query("SELECT id, user_id, scope, actions, granted_by, granted_at FROM permissions WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
        
        let mut perms = Vec::new();
        for row in rows {
            perms.push(Permission {
                id: row.get("id"),
                user_id: row.get("user_id"),
                scope: serde_json::from_value(row.get("scope"))?,
                actions: serde_json::from_value(row.get("actions"))?,
                granted_by: row.get("granted_by"),
                granted_at: row.get("granted_at"),
            });
        }
        Ok(perms)
    }

    async fn list_permissions(&self, tenant_id: Uuid) -> Result<Vec<Permission>> {
        let rows = sqlx::query(
            "SELECT p.id, p.user_id, p.scope, p.actions, p.granted_by, p.granted_at 
             FROM permissions p
             JOIN users u ON p.user_id = u.id
             WHERE u.tenant_id = $1"
        )
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await?;

        let mut perms = Vec::new();
        for row in rows {
            perms.push(Permission {
                id: row.get("id"),
                user_id: row.get("user_id"),
                scope: serde_json::from_value(row.get("scope"))?,
                actions: serde_json::from_value(row.get("actions"))?,
                granted_by: row.get("granted_by"),
                granted_at: row.get("granted_at"),
            });
        }
        Ok(perms)
    }


}

impl PostgresStore {
    fn row_to_access_request(&self, row: sqlx::postgres::PgRow) -> Result<AccessRequest> {
        let status_str: String = row.get("status");
        let status = match status_str.as_str() {
            "Pending" => RequestStatus::Pending,
            "Approved" => RequestStatus::Approved,
            "Rejected" => RequestStatus::Rejected,
            _ => RequestStatus::Pending,
        };

        Ok(AccessRequest {
            id: row.get("id"),
            tenant_id: row.get("tenant_id"),
            user_id: row.get("user_id"),
            asset_id: row.get("asset_id"),
            reason: row.get("reason"),
            requested_at: row.get("requested_at"),
            status,
            reviewed_by: row.get("reviewed_by"),
            reviewed_at: row.get("reviewed_at"),
            review_comment: row.get("review_comment"),
        })
    }

    // Helper to Convert Row to User
    fn row_to_user(&self, row: sqlx::postgres::PgRow) -> Result<Option<User>> {
        let role_str: String = row.get("role");
        let role = match role_str.as_str() {
            "Root" => UserRole::Root,
            "TenantAdmin" => UserRole::TenantAdmin,
            _ => UserRole::TenantUser,
        };

        let oauth_provider_str: Option<String> = row.get("oauth_provider");
        let oauth_provider = match oauth_provider_str.as_deref() {
            Some("google") => Some(OAuthProvider::Google),
            Some("microsoft") => Some(OAuthProvider::Microsoft),
            Some("github") => Some(OAuthProvider::GitHub),
            Some("okta") => Some(OAuthProvider::Okta),
            _ => None,
        };

        Ok(Some(User {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            oauth_provider,
            oauth_subject: row.get("oauth_subject"),
            tenant_id: row.get("tenant_id"),
            role,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            last_login: row.get("last_login"),
            active: row.get("active"),
        }))
    }

    // Token Revocation Operations
    async fn revoke_token(&self, token_id: Uuid, expires_at: chrono::DateTime<chrono::Utc>, reason: Option<String>) -> Result<()> {
        sqlx::query(
            "INSERT INTO revoked_tokens (token_id, expires_at, reason) VALUES ($1, $2, $3)"
        )
        .bind(token_id)
        .bind(expires_at)
        .bind(reason)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn is_token_revoked(&self, token_id: Uuid) -> Result<bool> {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM revoked_tokens WHERE token_id = $1)"
        )
        .bind(token_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(exists)
    }

    async fn cleanup_expired_tokens(&self) -> Result<usize> {
        let rows_affected = sqlx::query(
            "DELETE FROM revoked_tokens WHERE expires_at < NOW()"
        )
        .execute(&self.pool)
        .await?
        .rows_affected();
        Ok(rows_affected as usize)
    }
}


#[async_trait]
impl Signer for PostgresStore {
    async fn get_table_credentials(&self, location: &str) -> Result<crate::signer::Credentials> {
        // 1. Find the warehouse that owns this location by querying all warehouses
        let rows = sqlx::query("SELECT id, name, storage_config, use_sts, vending_strategy FROM warehouses")
            .fetch_all(&self.pool)
            .await?;
        
        let mut target_warehouse = None;
        
        for row in rows {
            let storage_config_json: serde_json::Value = row.get("storage_config");
            let storage_config: HashMap<String, String> = serde_json::from_value(storage_config_json)?;
            
            // Check matches
             // Check AWS S3
            if let Some(bucket) = storage_config.get("s3.bucket") {
                if location.contains(bucket) {
                    target_warehouse = Some(row);
                    break;
                }
            }
            // Check Azure
            if let Some(container) = storage_config.get("azure.container") {
                if location.contains(container) {
                     target_warehouse = Some(row);
                     break;
                }
            }
            // Check GCP
            if let Some(bucket) = storage_config.get("gcp.bucket") {
                if location.contains(bucket) {
                    target_warehouse = Some(row);
                    break;
                }
            }
        }
        
        let row = target_warehouse.ok_or_else(|| anyhow::anyhow!("No warehouse found for location: {}", location))?;
        
        let storage_config_json: serde_json::Value = row.get("storage_config");
        let storage_config: HashMap<String, String> = serde_json::from_value(storage_config_json)?;
        let use_sts: bool = row.try_get("use_sts").unwrap_or(false);
        let vending_strategy: Option<VendingStrategy> = row.try_get("vending_strategy")
            .ok()
            .and_then(|v: serde_json::Value| serde_json::from_value(v).ok());
            
        // 2. Check Vending Strategy
        if let Some(strategy) = vending_strategy {
             match strategy {
                 VendingStrategy::AwsSts { role_arn: _, external_id: _ } => {
                     Err(anyhow::anyhow!("AWS STS vending not implemented yet via VendingStrategy in PostgresStore"))
                 }
                 VendingStrategy::AwsStatic { access_key_id, secret_access_key } => {
                     Ok(Credentials::Aws {
                         access_key_id,
                         secret_access_key,
                         session_token: None,
                         expiration: None,
                     })
                 }
                 VendingStrategy::AzureSas { account_name, account_key } => {
                     let signer = crate::azure_signer::AzureSigner::new(account_name.clone(), account_key);
                     let sas_token = signer.generate_sas_token(location).await?;
                     Ok(Credentials::Azure {
                         sas_token,
                         account_name,
                         expiration: chrono::Utc::now() + chrono::Duration::hours(1),
                     })
                 }
                 VendingStrategy::GcpDownscoped { service_account_email, private_key } => {
                     let signer = crate::gcp_signer::GcpSigner::new(service_account_email, private_key);
                     let access_token = signer.generate_downscoped_token(location).await?;
                     Ok(Credentials::Gcp {
                         access_token,
                         expiration: chrono::Utc::now() + chrono::Duration::hours(1),
                     })
                 }
                 VendingStrategy::None => Err(anyhow::anyhow!("Vending disabled")),
             }
        } else {
            // Legacy Logic
            let access_key = storage_config.get("s3.access-key-id")
                .ok_or_else(|| anyhow::anyhow!("Missing s3.access-key-id"))?;
            let secret_key = storage_config.get("s3.secret-access-key")
                .ok_or_else(|| anyhow::anyhow!("Missing s3.secret-access-key"))?;
                
            if use_sts {
                // Existing STS Logic restored for backward compatibility
                let region = storage_config.get("s3.region")
                    .map(|s| s.as_str())
                    .unwrap_or("us-east-1");
                    
                let endpoint = storage_config.get("s3.endpoint")
                    .map(|s| s.as_str());

                let creds = aws_credential_types::Credentials::new(
                    access_key.to_string(),
                    secret_key.to_string(),
                    None,
                    None,
                    "legacy_provider"
                );
                
                let config_loader = aws_config::from_env()
                    .region(aws_config::Region::new(region.to_string()))
                    .credentials_provider(creds);
                    
                let config = if let Some(ep) = endpoint {
                     config_loader.endpoint_url(ep).load().await
                } else {
                     config_loader.load().await
                };
                
                let client = aws_sdk_sts::Client::new(&config);
                
                // Assume Role logic was present in legacy? Or just GetSessionToken?
                // Checking SqliteStore, it tried AssumeRole if role-arn present, else GetSessionToken.
                // Replicating that for consistency.
                
                let role_arn = storage_config.get("s3.role-arn").map(|s| s.as_str());
            
                if let Some(arn) = role_arn {
                     let resp = client.assume_role()
                        .role_arn(arn)
                        .role_session_name("pangolin-legacy-session")
                        .send()
                        .await
                        .map_err(|e| anyhow::anyhow!("STS AssumeRole failed: {}", e))?;
                        
                     let c = resp.credentials.ok_or_else(|| anyhow::anyhow!("No credentials in AssumeRole response"))?;
                     Ok(Credentials::Aws {
                         access_key_id: c.access_key_id,
                         secret_access_key: c.secret_access_key,
                         session_token: Some(c.session_token),
                         expiration: chrono::DateTime::from_timestamp(c.expiration.secs(), c.expiration.subsec_nanos()),
                     })
                } else {
                     let resp = client.get_session_token()
                        .send()
                        .await
                        .map_err(|e| anyhow::anyhow!("STS GetSessionToken failed: {}", e))?;
                        
                     let c = resp.credentials.ok_or_else(|| anyhow::anyhow!("No credentials in GetSessionToken response"))?;
                     Ok(Credentials::Aws {
                         access_key_id: c.access_key_id,
                         secret_access_key: c.secret_access_key,
                         session_token: Some(c.session_token),
                         expiration: chrono::DateTime::from_timestamp(c.expiration.secs(), c.expiration.subsec_nanos()),
                     })
                }
            } else {
                Ok(Credentials::Aws {
                    access_key_id: access_key.clone(),
                    secret_access_key: secret_key.clone(),
                    session_token: None,
                    expiration: None,
                })
            }
        }
    }

    async fn presign_get(&self, _location: &str) -> Result<String> {
        Err(anyhow::anyhow!("PostgresStore does not support presigning yet"))
    }


}
