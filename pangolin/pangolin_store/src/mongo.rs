use crate::CatalogStore;
use anyhow::Result;
use async_trait::async_trait;
use futures::stream::TryStreamExt;
use mongodb::{Client, Collection, Database};
use mongodb::bson::{doc, Document, Bson, Binary};
use mongodb::bson::spec::BinarySubtype;
use mongodb::options::{ClientOptions, ReplaceOptions};
use pangolin_core::model::{
    Asset, AssetType, Branch, Catalog, Commit, Namespace, Tag, Tenant, Warehouse, VendingStrategy,
};
use pangolin_core::user::{User, UserRole as CoreUserRole, OAuthProvider};
use pangolin_core::permission::{Role, Permission, PermissionGrant, UserRole as UserRoleAssignment};
use pangolin_core::business_metadata::{AccessRequest, RequestStatus};
use crate::signer::{Signer, Credentials};
use object_store::ObjectStore;
use object_store::aws::AmazonS3Builder;
use pangolin_core::audit::AuditLogEntry;
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Clone)]
pub struct MongoStore {
    client: Client,
    db: Database,
}

impl MongoStore {
    pub async fn new(connection_string: &str, database_name: &str) -> Result<Self> {
        let client_options = ClientOptions::parse(connection_string).await?;
        let client = Client::with_options(client_options)?;
        let db = client.database(database_name);
        Ok(Self { client, db })
    }

    fn tenants(&self) -> Collection<Tenant> {
        self.db.collection("tenants")
    }

    fn warehouses(&self) -> Collection<Warehouse> {
        self.db.collection("warehouses")
    }

    fn catalogs(&self) -> Collection<Catalog> {
        self.db.collection("catalogs")
    }

    fn namespaces(&self) -> Collection<Namespace> {
        self.db.collection("namespaces")
    }

    fn assets(&self) -> Collection<Asset> {
        self.db.collection("assets")
    }

    fn branches(&self) -> Collection<Branch> {
        self.db.collection("branches")
    }

    fn tags(&self) -> Collection<Tag> {
        self.db.collection("tags")
    }

    fn commits(&self) -> Collection<Commit> {
        self.db.collection("commits")
    }

    fn audit_logs(&self) -> Collection<AuditLogEntry> {
        self.db.collection("audit_logs")
    }
    
    fn users(&self) -> Collection<User> {
        self.db.collection("users")
    }
    
    fn roles(&self) -> Collection<Role> {
        self.db.collection("roles")
    }
    
    fn user_roles(&self) -> Collection<UserRoleAssignment> {
        self.db.collection("user_roles")
    }
    
    fn permissions(&self) -> Collection<Permission> {
        self.db.collection("permissions")
    }

    fn access_requests(&self) -> Collection<AccessRequest> {
        self.db.collection("access_requests")
    }
}

#[async_trait]
impl CatalogStore for MongoStore {
    // Tenant Operations
    async fn create_tenant(&self, tenant: Tenant) -> Result<()> {
        self.tenants().insert_one(tenant).await?;
        Ok(())
    }

    async fn get_tenant(&self, id: Uuid) -> Result<Option<Tenant>> {
        let filter = doc! { "id": to_bson_uuid(id) };
        let tenant = self.tenants().find_one(filter).await?;
        Ok(tenant)
    }

    async fn list_tenants(&self) -> Result<Vec<Tenant>> {
        let cursor = self.tenants().find(doc! {}).await?;
        let tenants: Vec<Tenant> = cursor.try_collect().await?;
        Ok(tenants)
    }

    async fn update_tenant(&self, tenant_id: Uuid, updates: pangolin_core::model::TenantUpdate) -> Result<Tenant> {
        let filter = doc! { "id": to_bson_uuid(tenant_id) };
        let mut update_doc = doc! {};
        
        if let Some(name) = updates.name {
            update_doc.insert("name", name);
        }
        if let Some(properties) = updates.properties {
            update_doc.insert("properties", bson::to_bson(&properties)?);
        }
        
        if update_doc.is_empty() {
            return self.get_tenant(tenant_id).await?
                .ok_or_else(|| anyhow::anyhow!("Tenant not found"));
        }
        
        let update = doc! { "$set": update_doc };
        self.tenants().update_one(filter.clone(), update).await?;
        
        self.get_tenant(tenant_id).await?
            .ok_or_else(|| anyhow::anyhow!("Tenant not found"))
    }

    async fn delete_tenant(&self, tenant_id: Uuid) -> Result<()> {
        let filter = doc! { "id": to_bson_uuid(tenant_id) };
        let result = self.tenants().delete_one(filter).await?;
        
        if result.deleted_count == 0 {
            return Err(anyhow::anyhow!("Tenant not found"));
        }
        Ok(())
    }

    // Warehouse Operations
    async fn create_warehouse(&self, tenant_id: Uuid, warehouse: Warehouse) -> Result<()> {
        // We might want to store tenant_id in the warehouse document if it's not already there
        // The Warehouse struct has tenant_id.
        self.warehouses().insert_one(warehouse).await?;
        Ok(())
    }

    async fn get_warehouse(&self, tenant_id: Uuid, name: String) -> Result<Option<Warehouse>> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id), "name": name };
        let warehouse = self.warehouses().find_one(filter).await?;
        Ok(warehouse)
    }

    async fn list_warehouses(&self, tenant_id: Uuid) -> Result<Vec<Warehouse>> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id) };
        let cursor = self.warehouses().find(filter).await?;
        let warehouses: Vec<Warehouse> = cursor.try_collect().await?;
        Ok(warehouses)
    }

    async fn update_warehouse(&self, tenant_id: Uuid, name: String, updates: pangolin_core::model::WarehouseUpdate) -> Result<Warehouse> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id), "name": &name };
        let mut update_doc = doc! {};
        
        if let Some(new_name) = &updates.name {
            update_doc.insert("name", new_name);
        }
        if let Some(config) = &updates.storage_config {
            update_doc.insert("storage_config", bson::to_bson(config)?);
        }
        if let Some(use_sts) = updates.use_sts {
            update_doc.insert("use_sts", use_sts);
        }
        if let Some(vending_strategy) = updates.vending_strategy {
            update_doc.insert("vending_strategy", bson::to_bson(&vending_strategy)?);
        }
        
        if update_doc.is_empty() {
            return self.get_warehouse(tenant_id, name).await?
                .ok_or_else(|| anyhow::anyhow!("Warehouse not found"));
        }
        
        let update = doc! { "$set": update_doc };
        self.warehouses().update_one(filter, update).await?;
        
        let new_name = updates.name.unwrap_or(name);
        self.get_warehouse(tenant_id, new_name).await?
            .ok_or_else(|| anyhow::anyhow!("Warehouse not found"))
    }

    async fn delete_warehouse(&self, tenant_id: Uuid, name: String) -> Result<()> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id), "name": &name };
        let result = self.warehouses().delete_one(filter).await?;
        
        if result.deleted_count == 0 {
            return Err(anyhow::anyhow!("Warehouse '{}' not found", name));
        }
        Ok(())
    }

    // Catalog Operations
    async fn create_catalog(&self, tenant_id: Uuid, catalog: Catalog) -> Result<()> {
        // Catalog struct doesn't have tenant_id, so we need to wrap it or add it?
        // Wait, Catalog struct in model.rs:
        // pub struct Catalog { pub name: String, pub warehouse_name: Option<String>, pub storage_location: Option<String>, pub properties: HashMap<String, String> }
        // It doesn't have tenant_id.
        // In Postgres we added a column. In Mongo we can wrap it in a document or just add the field dynamically if we use Document.
        // But we are using typed Collection<Catalog>.
        // We should probably use a wrapper struct for storage or just use Document.
        // Let's use Document for flexibility here since we need to add tenant_id context.
        
        let mut doc = doc! {
            "id": to_bson_uuid(catalog.id),
            "tenant_id": to_bson_uuid(tenant_id),
            "name": &catalog.name,
            "catalog_type": format!("{:?}", catalog.catalog_type),
            "properties": mongodb::bson::to_bson(&catalog.properties)?
        };
        
        // Add optional fields
        if let Some(ref warehouse_name) = catalog.warehouse_name {
            doc.insert("warehouse_name", warehouse_name);
        }
        if let Some(ref storage_location) = catalog.storage_location {
            doc.insert("storage_location", storage_location);
        }
        if let Some(ref federated_config) = catalog.federated_config {
            doc.insert("federated_config", mongodb::bson::to_bson(federated_config)?);
        }
        
        self.db.collection::<Document>("catalogs").insert_one(doc).await?;
        Ok(())
    }

    async fn get_catalog(&self, tenant_id: Uuid, name: String) -> Result<Option<Catalog>> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id), "name": name };
        let doc = self.db.collection::<Catalog>("catalogs").find_one(filter).await?;
        Ok(doc)
    }

    async fn list_catalogs(&self, tenant_id: Uuid) -> Result<Vec<Catalog>> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id) };
        let cursor = self.db.collection::<Catalog>("catalogs").find(filter).await?;
        let catalogs: Vec<Catalog> = cursor.try_collect().await?;
        Ok(catalogs)
    }

    async fn update_catalog(&self, tenant_id: Uuid, name: String, updates: pangolin_core::model::CatalogUpdate) -> Result<Catalog> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id), "name": &name };
        let mut update_doc = doc! {};
        
        if let Some(warehouse_name) = updates.warehouse_name {
            update_doc.insert("warehouse_name", warehouse_name);
        }
        if let Some(storage_location) = updates.storage_location {
            update_doc.insert("storage_location", storage_location);
        }
        if let Some(properties) = updates.properties {
            update_doc.insert("properties", bson::to_bson(&properties)?);
        }
        
        if update_doc.is_empty() {
            return self.get_catalog(tenant_id, name).await?
                .ok_or_else(|| anyhow::anyhow!("Catalog not found"));
        }
        
        let update = doc! { "$set": update_doc };
        self.db.collection::<Document>("catalogs").update_one(filter, update).await?;
        
        self.get_catalog(tenant_id, name).await?
            .ok_or_else(|| anyhow::anyhow!("Catalog not found"))
    }

    async fn delete_catalog(&self, tenant_id: Uuid, name: String) -> Result<()> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id), "name": &name }; // For catalog
        // For children, filter is slightly different (catalog_name property) or similar
        let child_filter = doc! { "tenant_id": to_bson_uuid(tenant_id), "catalog_name": &name };

        // 1. Tags
        self.db.collection::<Document>("tags").delete_many(child_filter.clone()).await?;
        // 2. Branches
        self.db.collection::<Document>("branches").delete_many(child_filter.clone()).await?;
        // 3. Assets
        self.db.collection::<Document>("assets").delete_many(child_filter.clone()).await?;
        // 4. Namespaces
        self.db.collection::<Document>("namespaces").delete_many(child_filter.clone()).await?;

        // 5. Catalog
        let result = self.db.collection::<Document>("catalogs").delete_one(filter).await?;
        
        if result.deleted_count == 0 {
            return Err(anyhow::anyhow!("Catalog '{}' not found", name));
        }
        Ok(())
    }

    // Namespace Operations
    async fn create_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Namespace) -> Result<()> {
        let doc = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "name": &namespace.name, // Vec<String> -> Array
            "properties": mongodb::bson::to_bson(&namespace.properties)?
        };
        self.db.collection::<Document>("namespaces").insert_one(doc).await?;
        Ok(())
    }

    async fn get_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<Option<Namespace>> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "name": namespace
        };
        let doc = self.db.collection::<Namespace>("namespaces").find_one(filter).await?;
        Ok(doc)
    }

    async fn list_namespaces(&self, tenant_id: Uuid, catalog_name: &str, parent: Option<String>) -> Result<Vec<Namespace>> {
        let mut filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name
        };
        
        if let Some(p) = parent {
            // This is tricky. Namespace name is Vec<String>.
            // "parent" usually implies a hierarchy.
            // If parent is "a.b", we want "a.b.c", "a.b.d".
            // We can filter where "name" starts with the parent components.
            // But `parent` argument is String (dot joined?). The trait says `parent: Option<String>`.
            // In Postgres we did LIKE 'parent%'.
            // Here we need to match array prefix.
            // Let's assume parent string is dot-separated or something.
            // Actually, `Namespace` struct has `name: Vec<String>`.
            // If parent is provided, we should convert it to Vec<String> and check if it's a prefix.
            // But `parent` is just a string.
            // Let's assume for now we just list all and filter in memory or implement prefix match if possible.
            // MVP: List all for catalog.
        }

        let cursor = self.db.collection::<Namespace>("namespaces").find(filter).await?;
        let namespaces: Vec<Namespace> = cursor.try_collect().await?;
        Ok(namespaces)
    }

    async fn delete_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<()> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "name": namespace
        };
        self.db.collection::<Document>("namespaces").delete_one(filter).await?;
        Ok(())
    }

    async fn update_namespace_properties(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>, properties: HashMap<String, String>) -> Result<()> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "name": namespace
        };
        
        // We need to merge properties.
        // $set: { "properties.key": "value" }
        let mut set_doc = doc! {};
        for (k, v) in properties {
            set_doc.insert(format!("properties.{}", k), v);
        }
        
        let update = doc! { "$set": set_doc };
        self.db.collection::<Document>("namespaces").update_one(filter, update).await?;
        Ok(())
    }

    // Asset Operations
    async fn create_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, asset: Asset) -> Result<()> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "namespace": &namespace,
            "name": &asset.name
        };
        
        let doc = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "namespace": namespace,
            "branch": branch,
            "id": to_bson_uuid(asset.id),
            "name": &asset.name,
            "kind": format!("{:?}", asset.kind),
            "location": &asset.location,
            "properties": mongodb::bson::to_bson(&asset.properties)?
        };
        
        let options = ReplaceOptions::builder().upsert(true).build();
        self.db.collection::<Document>("assets").replace_one(filter, doc).with_options(options).await?;
        Ok(())
    }

    async fn get_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<Option<Asset>> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "namespace": namespace,
            "name": name
        };
        // Note: Branch support in filter if needed, but usually asset name is unique in namespace?
        // Or is it versioned by branch?
        // In Postgres we had `active_branch`.
        // Let's stick to the filter.
        
        let doc = self.db.collection::<Document>("assets").find_one(filter).await?;
        
        if let Some(d) = doc {
            // Manual deserialization because we stored flattened fields
            let kind_str = d.get_str("kind")?;
            let kind = match kind_str {
                "IcebergTable" => AssetType::IcebergTable,
                "View" => AssetType::View,
                _ => AssetType::IcebergTable,
            };
            
            let properties: HashMap<String, String> = mongodb::bson::from_bson(d.get("properties").unwrap().clone())?;

            let id_bson = d.get("id").ok_or(anyhow::anyhow!("Missing id"))?;
            let id = from_bson_uuid(id_bson)?;

            Ok(Some(Asset {
                id,
                name: d.get_str("name")?.to_string(),
                kind,
                location: d.get_str("location").unwrap_or("").to_string(),
                properties,
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_assets(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>) -> Result<Vec<Asset>> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "namespace": namespace
        };
        let cursor = self.db.collection::<Document>("assets").find(filter).await?;
        let docs: Vec<Document> = cursor.try_collect().await?;
        
        let mut assets = Vec::new();
        for d in docs {
             let kind_str = d.get_str("kind")?;
            let kind = match kind_str {
                "IcebergTable" => AssetType::IcebergTable,
                "View" => AssetType::View,
                _ => AssetType::IcebergTable,
            };
            
            let properties: HashMap<String, String> = mongodb::bson::from_bson(d.get("properties").unwrap().clone())?;
            let id = if let Ok(i) = d.get("id").ok_or(anyhow::anyhow!("Missing id")).and_then(|b| from_bson_uuid(b)) {
                i
            } else {
                Uuid::new_v4() // Fallback if old data
            };

            assets.push(Asset {
                id,
                name: d.get_str("name")?.to_string(),
                kind,
                location: d.get_str("location").unwrap_or("").to_string(),
                properties,
            });
        }
        Ok(assets)
    }

    async fn get_asset_by_id(&self, tenant_id: Uuid, asset_id: Uuid) -> Result<Option<(Asset, String, Vec<String>)>> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "id": to_bson_uuid(asset_id)
        };
        let d = self.db.collection::<Document>("assets").find_one(filter).await?;
        
        if let Some(d) = d {
            let catalog_name = d.get_str("catalog_name")?.to_string();
            let namespace = d.get_array("namespace")?.iter().map(|v| v.as_str().unwrap().to_string()).collect(); 
            let kind_str = d.get_str("kind")?;
            let kind = match kind_str {
                "IcebergTable" => AssetType::IcebergTable,
                "View" => AssetType::View,
                _ => AssetType::IcebergTable,
            };
            
            let properties: HashMap<String, String> = mongodb::bson::from_bson(d.get("properties").unwrap().clone())?;

            let asset = Asset {
                id: asset_id,
                name: d.get_str("name")?.to_string(),
                kind,
                location: d.get_str("location").unwrap_or("").to_string(),
                properties,
            };
            
            Ok(Some((asset, catalog_name, namespace)))
        } else {
            Ok(None)
        }
    }

    async fn delete_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<()> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "namespace": namespace,
            "name": name
        };
        self.db.collection::<Document>("assets").delete_one(filter).await?;
        Ok(())
    }

    async fn rename_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, source_namespace: Vec<String>, source_name: String, dest_namespace: Vec<String>, dest_name: String) -> Result<()> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "namespace": source_namespace,
            "name": source_name
        };
        
        let update = doc! {
            "$set": {
                "namespace": dest_namespace,
                "name": dest_name
            }
        };
        
        self.db.collection::<Document>("assets").update_one(filter, update).await?;
        Ok(())
    }

    // Branch Operations
    async fn create_branch(&self, tenant_id: Uuid, catalog_name: &str, branch: Branch) -> Result<()> {
        let doc = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "name": &branch.name,
            "head_commit_id": branch.head_commit_id,
            "branch_type": format!("{:?}", branch.branch_type),
            "assets": &branch.assets
        };
        self.db.collection::<Document>("branches").insert_one(doc).await?;
        Ok(())
    }

    async fn get_branch(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Branch>> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "name": name
        };
        let doc = self.db.collection::<Document>("branches").find_one(filter).await?;
        
        if let Some(d) = doc {
             let type_str = d.get_str("branch_type")?;
             let branch_type = match type_str {
                 "Ingest" => pangolin_core::model::BranchType::Ingest,
                 "Experimental" => pangolin_core::model::BranchType::Experimental,
                 _ => pangolin_core::model::BranchType::Experimental,
             };
             
             Ok(Some(Branch {
                 name: d.get_str("name")?.to_string(),
                 head_commit_id: mongodb::bson::from_bson(d.get("head_commit_id").unwrap().clone())?,
                 branch_type,
                 assets: mongodb::bson::from_bson(d.get("assets").unwrap().clone())?,
             }))
        } else {
            Ok(None)
        }
    }

    async fn list_branches(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<Branch>> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name
        };
        let cursor = self.db.collection::<Document>("branches").find(filter).await?;
        let docs: Vec<Document> = cursor.try_collect().await?;
        
        let mut branches = Vec::new();
        for d in docs {
             let type_str = d.get_str("branch_type")?;
             let branch_type = match type_str {
                 "Ingest" => pangolin_core::model::BranchType::Ingest,
                 "Experimental" => pangolin_core::model::BranchType::Experimental,
                 _ => pangolin_core::model::BranchType::Experimental,
             };
             
             branches.push(Branch {
                 name: d.get_str("name")?.to_string(),
                 head_commit_id: mongodb::bson::from_bson(d.get("head_commit_id").unwrap().clone())?,
                 branch_type,
                 assets: mongodb::bson::from_bson(d.get("assets").unwrap().clone())?,
             });
        }
        Ok(branches)
    }

    async fn merge_branch(&self, tenant_id: Uuid, catalog_name: &str, source_branch: String, target_branch: String) -> Result<()> {
        let source = self.get_branch(tenant_id, catalog_name, source_branch.clone()).await?
            .ok_or_else(|| anyhow::anyhow!("Source branch not found"))?;
            
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "name": target_branch
        };
        
        let update = doc! {
            "$set": {
                "head_commit_id": source.head_commit_id
            }
        };
        
        self.db.collection::<Document>("branches").update_one(filter, update).await?;
        Ok(())
    }

    // Commit Operations
    async fn create_commit(&self, tenant_id: Uuid, commit: Commit) -> Result<()> {
        let mut doc = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "id": to_bson_uuid(commit.id),
            "timestamp": commit.timestamp,
            "author": &commit.author,
            "message": &commit.message,
            "operations": mongodb::bson::to_bson(&commit.operations)?
        };
        if let Some(parent_id) = commit.parent_id {
            doc.insert("parent_id", to_bson_uuid(parent_id));
        } else {
            doc.insert("parent_id", Bson::Null);
        }
        self.db.collection::<Document>("commits").insert_one(doc).await?;
        Ok(())
    }

    async fn get_commit(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Commit>> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "id": to_bson_uuid(id)
        };
        let doc = self.db.collection::<Document>("commits").find_one(filter).await?;
        
        if let Some(d) = doc {
            Ok(Some(Commit {
                id: mongodb::bson::from_bson(d.get("id").unwrap().clone())?,
                parent_id: mongodb::bson::from_bson(d.get("parent_id").unwrap().clone())?,
                timestamp: d.get_i64("timestamp")?,
                author: d.get_str("author")?.to_string(),
                message: d.get_str("message")?.to_string(),
                operations: mongodb::bson::from_bson(d.get("operations").unwrap().clone())?,
            }))
        } else {
            Ok(None)
        }
    }

    // File Operations
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
             let location = object_store::path::Path::from(key);
             let result = store.get(&location).await?;
             let bytes = result.bytes().await?;
             Ok(bytes.to_vec())
        } else {
             Err(anyhow::anyhow!("Only s3:// paths are supported in Mongo store"))
        }
    }

    async fn write_file(&self, path: &str, data: Vec<u8>) -> Result<()> {
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
             let location = object_store::path::Path::from(key);
             store.put(&location, data.into()).await?;
             Ok(())
        } else {
             Err(anyhow::anyhow!("Only s3:// paths are supported in Mongo store"))
        }
    }


    // Tag Operations
    async fn create_tag(&self, tenant_id: Uuid, catalog_name: &str, tag: Tag) -> Result<()> {
        let doc = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "name": &tag.name,
            "commit_id": to_bson_uuid(tag.commit_id)
        };
        self.db.collection::<Document>("tags").insert_one(doc).await?;
        Ok(())
    }

    async fn get_tag(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Tag>> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "name": name
        };
        let doc = self.db.collection::<Document>("tags").find_one(filter).await?;
        
        if let Some(d) = doc {
            Ok(Some(Tag {
                name: d.get_str("name")?.to_string(),
                commit_id: mongodb::bson::from_bson(d.get("commit_id").unwrap().clone())?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_tags(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<Tag>> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name
        };
        let cursor = self.db.collection::<Document>("tags").find(filter).await?;
        let docs: Vec<Document> = cursor.try_collect().await?;
        
        let mut tags = Vec::new();
        for d in docs {
            tags.push(Tag {
                name: d.get_str("name")?.to_string(),
                commit_id: mongodb::bson::from_bson(d.get("commit_id").unwrap().clone())?,
            });
        }
        Ok(tags)
    }

    async fn delete_tag(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<()> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "name": name
        };
        self.db.collection::<Document>("tags").delete_one(filter).await?;
        Ok(())
    }

    // Audit Operations
    async fn log_audit_event(&self, tenant_id: Uuid, event: AuditLogEntry) -> Result<()> {
        let doc = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "id": to_bson_uuid(event.id),
            "user_id": event.user_id.map(to_bson_uuid).unwrap_or(Bson::Null),
            "username": &event.username,
            "action": format!("{:?}", event.action),
            "resource_type": format!("{:?}", event.resource_type),
            "resource_id": event.resource_id.map(to_bson_uuid).unwrap_or(Bson::Null),
            "resource_name": &event.resource_name,
            "timestamp": mongodb::bson::DateTime::from_chrono(event.timestamp),
            "ip_address": event.ip_address.as_ref().map(|s| s.as_str()).unwrap_or(""),
            "user_agent": event.user_agent.as_ref().map(|s| s.as_str()).unwrap_or(""),
            "result": format!("{:?}", event.result),
            "error_message": event.error_message.as_ref().map(|s| s.as_str()).unwrap_or(""),
            "metadata": mongodb::bson::to_bson(&event.metadata)?
        };
        self.db.collection::<Document>("audit_logs").insert_one(doc).await?;
        Ok(())
    }

    async fn list_audit_events(&self, tenant_id: Uuid, filter: Option<pangolin_core::audit::AuditLogFilter>) -> Result<Vec<AuditLogEntry>> {
        let mut query = doc! { "tenant_id": to_bson_uuid(tenant_id) };
        
        // Build filter conditions
        if let Some(ref f) = filter {
            if let Some(user_id) = f.user_id {
                query.insert("user_id", to_bson_uuid(user_id));
            }
            if let Some(ref action) = f.action {
                query.insert("action", format!("{:?}", action));
            }
            if let Some(ref resource_type) = f.resource_type {
                query.insert("resource_type", format!("{:?}", resource_type));
            }
            if let Some(resource_id) = f.resource_id {
                query.insert("resource_id", to_bson_uuid(resource_id));
            }
            if let Some(start_time) = f.start_time {
                query.insert("timestamp", doc! { "$gte": mongodb::bson::DateTime::from_chrono(start_time) });
            }
            if let Some(end_time) = f.end_time {
                let existing = query.get_document_mut("timestamp").ok();
                if let Some(existing_doc) = existing {
                    existing_doc.insert("$lte", mongodb::bson::DateTime::from_chrono(end_time));
                } else {
                    query.insert("timestamp", doc! { "$lte": mongodb::bson::DateTime::from_chrono(end_time) });
                }
            }
            if let Some(ref result) = f.result {
                query.insert("result", format!("{:?}", result));
            }
        }
        
        // Build options with pagination
        let limit = filter.as_ref().and_then(|f| f.limit).unwrap_or(100) as i64;
        let skip = filter.as_ref().and_then(|f| f.offset).map(|o| o as u64);
        
        let mut options = mongodb::options::FindOptions::builder()
            .sort(doc! { "timestamp": -1 })
            .limit(limit)
            .build();
        
        if let Some(skip_val) = skip {
            options.skip = Some(skip_val);
        }
        
        let cursor = self.db.collection::<Document>("audit_logs")
            .find(query)
            .with_options(options)
            .await?;
        let docs: Vec<Document> = cursor.try_collect().await?;
        
        let mut events = Vec::new();
        for d in docs {
            // Parse action enum from string
            let action_str = d.get_str("action")?;
            let action = serde_json::from_str(&format!("\"{}\"" , action_str.to_lowercase()))
                .unwrap_or(pangolin_core::audit::AuditAction::CreateCatalog);
            
            // Parse resource_type enum from string
            let resource_type_str = d.get_str("resource_type")?;
            let resource_type = serde_json::from_str(&format!("\"{}\"", resource_type_str.to_lowercase()))
                .unwrap_or(pangolin_core::audit::ResourceType::Catalog);
            
            // Parse result enum from string
            let result_str = d.get_str("result")?;
            let result = serde_json::from_str(&format!("\"{}\"", result_str.to_lowercase()))
                .unwrap_or(pangolin_core::audit::AuditResult::Success);
            
            events.push(AuditLogEntry {
                id: mongodb::bson::from_bson(d.get("id").unwrap().clone())?,
                tenant_id,
                user_id: d.get("user_id").and_then(|b| from_bson_uuid(b).ok()),
                username: d.get_str("username")?.to_string(),
                action,
                resource_type,
                resource_id: d.get("resource_id").and_then(|b| from_bson_uuid(b).ok()),
                resource_name: d.get_str("resource_name")?.to_string(),
                timestamp: d.get_datetime("timestamp")?.to_chrono(),
                ip_address: d.get_str("ip_address").ok().filter(|s| !s.is_empty()).map(|s| s.to_string()),
                user_agent: d.get_str("user_agent").ok().filter(|s| !s.is_empty()).map(|s| s.to_string()),
                result,
                error_message: d.get_str("error_message").ok().filter(|s| !s.is_empty()).map(|s| s.to_string()),
                metadata: mongodb::bson::from_bson(d.get("metadata").unwrap().clone())?,
            });
        }
        Ok(events)
    }
    
    async fn get_audit_event(&self, tenant_id: Uuid, event_id: Uuid) -> Result<Option<AuditLogEntry>> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "id": to_bson_uuid(event_id)
        };
        
        let doc = self.db.collection::<Document>("audit_logs").find_one(filter).await?;
        
        if let Some(d) = doc {
            let action_str = d.get_str("action")?;
            let action = serde_json::from_str(&format!("\"{}\"", action_str.to_lowercase()))
                .unwrap_or(pangolin_core::audit::AuditAction::CreateCatalog);
            
            let resource_type_str = d.get_str("resource_type")?;
            let resource_type = serde_json::from_str(&format!("\"{}\"", resource_type_str.to_lowercase()))
                .unwrap_or(pangolin_core::audit::ResourceType::Catalog);
            
            let result_str = d.get_str("result")?;
            let result = serde_json::from_str(&format!("\"{}\"", result_str.to_lowercase()))
                .unwrap_or(pangolin_core::audit::AuditResult::Success);
            
            Ok(Some(AuditLogEntry {
                id: mongodb::bson::from_bson(d.get("id").unwrap().clone())?,
                tenant_id,
                user_id: d.get("user_id").and_then(|b| from_bson_uuid(b).ok()),
                username: d.get_str("username")?.to_string(),
                action,
                resource_type,
                resource_id: d.get("resource_id").and_then(|b| from_bson_uuid(b).ok()),
                resource_name: d.get_str("resource_name")?.to_string(),
                timestamp: d.get_datetime("timestamp")?.to_chrono(),
                ip_address: d.get_str("ip_address").ok().filter(|s| !s.is_empty()).map(|s| s.to_string()),
                user_agent: d.get_str("user_agent").ok().filter(|s| !s.is_empty()).map(|s| s.to_string()),
                result,
                error_message: d.get_str("error_message").ok().filter(|s| !s.is_empty()).map(|s| s.to_string()),
                metadata: mongodb::bson::from_bson(d.get("metadata").unwrap().clone())?,
            }))
        } else {
            Ok(None)
        }
    }
    
    async fn count_audit_events(&self, tenant_id: Uuid, filter: Option<pangolin_core::audit::AuditLogFilter>) -> Result<usize> {
        let mut query = doc! { "tenant_id": to_bson_uuid(tenant_id) };
        
        // Build same filter conditions as list_audit_events
        if let Some(ref f) = filter {
            if let Some(user_id) = f.user_id {
                query.insert("user_id", to_bson_uuid(user_id));
            }
            if let Some(ref action) = f.action {
                query.insert("action", format!("{:?}", action));
            }
            if let Some(ref resource_type) = f.resource_type {
                query.insert("resource_type", format!("{:?}", resource_type));
            }
            if let Some(resource_id) = f.resource_id {
                query.insert("resource_id", to_bson_uuid(resource_id));
            }
            if let Some(start_time) = f.start_time {
                query.insert("timestamp", doc! { "$gte": mongodb::bson::DateTime::from_chrono(start_time) });
            }
            if let Some(end_time) = f.end_time {
                let existing = query.get_document_mut("timestamp").ok();
                if let Some(existing_doc) = existing {
                    existing_doc.insert("$lte", mongodb::bson::DateTime::from_chrono(end_time));
                } else {
                    query.insert("timestamp", doc! { "$lte": mongodb::bson::DateTime::from_chrono(end_time) });
                }
            }
            if let Some(ref result) = f.result {
                query.insert("result", format!("{:?}", result));
            }
        }
        
        let count = self.db.collection::<Document>("audit_logs")
            .count_documents(query)
            .await? as usize;
        
        Ok(count)
    }

     // User Operations
    async fn create_user(&self, user: User) -> Result<()> {
        self.users().insert_one(user).await?;
        Ok(())
    }

    async fn get_user(&self, user_id: Uuid) -> Result<Option<User>> {
        let filter = doc! { "id": to_bson_uuid(user_id) };
        let user = self.users().find_one(filter).await?;
        Ok(user)
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let filter = doc! { "username": username };
        let user = self.users().find_one(filter).await?;
        Ok(user)
    }

    async fn list_users(&self, tenant_id: Option<Uuid>) -> Result<Vec<User>> {
        let filter = if let Some(tid) = tenant_id {
            doc! { "tenant_id": to_bson_uuid(tid) }
        } else {
            doc! {}
        };
        let cursor = self.users().find(filter).await?;
        let users: Vec<User> = cursor.try_collect().await?;
        Ok(users)
    }

    async fn update_user(&self, user: User) -> Result<()> {
        let filter = doc! { "id": to_bson_uuid(user.id) };
        let update = doc! { "$set": mongodb::bson::to_document(&user)? };
        self.users().update_one(filter, update).await?;
        Ok(())
    }

    async fn delete_user(&self, user_id: Uuid) -> Result<()> {
        let filter = doc! { "id": to_bson_uuid(user_id) };
        self.users().delete_one(filter).await?;
        Ok(())
    }

    // Role Operations
    async fn create_role(&self, role: Role) -> Result<()> {
        self.roles().insert_one(role).await?;
        Ok(())
    }

    async fn get_role(&self, role_id: Uuid) -> Result<Option<Role>> {
        let filter = doc! { "id": to_bson_uuid(role_id) };
        let role = self.roles().find_one(filter).await?;
        Ok(role)
    }

    async fn list_roles(&self, tenant_id: Uuid) -> Result<Vec<Role>> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id) };
        let cursor = self.roles().find(filter).await?;
        let roles: Vec<Role> = cursor.try_collect().await?;
        Ok(roles)
    }

    async fn delete_role(&self, role_id: Uuid) -> Result<()> {
        let filter = doc! { "id": to_bson_uuid(role_id) };
        self.roles().delete_one(filter).await?;
        Ok(())
    }

    async fn update_role(&self, role: Role) -> Result<()> {
        let filter = doc! { "id": to_bson_uuid(role.id) };
        let update = doc! { "$set": mongodb::bson::to_document(&role)? };
        self.roles().update_one(filter, update).await?;
        Ok(())
    }

    async fn assign_role(&self, user_role: UserRoleAssignment) -> Result<()> {
        self.user_roles().insert_one(user_role).await?;
        Ok(())
    }

    async fn revoke_role(&self, user_id: Uuid, role_id: Uuid) -> Result<()> {
        let filter = doc! { 
            "user_id": to_bson_uuid(user_id),
            "role_id": to_bson_uuid(role_id)
        };
        self.user_roles().delete_one(filter).await?;
        Ok(())
    }

    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<UserRoleAssignment>> {
        let filter = doc! { "user_id": to_bson_uuid(user_id) };
        let cursor = self.user_roles().find(filter).await?;
        let roles: Vec<UserRoleAssignment> = cursor.try_collect().await?;
        Ok(roles)
    }

    // Permission Operations
    async fn create_permission(&self, permission: Permission) -> Result<()> {
        self.permissions().insert_one(permission).await?;
        Ok(())
    }

    async fn revoke_permission(&self, permission_id: Uuid) -> Result<()> {
        let filter = doc! { "id": to_bson_uuid(permission_id) };
        self.permissions().delete_one(filter).await?;
        Ok(())
    }

    async fn list_user_permissions(&self, user_id: Uuid) -> Result<Vec<Permission>> {
        let filter = doc! { "user_id": to_bson_uuid(user_id) };
        let cursor = self.permissions().find(filter).await?;
        let perms: Vec<Permission> = cursor.try_collect().await?;
        Ok(perms)
    }

    async fn list_permissions(&self, tenant_id: Uuid) -> Result<Vec<Permission>> {
        // 1. Get all user IDs for the tenant
        let user_filter = doc! { "tenant_id": to_bson_uuid(tenant_id) };
        let user_cursor = self.users().find(user_filter).await?;
        let users: Vec<User> = user_cursor.try_collect().await?;
        let user_ids: Vec<mongodb::bson::Bson> = users.iter().map(|u| to_bson_uuid(u.id)).collect();

        if user_ids.is_empty() {
            return Ok(vec![]);
        }

        // 2. Get permissions for those users
        let perm_filter = doc! { "user_id": { "$in": user_ids } };
        let perm_cursor = self.permissions().find(perm_filter).await?;
        let perms: Vec<Permission> = perm_cursor.try_collect().await?;
        Ok(perms)
    }

    // Maintenance Operations
    async fn expire_snapshots(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _namespace: Vec<String>, _table: String, _retention_ms: i64) -> Result<()> {
        Ok(())
    }

    // Access Request Operations
    async fn create_access_request(&self, request: AccessRequest) -> Result<()> {
        self.access_requests().insert_one(request).await?;
        Ok(())
    }

    async fn get_access_request(&self, id: Uuid) -> Result<Option<AccessRequest>> {
        let filter = doc! { "id": to_bson_uuid(id) };
        let req = self.access_requests().find_one(filter).await?;
        Ok(req)
    }

    async fn list_access_requests(&self, tenant_id: Uuid) -> Result<Vec<AccessRequest>> {
        // AccessRequests stored with UserID/AssetID but not TenantID directly?
        // Struct has: id, user_id, asset_id...
        // User has tenant_id.
        // To filter by tenant_id, we need a join (lookup) or we store tenant_id denormalized on AccessRequest?
        // SQL implementation joins Users.
        // Mongo: $lookup.
        
        // Creating aggregation pipeline:
        let pipeline = vec![
            doc! {
                "$lookup": {
                    "from": "users",
                    "localField": "user_id",
                    "foreignField": "id",
                    "as": "user"
                }
            },
            doc! { "$unwind": "$user" },
            doc! { "$match": { "user.tenant_id": to_bson_uuid(tenant_id) } },
            // Project back to AccessRequest root fields only?
            // "replaceRoot"? Or simple map.
            doc! {
                "$project": {
                    "user": 0 // remove joined field to match struct
                }
            }
        ];

        let cursor = self.access_requests().aggregate(pipeline).await?;
        // Cursor returns Documents, need to deserialize.
        // aggregate returns Cursor<Document>.
        let docs: Vec<Document> = cursor.try_collect().await?;
        
        let mut reqs = Vec::new();
        for d in docs {
             reqs.push(mongodb::bson::from_document(d)?);
        }
        Ok(reqs)
    }

    async fn update_access_request(&self, request: AccessRequest) -> Result<()> {
        let filter = doc! { "id": to_bson_uuid(request.id) };
        self.access_requests().replace_one(filter, request).await?;
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
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "namespace": namespace,
            "name": table
        };
        
        // CAS Logic
        let mut query = filter.clone();
        if let Some(expected) = expected_location {
            query.insert("properties.metadata_location", expected);
        } else {
            // expected is None, meaning it shouldn't exist or should be null.
            // In Mongo, we can check { $exists: false } or { $eq: null }
            query.insert("properties.metadata_location", doc! { "$exists": false });
        }
        
        let update = doc! {
            "$set": {
                "properties.metadata_location": new_location
            }
        };
        
        let result = self.db.collection::<Document>("assets").update_one(query, update).await?;
        
        if result.matched_count == 0 {
             return Err(anyhow::anyhow!("CAS check failed: Metadata location mismatch or asset not found"));
        }
        
        Ok(())
    }

    // Token Revocation Operations
    async fn revoke_token(&self, token_id: Uuid, expires_at: chrono::DateTime<chrono::Utc>, reason: Option<String>) -> Result<()> {
        let revoked = pangolin_core::token::RevokedToken::new(token_id, expires_at, reason);
        self.db.collection("revoked_tokens").insert_one(revoked).await?;
        Ok(())
    }

    async fn is_token_revoked(&self, token_id: Uuid) -> Result<bool> {
        let filter = doc! { "token_id": to_bson_uuid(token_id) };
        let result = self.db.collection::<pangolin_core::token::RevokedToken>("revoked_tokens")
            .find_one(filter)
            .await?;
        Ok(result.is_some())
    }

    async fn cleanup_expired_tokens(&self) -> Result<usize> {
        let now = chrono::Utc::now();
        let filter = doc! { "expires_at": { "$lt": now } };
        let result = self.db.collection::<pangolin_core::token::RevokedToken>("revoked_tokens")
            .delete_many(filter)
            .await?;
        Ok(result.deleted_count as usize)
    }
}


#[async_trait]
impl Signer for MongoStore {
    async fn get_table_credentials(&self, location: &str) -> Result<Credentials> {
         // Attempt to extract bucket/container from location
         let (_scheme, container) = if location.starts_with("s3://") {
             ("s3", location[5..].split('/').next().unwrap_or("").to_string())
         } else if location.starts_with("az://") {
             ("az", location[5..].split('/').next().unwrap_or("").to_string())
         } else if location.starts_with("gs://") {
             ("gs", location[5..].split('/').next().unwrap_or("").to_string())
         } else if location.starts_with("abfs://") {
             ("abfs", location[7..].split('/').next().unwrap_or("").split('@').next().unwrap_or("").to_string())
         } else {
             ("unknown", String::new()) 
         };

         // Find warehouse matching this container
         let filter = doc! {
             "$or": [
                 { "storage_config.s3.bucket": &container },
                 { "storage_config.azure.container": &container },
                 { "storage_config.gcp.bucket": &container }
             ]
         };
         
         let warehouse = self.warehouses().find_one(filter).await?
             .ok_or_else(|| anyhow::anyhow!("No warehouse found for location: {}", location))?;

         match &warehouse.vending_strategy {
             Some(VendingStrategy::AwsSts { role_arn: _, external_id: _ }) => {
                 Err(anyhow::anyhow!("AWS STS vending not implemented yet via VendingStrategy in MongoStore"))
             }
             Some(VendingStrategy::AwsStatic { access_key_id, secret_access_key }) => {
                 Ok(Credentials::Aws {
                     access_key_id: access_key_id.clone(),
                     secret_access_key: secret_access_key.clone(),
                     session_token: None,
                     expiration: None,
                 })
             }
             Some(VendingStrategy::AzureSas { account_name, account_key }) => {
                 let signer = crate::azure_signer::AzureSigner::new(account_name.clone(), account_key.clone());
                 let sas_token = signer.generate_sas_token(location).await?;
                 Ok(Credentials::Azure {
                     sas_token,
                     account_name: account_name.clone(),
                     expiration: chrono::Utc::now() + chrono::Duration::hours(1),
                 })
             }
             Some(VendingStrategy::GcpDownscoped { service_account_email, private_key }) => {
                 let signer = crate::gcp_signer::GcpSigner::new(service_account_email.clone(), private_key.clone());
                 let access_token = signer.generate_downscoped_token(location).await?;
                 Ok(Credentials::Gcp {
                     access_token,
                     expiration: chrono::Utc::now() + chrono::Duration::hours(1),
                 })
             }
             Some(VendingStrategy::None) => Err(anyhow::anyhow!("Vending disabled")),
             None => {
                 // Backward compatibility logic
                 let access_key = warehouse.storage_config.get("s3.access-key-id")
                    .ok_or_else(|| anyhow::anyhow!("Missing s3.access-key-id"))?;
                 let secret_key = warehouse.storage_config.get("s3.secret-access-key")
                    .ok_or_else(|| anyhow::anyhow!("Missing s3.secret-access-key"))?;
                    
                 if warehouse.use_sts {
                      // Existing STS Logic restored for backward compatibility
                      let region = warehouse.storage_config.get("s3.region")
                          .map(|s| s.as_str())
                          .unwrap_or("us-east-1");
                          
                      let endpoint = warehouse.storage_config.get("s3.endpoint")
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
                      
                      let role_arn = warehouse.storage_config.get("s3.role-arn").map(|s| s.as_str());
                  
                      if let Some(arn) = role_arn {
                           let resp = client.assume_role()
                              .role_arn(arn)
                              .role_session_name("pangolin-mongo-legacy")
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
    }

    async fn presign_get(&self, _location: &str) -> Result<String> {
        Err(anyhow::anyhow!("MongoStore does not support presigning yet"))
    }
}

fn to_bson_uuid(id: Uuid) -> Bson {
    Bson::Binary(Binary {
        subtype: BinarySubtype::Generic,
        bytes: id.as_bytes().to_vec(),
    })
}

fn from_bson_uuid(bson: &Bson) -> Result<Uuid> {
    match bson {
        Bson::Binary(Binary { subtype: BinarySubtype::Generic, bytes }) => {
            Ok(Uuid::from_slice(bytes)?)
        },
        _ => Err(anyhow::anyhow!("Invalid UUID bson")),
    }
}

impl MongoStore {
    pub async fn create_user(&self, user: User) -> Result<()> {
        self.users().insert_one(user).await?;
        Ok(())
    }

    pub async fn get_user(&self, id: Uuid) -> Result<Option<User>> {
        let filter = doc! { "id": to_bson_uuid(id) };
        let user = self.users().find_one(filter).await?;
        Ok(user)
    }

    pub async fn list_users(&self, tenant_id: Option<Uuid>) -> Result<Vec<User>> {
        let filter = if let Some(tid) = tenant_id {
            doc! { "tenant_id": to_bson_uuid(tid) }
        } else {
            doc! {}
        };
        let cursor = self.users().find(filter).await?;
        let users: Vec<User> = cursor.try_collect().await?;
        Ok(users)
    }
}
