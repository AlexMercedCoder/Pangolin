use async_trait::async_trait;
use crate::CatalogStore;
use pangolin_core::model::*;
use pangolin_core::audit::AuditLogEntry;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;
use anyhow::Result;

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

    async fn delete_catalog(&self, tenant_id: Uuid, name: String) -> Result<()> {
        let result = sqlx::query("DELETE FROM catalogs WHERE tenant_id = ? AND name = ?")
            .bind(tenant_id.to_string())
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
            .bind(Uuid::new_v4().to_string())
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

    async fn list_audit_events(&self, _tenant_id: Uuid) -> Result<Vec<AuditLogEntry>> {
        Ok(Vec::new())
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
}
