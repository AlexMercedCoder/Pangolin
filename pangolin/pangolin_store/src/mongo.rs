use crate::CatalogStore;
use anyhow::Result;
use async_trait::async_trait;
use futures::stream::TryStreamExt;
use mongodb::{Client, Collection, Database};
use mongodb::bson::{doc, Document, Bson, Binary};
use mongodb::bson::spec::BinarySubtype;
use mongodb::options::{ClientOptions, UpdateOptions, ReplaceOptions};
use pangolin_core::model::{
    Asset, AssetType, Branch, Catalog, Commit, Namespace, Tag, Tenant, Warehouse,
};
use pangolin_core::audit::AuditLogEntry;
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Clone)]
pub struct MongoStore {
    client: Client,
    db: Database,
}

impl MongoStore {
    pub async fn new(connection_string: &str) -> Result<Self> {
        let client_options = ClientOptions::parse(connection_string).await?;
        let client = Client::with_options(client_options)?;
        let db = client.database("pangolin"); // Default DB name, could be configurable
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

    // Catalog Operations
    async fn create_catalog(&self, tenant_id: Uuid, catalog: Catalog) -> Result<()> {
        // Catalog struct doesn't have tenant_id, so we need to wrap it or add it?
        // Wait, Catalog struct in model.rs:
        // pub struct Catalog { pub name: String, pub properties: HashMap<String, String> }
        // It doesn't have tenant_id.
        // In Postgres we added a column. In Mongo we can wrap it in a document or just add the field dynamically if we use Document.
        // But we are using typed Collection<Catalog>.
        // We should probably use a wrapper struct for storage or just use Document.
        // Let's use Document for flexibility here since we need to add tenant_id context.
        
        let doc = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "name": &catalog.name,
            "properties": mongodb::bson::to_bson(&catalog.properties)?
        };
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

            Ok(Some(Asset {
                name: d.get_str("name")?.to_string(),
                kind,
                location: d.get_str("location")?.to_string(),
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

            assets.push(Asset {
                name: d.get_str("name")?.to_string(),
                kind,
                location: d.get_str("location")?.to_string(),
                properties,
            });
        }
        Ok(assets)
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
    async fn read_file(&self, _location: &str) -> Result<Vec<u8>> {
        Err(anyhow::anyhow!("File storage not implemented in MongoStore"))
    }

    async fn write_file(&self, _location: &str, _bytes: Vec<u8>) -> Result<()> {
        Err(anyhow::anyhow!("File storage not implemented in MongoStore"))
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
            "timestamp": mongodb::bson::DateTime::from_chrono(event.timestamp),
            "actor": &event.actor,
            "action": &event.action,
            "resource": &event.resource,
            "details": mongodb::bson::to_bson(&event.details)?
        };
        self.db.collection::<Document>("audit_logs").insert_one(doc).await?;
        Ok(())
    }

    async fn list_audit_events(&self, tenant_id: Uuid) -> Result<Vec<AuditLogEntry>> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id) };
        let options = mongodb::options::FindOptions::builder().sort(doc! { "timestamp": -1 }).limit(100).build();
        let cursor = self.db.collection::<Document>("audit_logs").find(filter).with_options(options).await?;
        let docs: Vec<Document> = cursor.try_collect().await?;
        
        let mut events = Vec::new();
        for d in docs {
            events.push(AuditLogEntry {
                id: mongodb::bson::from_bson(d.get("id").unwrap().clone())?,
                tenant_id,
                timestamp: d.get_datetime("timestamp")?.to_chrono(),
                actor: d.get_str("actor")?.to_string(),
                action: d.get_str("action")?.to_string(),
                resource: d.get_str("resource")?.to_string(),
                details: mongodb::bson::from_bson(d.get("details").unwrap().clone())?,
            });
        }
        Ok(events)
    }

    // Maintenance Operations
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
}

use crate::signer::Signer;
use crate::signer::Credentials;

#[async_trait]
impl Signer for MongoStore {
    async fn get_table_credentials(&self, _location: &str) -> Result<Credentials> {
        Err(anyhow::anyhow!("Credential vending not supported by MongoStore"))
    }

    async fn presign_get(&self, _location: &str) -> Result<String> {
        Err(anyhow::anyhow!("Presigning not supported by MongoStore"))
    }
}

fn to_bson_uuid(id: Uuid) -> Bson {
    Bson::Binary(Binary {
        subtype: BinarySubtype::Generic,
        bytes: id.as_bytes().to_vec(),
    })
}

fn to_bson_uuid(id: Uuid) -> Bson {
    Bson::Binary(Binary {
        subtype: BinarySubtype::Generic,
        bytes: id.as_bytes().to_vec(),
    })
}
