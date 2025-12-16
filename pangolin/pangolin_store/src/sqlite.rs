use async_trait::async_trait;
use crate::CatalogStore;
use pangolin_core::model::*;
use pangolin_core::user::{User, UserRole, OAuthProvider};
use pangolin_core::permission::{Role, Permission, PermissionGrant, UserRole as UserRoleAssignment};
use pangolin_core::audit::AuditLogEntry;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;
use anyhow::Result;
use chrono::{DateTime, Utc, TimeZone};
use pangolin_core::business_metadata::{AccessRequest, RequestStatus};

use crate::signer::Signer;

#[derive(Clone)]
pub struct SqliteStore {
    pub(crate) pool: SqlitePool,
}

impl SqliteStore {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        
        // Enable foreign keys
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await?;
        
        Ok(Self { pool })
    }
    
    pub async fn apply_schema(&self, schema_sql: &str) -> Result<()> {
        // Disable foreign keys during schema creation
        sqlx::query("PRAGMA foreign_keys = OFF")
            .execute(&self.pool)
            .await?;
        
        // Parse statements more carefully - don't split on ; inside parentheses
        let mut statements = Vec::new();
        let mut current_statement = String::new();
        let mut paren_depth = 0;
        
        for line in schema_sql.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("--") {
                continue;
            }
            
            for ch in line.chars() {
                match ch {
                    '(' => paren_depth += 1,
                    ')' => paren_depth -= 1,
                    ';' if paren_depth == 0 => {
                        if !current_statement.trim().is_empty() {
                            statements.push(current_statement.trim().to_string());
                            current_statement.clear();
                        }
                        continue;
                    }
                    _ => {}
                }
                current_statement.push(ch);
            }
            current_statement.push('\n');
        }
        
        // Add any remaining statement
        if !current_statement.trim().is_empty() {
            statements.push(current_statement.trim().to_string());
        }
        
        // Execute each statement
        for statement in statements {
            sqlx::query(&statement)
                .execute(&self.pool)
                .await?;
        }
        
        // Re-enable foreign keys
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
}

#[async_trait]
impl Signer for SqliteStore {
    async fn get_table_credentials(&self, _location: &str) -> anyhow::Result<crate::signer::Credentials> {
        Err(anyhow::anyhow!("Credential vending not supported in SQLite store"))
    }

    async fn presign_get(&self, _location: &str) -> anyhow::Result<String> {
        Err(anyhow::anyhow!("URL signing not supported in SQLite store"))
    }
}

#[async_trait]
impl CatalogStore for SqliteStore {
    // Tenant Operations
    async fn create_tenant(&self, tenant: Tenant) -> Result<()> {
        sqlx::query("INSERT INTO tenants (id, name, properties) VALUES (?, ?, ?)")
            .bind(tenant.id.to_string())
            .bind(&tenant.name)
            .bind(serde_json::to_string(&tenant.properties)?)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_tenant(&self, id: Uuid) -> Result<Option<Tenant>> {
        let row = sqlx::query("SELECT id, name, properties FROM tenants WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(Tenant {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                name: row.get("name"),
                properties: serde_json::from_str(&row.get::<String, _>("properties"))?,
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
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                name: row.get("name"),
                properties: serde_json::from_str(&row.get::<String, _>("properties"))?,
            });
        }
        Ok(tenants)
    }

    async fn update_tenant(&self, tenant_id: Uuid, updates: pangolin_core::model::TenantUpdate) -> Result<Tenant> {
        let mut query = String::from("UPDATE tenants SET ");
        let mut set_clauses = Vec::new();
        
        if updates.name.is_some() {
            set_clauses.push("name = ?");
        }
        if updates.properties.is_some() {
            set_clauses.push("properties = ?");
        }
        
        if set_clauses.is_empty() {
            return self.get_tenant(tenant_id).await?
                .ok_or_else(|| anyhow::anyhow!("Tenant not found"));
        }
        
        query.push_str(&set_clauses.join(", "));
        query.push_str(" WHERE id = ?");
        
        let mut q = sqlx::query(&query);
        if let Some(name) = &updates.name {
            q = q.bind(name);
        }
        if let Some(properties) = &updates.properties {
            q = q.bind(serde_json::to_string(properties)?);
        }
        q = q.bind(tenant_id.to_string());
        
        q.execute(&self.pool).await?;
        
        self.get_tenant(tenant_id).await?
            .ok_or_else(|| anyhow::anyhow!("Tenant not found"))
    }

    async fn delete_tenant(&self, tenant_id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM tenants WHERE id = ?")
            .bind(tenant_id.to_string())
            .execute(&self.pool)
            .await?;
        
        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Tenant not found"));
        }
        Ok(())
    }

    // Warehouse Operations
    async fn create_warehouse(&self, tenant_id: Uuid, warehouse: Warehouse) -> Result<()> {
        sqlx::query("INSERT INTO warehouses (id, tenant_id, name, use_sts, storage_config) VALUES (?, ?, ?, ?, ?)")
            .bind(warehouse.id.to_string())
            .bind(tenant_id.to_string())
            .bind(&warehouse.name)
            .bind(if warehouse.use_sts { 1 } else { 0 })
            .bind(serde_json::to_string(&warehouse.storage_config)?)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_warehouse(&self, tenant_id: Uuid, name: String) -> Result<Option<Warehouse>> {
        let row = sqlx::query("SELECT id, name, use_sts, storage_config FROM warehouses WHERE tenant_id = ? AND name = ?")
            .bind(tenant_id.to_string())
            .bind(&name)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(Warehouse {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                tenant_id,
                name: row.get("name"),
                use_sts: row.get::<i32, _>("use_sts") != 0,
                storage_config: serde_json::from_str(&row.get::<String, _>("storage_config"))?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_warehouses(&self, tenant_id: Uuid) -> Result<Vec<Warehouse>> {
        let rows = sqlx::query("SELECT id, name, use_sts, storage_config FROM warehouses WHERE tenant_id = ?")
            .bind(tenant_id.to_string())
            .fetch_all(&self.pool)
            .await?;

        let mut warehouses = Vec::new();
        for row in rows {
            warehouses.push(Warehouse {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                tenant_id,
                name: row.get("name"),
                use_sts: row.get::<i32, _>("use_sts") != 0,
                storage_config: serde_json::from_str(&row.get::<String, _>("storage_config"))?,
            });
        }
        Ok(warehouses)
    }

    async fn update_warehouse(&self, tenant_id: Uuid, name: String, updates: pangolin_core::model::WarehouseUpdate) -> Result<Warehouse> {
        let mut query = String::from("UPDATE warehouses SET ");
        let mut set_clauses = Vec::new();
        
        if updates.name.is_some() {
            set_clauses.push("name = ?");
        }
        if updates.storage_config.is_some() {
            set_clauses.push("storage_config = ?");
        }
        if updates.use_sts.is_some() {
            set_clauses.push("use_sts = ?");
        }
        
        if set_clauses.is_empty() {
            return self.get_warehouse(tenant_id, name).await?
                .ok_or_else(|| anyhow::anyhow!("Warehouse not found"));
        }
        
        query.push_str(&set_clauses.join(", "));
        query.push_str(" WHERE tenant_id = ? AND name = ?");
        
        let mut q = sqlx::query(&query);
        if let Some(new_name) = &updates.name {
            q = q.bind(new_name);
        }
        if let Some(config) = &updates.storage_config {
            q = q.bind(serde_json::to_string(config)?);
        }
        if let Some(use_sts) = updates.use_sts {
            q = q.bind(use_sts as i32);
        }
        q = q.bind(tenant_id.to_string()).bind(&name);
        
        q.execute(&self.pool).await?;
        
        let new_name = updates.name.unwrap_or(name);
        self.get_warehouse(tenant_id, new_name).await?
            .ok_or_else(|| anyhow::anyhow!("Warehouse not found"))
    }

    async fn delete_warehouse(&self, tenant_id: Uuid, name: String) -> Result<()> {
        let result = sqlx::query("DELETE FROM warehouses WHERE tenant_id = ? AND name = ?")
            .bind(tenant_id.to_string())
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
        sqlx::query("INSERT INTO catalogs (id, tenant_id, name, catalog_type, warehouse_name, storage_location, federated_config, properties) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(catalog.id.to_string())
            .bind(tenant_id.to_string())
            .bind(&catalog.name)
            .bind(format!("{:?}", catalog.catalog_type))
            .bind(&catalog.warehouse_name)
            .bind(&catalog.storage_location)
            .bind(serde_json::to_string(&catalog.federated_config)?)
            .bind(serde_json::to_string(&catalog.properties)?)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_catalog(&self, tenant_id: Uuid, name: String) -> Result<Option<Catalog>> {
        let row = sqlx::query("SELECT id, name, catalog_type, warehouse_name, storage_location, federated_config, properties FROM catalogs WHERE tenant_id = ? AND name = ?")
            .bind(tenant_id.to_string())
            .bind(&name)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let catalog_type_str: String = row.get("catalog_type");
            let catalog_type = match catalog_type_str.as_str() {
                "Local" => CatalogType::Local,
                "Federated" => CatalogType::Federated,
                _ => CatalogType::Local,
            };
            
            Ok(Some(Catalog {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                name: row.get("name"),
                catalog_type,
                warehouse_name: row.get("warehouse_name"),
                storage_location: row.get("storage_location"),
                federated_config: serde_json::from_str(&row.get::<String, _>("federated_config")).ok(),
                properties: serde_json::from_str(&row.get::<String, _>("properties")).unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_catalogs(&self, tenant_id: Uuid) -> Result<Vec<Catalog>> {
        let rows = sqlx::query("SELECT id, name, catalog_type, warehouse_name, storage_location, federated_config, properties FROM catalogs WHERE tenant_id = ?")
            .bind(tenant_id.to_string())
            .fetch_all(&self.pool)
            .await?;

        let mut catalogs = Vec::new();
        for row in rows {
            let catalog_type_str: String = row.get("catalog_type");
            let catalog_type = match catalog_type_str.as_str() {
                "Local" => CatalogType::Local,
                "Federated" => CatalogType::Federated,
                _ => CatalogType::Local,
            };
            
            catalogs.push(Catalog {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                name: row.get("name"),
                catalog_type,
                warehouse_name: row.get("warehouse_name"),
                storage_location: row.get("storage_location"),
                federated_config: serde_json::from_str(&row.get::<String, _>("federated_config")).ok(),
                properties: serde_json::from_str(&row.get::<String, _>("properties")).unwrap_or_default(),
            });
        }
        Ok(catalogs)
    }

    async fn update_catalog(&self, tenant_id: Uuid, name: String, updates: pangolin_core::model::CatalogUpdate) -> Result<Catalog> {
        let mut query = String::from("UPDATE catalogs SET ");
        let mut set_clauses = Vec::new();
        
        if updates.warehouse_name.is_some() {
            set_clauses.push("warehouse_name = ?");
        }
        if updates.storage_location.is_some() {
            set_clauses.push("storage_location = ?");
        }
        if updates.properties.is_some() {
            set_clauses.push("properties = ?");
        }
        
        if set_clauses.is_empty() {
            return self.get_catalog(tenant_id, name).await?
                .ok_or_else(|| anyhow::anyhow!("Catalog not found"));
        }
        
        query.push_str(&set_clauses.join(", "));
        query.push_str(" WHERE tenant_id = ? AND name = ?");
        
        let mut q = sqlx::query(&query);
        if let Some(warehouse_name) = &updates.warehouse_name {
            q = q.bind(warehouse_name);
        }
        if let Some(storage_location) = &updates.storage_location {
            q = q.bind(storage_location);
        }
        if let Some(properties) = &updates.properties {
            q = q.bind(serde_json::to_string(properties)?);
        }
        q = q.bind(tenant_id.to_string()).bind(&name);
        
        q.execute(&self.pool).await?;
        
        self.get_catalog(tenant_id, name).await?
            .ok_or_else(|| anyhow::anyhow!("Catalog not found"))
    }

    async fn delete_catalog(&self, tenant_id: Uuid, name: String) -> Result<()> {
        // SQLite supports ON DELETE CASCADE if Foreign Keys are enabled.
        // I enabled them in `new()`, so standard deletion should actually work for children.
        // BUT my schema for `tags`, `branches`, etc. relies on keys.
        // Schema: FOREIGN KEY (tenant_id) REFERENCES tenants(id) -> This cascades tenant deletion.
        // Wait, `catalogs` doesn't have child referencing it by FK in `branches` implementation in schema?
        // In schema: `branches` PK (tenant_id, catalog_name, name). No direct FK to `catalogs`.
        // So I must delete cascadingly manually.
        
        let tid = tenant_id.to_string();
        
        // 1. Tags
        sqlx::query("DELETE FROM tags WHERE tenant_id = ? AND catalog_name = ?")
            .bind(&tid).bind(&name).execute(&self.pool).await?;
            
        // 2. Branches
        sqlx::query("DELETE FROM branches WHERE tenant_id = ? AND catalog_name = ?")
            .bind(&tid).bind(&name).execute(&self.pool).await?;

        // 3. Assets
        sqlx::query("DELETE FROM assets WHERE tenant_id = ? AND catalog_name = ?")
            .bind(&tid).bind(&name).execute(&self.pool).await?;

        // 4. Namespaces
        sqlx::query("DELETE FROM namespaces WHERE tenant_id = ? AND catalog_name = ?")
            .bind(&tid).bind(&name).execute(&self.pool).await?;

        // 5. Catalog
        let result = sqlx::query("DELETE FROM catalogs WHERE tenant_id = ? AND name = ?")
            .bind(&tid)
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
        let namespace_path = serde_json::to_string(&namespace.name)?;
        sqlx::query("INSERT INTO namespaces (id, tenant_id, catalog_name, namespace_path, properties) VALUES (?, ?, ?, ?, ?)")
            .bind(Uuid::new_v4().to_string())
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(&namespace_path)
            .bind(serde_json::to_string(&namespace.properties)?)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<Option<Namespace>> {
        let namespace_path = serde_json::to_string(&namespace)?;
        let row = sqlx::query("SELECT namespace_path, properties FROM namespaces WHERE tenant_id = ? AND catalog_name = ? AND namespace_path = ?")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(&namespace_path)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(Namespace {
                name: serde_json::from_str(&row.get::<String, _>("namespace_path"))?,
                properties: serde_json::from_str(&row.get::<String, _>("properties"))?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_namespaces(&self, tenant_id: Uuid, catalog_name: &str, _parent: Option<String>) -> Result<Vec<Namespace>> {
        let rows = sqlx::query("SELECT namespace_path, properties FROM namespaces WHERE tenant_id = ? AND catalog_name = ?")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .fetch_all(&self.pool)
            .await?;

        let mut namespaces = Vec::new();
        for row in rows {
            namespaces.push(Namespace {
                name: serde_json::from_str(&row.get::<String, _>("namespace_path"))?,
                properties: serde_json::from_str(&row.get::<String, _>("properties"))?,
            });
        }
        Ok(namespaces)
    }

    async fn delete_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<()> {
        let namespace_path = serde_json::to_string(&namespace)?;
        sqlx::query("DELETE FROM namespaces WHERE tenant_id = ? AND catalog_name = ? AND namespace_path = ?")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(&namespace_path)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_namespace_properties(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>, properties: HashMap<String, String>) -> Result<()> {
        let ns = self.get_namespace(tenant_id, catalog_name, namespace.clone()).await?;
        if let Some(mut n) = ns {
            n.properties.extend(properties);
            let namespace_path = serde_json::to_string(&namespace)?;
            sqlx::query("UPDATE namespaces SET properties = ? WHERE tenant_id = ? AND catalog_name = ? AND namespace_path = ?")
                .bind(serde_json::to_string(&n.properties)?)
                .bind(tenant_id.to_string())
                .bind(catalog_name)
                .bind(&namespace_path)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    // Asset Operations
    async fn create_asset(&self, tenant_id: Uuid, catalog_name: &str, _branch: Option<String>, namespace: Vec<String>, asset: Asset) -> Result<()> {
        let namespace_path = serde_json::to_string(&namespace)?;
        sqlx::query("INSERT INTO assets (id, tenant_id, catalog_name, namespace_path, name, asset_type, metadata_location, properties) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(asset.id.to_string())
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(&namespace_path)
            .bind(&asset.name)
            .bind(format!("{:?}", asset.kind))
            .bind(&asset.location)
            .bind(serde_json::to_string(&asset.properties)?)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_asset(&self, tenant_id: Uuid, catalog_name: &str, _branch: Option<String>, namespace: Vec<String>, name: String) -> Result<Option<Asset>> {
        let namespace_path = serde_json::to_string(&namespace)?;
        let row = sqlx::query("SELECT id, name, asset_type, metadata_location, properties FROM assets WHERE tenant_id = ? AND catalog_name = ? AND namespace_path = ? AND name = ?")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(&namespace_path)
            .bind(&name)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let asset_type_str: String = row.get("asset_type");
            let kind = match asset_type_str.as_str() {
                "IcebergTable" => AssetType::IcebergTable,
                "View" => AssetType::View,
                _ => AssetType::IcebergTable,
            };
            
            Ok(Some(Asset {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                name: row.get("name"),
                kind,
                location: row.get::<Option<String>, _>("metadata_location").unwrap_or_default(),
                properties: serde_json::from_str(&row.get::<String, _>("properties")).unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_asset_by_id(&self, tenant_id: Uuid, asset_id: Uuid) -> Result<Option<(Asset, String)>> {
        let row = sqlx::query("SELECT id, name, catalog_name, asset_type, metadata_location, properties FROM assets WHERE tenant_id = ? AND id = ?")
            .bind(tenant_id.to_string())
            .bind(asset_id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let catalog_name: String = row.get("catalog_name");
            let asset_type_str: String = row.get("asset_type");
            let kind = match asset_type_str.as_str() {
                "IcebergTable" => AssetType::IcebergTable,
                "View" => AssetType::View,
                _ => AssetType::IcebergTable,
            };
            
            let asset = Asset {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                name: row.get("name"),
                kind,
                location: row.get::<Option<String>, _>("metadata_location").unwrap_or_default(),
                properties: serde_json::from_str(&row.get::<String, _>("properties")).unwrap_or_default(),
            };
            
            Ok(Some((asset, catalog_name)))
        } else {
            Ok(None)
        }
    }

    async fn list_assets(&self, tenant_id: Uuid, catalog_name: &str, _branch: Option<String>, namespace: Vec<String>) -> Result<Vec<Asset>> {
        let namespace_path = serde_json::to_string(&namespace)?;
        let rows = sqlx::query("SELECT id, name, asset_type, metadata_location, properties FROM assets WHERE tenant_id = ? AND catalog_name = ? AND namespace_path = ?")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(&namespace_path)
            .fetch_all(&self.pool)
            .await?;

        let mut assets = Vec::new();
        for row in rows {
            let asset_type_str: String = row.get("asset_type");
            let kind = match asset_type_str.as_str() {
                "IcebergTable" => AssetType::IcebergTable,
                "View" => AssetType::View,
                _ => AssetType::IcebergTable,
            };
            
            assets.push(Asset {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                name: row.get("name"),
                kind,
                location: row.get::<Option<String>, _>("metadata_location").unwrap_or_default(),
                properties: serde_json::from_str(&row.get::<String, _>("properties")).unwrap_or_default(),
            });
        }
        Ok(assets)
    }

    async fn delete_asset(&self, tenant_id: Uuid, catalog_name: &str, _branch: Option<String>, namespace: Vec<String>, name: String) -> Result<()> {
        let namespace_path = serde_json::to_string(&namespace)?;
        sqlx::query("DELETE FROM assets WHERE tenant_id = ? AND catalog_name = ? AND namespace_path = ? AND name = ?")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(&namespace_path)
            .bind(&name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn rename_asset(&self, tenant_id: Uuid, catalog_name: &str, _branch: Option<String>, source_namespace: Vec<String>, source_name: String, dest_namespace: Vec<String>, dest_name: String) -> Result<()> {
        let source_ns_path = serde_json::to_string(&source_namespace)?;
        let dest_ns_path = serde_json::to_string(&dest_namespace)?;
        
        sqlx::query("UPDATE assets SET namespace_path = ?, name = ? WHERE tenant_id = ? AND catalog_name = ? AND namespace_path = ? AND name = ?")
            .bind(&dest_ns_path)
            .bind(&dest_name)
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(&source_ns_path)
            .bind(&source_name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Branch, Tag, Commit operations (placeholder implementations)
    async fn create_branch(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Branch) -> Result<()> {
        Ok(())
    }

    async fn get_branch(&self, _tenant_id: Uuid, _catalog_name: &str, _name: String) -> Result<Option<Branch>> {
        Ok(None)
    }

    async fn list_branches(&self, _tenant_id: Uuid, _catalog_name: &str) -> Result<Vec<Branch>> {
        Ok(Vec::new())
    }

    async fn merge_branch(&self, _tenant_id: Uuid, _catalog_name: &str, _source: String, _target: String) -> Result<()> {
        Ok(())
    }

    async fn create_tag(&self, _tenant_id: Uuid, _catalog_name: &str, _tag: Tag) -> Result<()> {
        Ok(())
    }

    async fn get_tag(&self, _tenant_id: Uuid, _catalog_name: &str, _name: String) -> Result<Option<Tag>> {
        Ok(None)
    }

    async fn list_tags(&self, _tenant_id: Uuid, _catalog_name: &str) -> Result<Vec<Tag>> {
        Ok(Vec::new())
    }

    async fn delete_tag(&self, _tenant_id: Uuid, _catalog_name: &str, _name: String) -> Result<()> {
        Ok(())
    }

    async fn create_commit(&self, _tenant_id: Uuid, _commit: Commit) -> Result<()> {
        Ok(())
    }

    async fn get_commit(&self, _tenant_id: Uuid, _id: Uuid) -> Result<Option<Commit>> {
        Ok(None)
    }

    async fn get_metadata_location(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _namespace: Vec<String>, _table: String) -> Result<Option<String>> {
        Ok(None)
    }

    async fn update_metadata_location(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _namespace: Vec<String>, _table: String, _expected_location: Option<String>, _new_location: String) -> Result<()> {
        Ok(())
    }

    async fn log_audit_event(&self, _tenant_id: Uuid, _event: AuditLogEntry) -> Result<()> {
        Ok(())
    }

    async fn list_audit_events(&self, tenant_id: Uuid) -> Result<Vec<AuditLogEntry>> {
        let rows = sqlx::query("SELECT id, tenant_id, timestamp, actor, action, resource, details FROM audit_logs WHERE tenant_id = ? ORDER BY timestamp DESC LIMIT 100")
            .bind(tenant_id.to_string())
            .fetch_all(&self.pool)
            .await?;
            
        let mut events = Vec::new();
        for row in rows {
            let ts_millis: i64 = row.get("timestamp");
            events.push(AuditLogEntry {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                tenant_id,
                timestamp: Utc.timestamp_millis_opt(ts_millis).single().unwrap_or_default(),
                actor: row.get("actor"),
                action: row.get("action"),
                resource: row.get("resource"),
                details: serde_json::from_str(&row.get::<String, _>("details")).unwrap_or_default(),
            });
        }
        Ok(events)
    }

    // User Operations
    async fn create_user(&self, user: User) -> Result<()> {
        sqlx::query("INSERT INTO users (id, username, email, password_hash, oauth_provider, oauth_subject, tenant_id, role, created_at, updated_at, last_login, active) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(user.id.to_string())
            .bind(user.username)
            .bind(user.email)
            .bind(user.password_hash)
            .bind(user.oauth_provider.map(|p| format!("{:?}", p).to_lowercase()))
            .bind(user.oauth_subject)
            .bind(user.tenant_id.map(|u| u.to_string()))
            .bind(format!("{:?}", user.role))
            .bind(user.created_at.timestamp_millis())
            .bind(user.updated_at.timestamp_millis())
            .bind(user.last_login.map(|t| t.timestamp_millis()))
            .bind(if user.active { 1 } else { 0 })
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_user(&self, user_id: Uuid) -> Result<Option<User>> {
        let row = sqlx::query("SELECT id, username, email, password_hash, oauth_provider, oauth_subject, tenant_id, role, created_at, updated_at, last_login, active FROM users WHERE id = ?")
            .bind(user_id.to_string())
            .fetch_optional(&self.pool)
            .await?;
            
        self.row_to_user(row)
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let row = sqlx::query("SELECT id, username, email, password_hash, oauth_provider, oauth_subject, tenant_id, role, created_at, updated_at, last_login, active FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;
            
        self.row_to_user(row)
    }

    async fn list_users(&self, tenant_id: Option<Uuid>) -> Result<Vec<User>> {
        let rows = if let Some(tid) = tenant_id {
            sqlx::query("SELECT id, username, email, password_hash, oauth_provider, oauth_subject, tenant_id, role, created_at, updated_at, last_login, active FROM users WHERE tenant_id = ?")
                .bind(tid.to_string())
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query("SELECT id, username, email, password_hash, oauth_provider, oauth_subject, tenant_id, role, created_at, updated_at, last_login, active FROM users")
                .fetch_all(&self.pool)
                .await?
        };
        
        let mut users = Vec::new();
        for row in rows {
            if let Some(user) = self.row_to_user(Some(row))? {
                users.push(user);
            }
        }
        Ok(users)
    }

    async fn update_user(&self, user: User) -> Result<()> {
        sqlx::query("UPDATE users SET username = ?, email = ?, password_hash = ?, role = ?, active = ?, updated_at = ?, last_login = ? WHERE id = ?")
            .bind(user.username)
            .bind(user.email)
            .bind(user.password_hash)
            .bind(format!("{:?}", user.role))
            .bind(if user.active { 1 } else { 0 })
            .bind(Utc::now().timestamp_millis())
            .bind(user.last_login.map(|t| t.timestamp_millis()))
            .bind(user.id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete_user(&self, user_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = ?").bind(user_id.to_string()).execute(&self.pool).await?;
        Ok(())
    }

    // Role Operations
    async fn create_role(&self, role: Role) -> Result<()> {
        sqlx::query("INSERT INTO roles (id, tenant_id, name, description, permissions, created_by, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(role.id.to_string())
            .bind(role.tenant_id.to_string())
            .bind(role.name)
            .bind(role.description)
            .bind(serde_json::to_string(&role.permissions)?)
            .bind(role.created_by.to_string())
            .bind(role.created_at.timestamp_millis())
            .bind(role.updated_at.timestamp_millis())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_role(&self, role_id: Uuid) -> Result<Option<Role>> {
        let row = sqlx::query("SELECT id, tenant_id, name, description, permissions, created_by, created_at, updated_at FROM roles WHERE id = ?")
            .bind(role_id.to_string())
            .fetch_optional(&self.pool)
            .await?;
        
        if let Some(row) = row {
            Ok(Some(Role {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                tenant_id: Uuid::parse_str(&row.get::<String, _>("tenant_id"))?,
                name: row.get("name"),
                description: row.get("description"),
                permissions: serde_json::from_str(&row.get::<String, _>("permissions"))?,
                created_by: Uuid::parse_str(&row.get::<String, _>("created_by"))?,
                created_at: Utc.timestamp_millis_opt(row.get("created_at")).single().unwrap_or_default(),
                updated_at: Utc.timestamp_millis_opt(row.get("updated_at")).single().unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_roles(&self, tenant_id: Uuid) -> Result<Vec<Role>> {
        let rows = sqlx::query("SELECT id, tenant_id, name, description, permissions, created_by, created_at, updated_at FROM roles WHERE tenant_id = ?")
            .bind(tenant_id.to_string())
            .fetch_all(&self.pool)
            .await?;
            
        let mut roles = Vec::new();
        for row in rows {
            roles.push(Role {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                tenant_id: Uuid::parse_str(&row.get::<String, _>("tenant_id"))?,
                name: row.get("name"),
                description: row.get("description"),
                permissions: serde_json::from_str(&row.get::<String, _>("permissions"))?,
                created_by: Uuid::parse_str(&row.get::<String, _>("created_by"))?,
                created_at: Utc.timestamp_millis_opt(row.get("created_at")).single().unwrap_or_default(),
                updated_at: Utc.timestamp_millis_opt(row.get("updated_at")).single().unwrap_or_default(),
            });
        }
        Ok(roles)
    }

    async fn delete_role(&self, role_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM roles WHERE id = ?").bind(role_id.to_string()).execute(&self.pool).await?;
        Ok(())
    }

    async fn update_role(&self, role: Role) -> Result<()> {
        sqlx::query("UPDATE roles SET name = ?, description = ?, permissions = ?, updated_at = ? WHERE id = ?")
            .bind(role.name)
            .bind(role.description)
            .bind(serde_json::to_string(&role.permissions)?)
            .bind(Utc::now().timestamp_millis())
            .bind(role.id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn assign_role(&self, user_role: UserRoleAssignment) -> Result<()> {
        sqlx::query("INSERT INTO user_roles (user_id, role_id, assigned_by, assigned_at) VALUES (?, ?, ?, ?)")
            .bind(user_role.user_id.to_string())
            .bind(user_role.role_id.to_string())
            .bind(user_role.assigned_by.to_string())
            .bind(user_role.assigned_at.timestamp_millis())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn revoke_role(&self, user_id: Uuid, role_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM user_roles WHERE user_id = ? AND role_id = ?")
            .bind(user_id.to_string())
            .bind(role_id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<UserRoleAssignment>> {
        let rows = sqlx::query("SELECT user_id, role_id, assigned_by, assigned_at FROM user_roles WHERE user_id = ?")
            .bind(user_id.to_string())
            .fetch_all(&self.pool)
            .await?;
        
        let mut roles = Vec::new();
        for row in rows {
            roles.push(UserRoleAssignment {
                user_id: Uuid::parse_str(&row.get::<String, _>("user_id"))?,
                role_id: Uuid::parse_str(&row.get::<String, _>("role_id"))?,
                assigned_by: Uuid::parse_str(&row.get::<String, _>("assigned_by"))?,
                assigned_at: Utc.timestamp_millis_opt(row.get("assigned_at")).single().unwrap_or_default(),
            });
        }
        Ok(roles)
    }

    // Permission Operations
    async fn create_permission(&self, permission: Permission) -> Result<()> {
        sqlx::query("INSERT INTO permissions (id, user_id, scope, actions, granted_by, granted_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(permission.id.to_string())
            .bind(permission.user_id.to_string())
            .bind(serde_json::to_string(&permission.scope)?)
            .bind(serde_json::to_string(&permission.actions)?)
            .bind(permission.granted_by.to_string())
            .bind(permission.granted_at.timestamp_millis())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn revoke_permission(&self, permission_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM permissions WHERE id = ?").bind(permission_id.to_string()).execute(&self.pool).await?;
        Ok(())
    }

    async fn list_user_permissions(&self, user_id: Uuid) -> Result<Vec<Permission>> {
        let rows = sqlx::query("SELECT id, user_id, scope, actions, granted_by, granted_at FROM permissions WHERE user_id = ?")
            .bind(user_id.to_string())
            .fetch_all(&self.pool)
            .await?;
            
        let mut perms = Vec::new();
        for row in rows {
            perms.push(Permission {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                user_id: Uuid::parse_str(&row.get::<String, _>("user_id"))?,
                scope: serde_json::from_str(&row.get::<String, _>("scope"))?,
                actions: serde_json::from_str(&row.get::<String, _>("actions"))?,
                granted_by: Uuid::parse_str(&row.get::<String, _>("granted_by"))?,
                granted_at: Utc.timestamp_millis_opt(row.get("granted_at")).single().unwrap_or_default(),
            });
        }
        Ok(perms)
    }

    async fn read_file(&self, _path: &str) -> Result<Vec<u8>> {
        Err(anyhow::anyhow!("File operations not supported in SQLite store"))
    }

    async fn write_file(&self, _path: &str, _data: Vec<u8>) -> Result<()> {
        Err(anyhow::anyhow!("File operations not supported in SQLite store"))
    }

    async fn expire_snapshots(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _namespace: Vec<String>, _table: String, _retention_ms: i64) -> Result<()> {
        Ok(())
    }

    async fn remove_orphan_files(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _namespace: Vec<String>, _table: String, _older_than_ms: i64) -> Result<()> {
        Ok(())
    }

    // Access Request Operations
    async fn create_access_request(&self, request: AccessRequest) -> Result<()> {
        sqlx::query(
            "INSERT INTO access_requests (id, tenant_id, user_id, asset_id, reason, requested_at, status, reviewed_by, reviewed_at, review_comment) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(request.id.to_string())
        .bind(request.tenant_id.to_string())
        .bind(request.user_id.to_string())
        .bind(request.asset_id.to_string())
        .bind(request.reason)
        .bind(request.requested_at.timestamp_millis())
        .bind(format!("{:?}", request.status)) // Enum to string
        .bind(request.reviewed_by.map(|u| u.to_string()))
        .bind(request.reviewed_at.map(|t| t.timestamp_millis()))
        .bind(request.review_comment)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_access_request(&self, id: Uuid) -> Result<Option<AccessRequest>> {
        let row = sqlx::query("SELECT * FROM access_requests WHERE id = ?")
            .bind(id.to_string())
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
            "SELECT * FROM access_requests WHERE tenant_id = ?"
        )
        .bind(tenant_id.to_string())
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
            "UPDATE access_requests SET status = ?, reviewed_by = ?, reviewed_at = ?, review_comment = ? WHERE id = ?"
        )
        .bind(format!("{:?}", request.status))
        .bind(request.reviewed_by.map(|u| u.to_string()))
        .bind(request.reviewed_at.map(|t| t.timestamp_millis()))
        .bind(request.review_comment)
        .bind(request.id.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

impl SqliteStore {
    fn row_to_access_request(&self, row: sqlx::sqlite::SqliteRow) -> Result<AccessRequest> {
        let status_str: String = row.get("status");
        let status = match status_str.as_str() {
            "Pending" => RequestStatus::Pending,
            "Approved" => RequestStatus::Approved,
            "Rejected" => RequestStatus::Rejected,
            _ => RequestStatus::Pending, 
        };

        Ok(AccessRequest {
            id: Uuid::parse_str(&row.get::<String, _>("id"))?,
            tenant_id: Uuid::parse_str(&row.get::<String, _>("tenant_id"))?,
            user_id: Uuid::parse_str(&row.get::<String, _>("user_id"))?,
            asset_id: Uuid::parse_str(&row.get::<String, _>("asset_id"))?,
            reason: row.get("reason"),
            requested_at: Utc.timestamp_millis_opt(row.get("requested_at")).single().unwrap_or_default(),
            status,
            reviewed_by: row.get::<Option<String>, _>("reviewed_by").map(|s| Uuid::parse_str(&s)).transpose()?,
            reviewed_at: row.get::<Option<i64>, _>("reviewed_at").map(|t| Utc.timestamp_millis_opt(t).single().unwrap_or_default()),
            review_comment: row.get("review_comment"),
        })
    }

    fn row_to_user(&self, row: Option<sqlx::sqlite::SqliteRow>) -> Result<Option<User>> {
        if let Some(row) = row {
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
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                username: row.get("username"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                oauth_provider,
                oauth_subject: row.get("oauth_subject"),
                tenant_id: row.get::<Option<String>, _>("tenant_id").map(|s| Uuid::parse_str(&s)).transpose()?,
                role,
                created_at: Utc.timestamp_millis_opt(row.get("created_at")).single().unwrap_or_default(),
                updated_at: Utc.timestamp_millis_opt(row.get("updated_at")).single().unwrap_or_default(),
                last_login: row.get::<Option<i64>, _>("last_login").map(|t| Utc.timestamp_millis_opt(t).single().unwrap_or_default()),
                active: row.get::<i32, _>("active") != 0,
            }))
        } else {
            Ok(None)
        }
    }
}

