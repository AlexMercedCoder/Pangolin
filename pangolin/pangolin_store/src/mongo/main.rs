use mongodb::{Client, Collection, Database};
use mongodb::options::ClientOptions;
use mongodb::bson::{doc, Document, Bson, Binary};
use mongodb::bson::spec::BinarySubtype;
use anyhow::Result;
use uuid::Uuid;
use pangolin_core::model::*;
use pangolin_core::user::*;
use pangolin_core::permission::*;
use pangolin_core::business_metadata::*;
use pangolin_core::audit::*;

#[derive(Clone)]
pub struct MongoStore {
    pub(crate) client: Client,
    pub(crate) db: Database,
    pub(crate) object_store_cache: crate::ObjectStoreCache,
    pub(crate) metadata_cache: crate::MetadataCache,
}

impl MongoStore {
    pub async fn new(connection_string: &str, database_name: &str) -> Result<Self> {
        let mut client_options = ClientOptions::parse(connection_string).await?;
        
        // Configure connection pool from environment variables
        if let Ok(max_pool_size) = std::env::var("MONGO_MAX_POOL_SIZE") {
            if let Ok(size) = max_pool_size.parse::<u32>() {
                client_options.max_pool_size = Some(size);
                tracing::info!("MongoDB max pool size set to: {}", size);
            }
        }
        
        if let Ok(min_pool_size) = std::env::var("MONGO_MIN_POOL_SIZE") {
            if let Ok(size) = min_pool_size.parse::<u32>() {
                client_options.min_pool_size = Some(size);
                tracing::info!("MongoDB min pool size set to: {}", size);
            }
        }

        // Set app name
        client_options.app_name = Some("Pangolin".to_string());
        
        let client = Client::with_options(client_options)?;
        let db = client.database(database_name);
        Ok(Self { 
            client, 
            db,
            object_store_cache: crate::ObjectStoreCache::default(),
            metadata_cache: crate::MetadataCache::default(),
        })
    }

    pub(crate) fn tenants(&self) -> Collection<Tenant> {
        self.db.collection("tenants")
    }

    pub(crate) fn warehouses(&self) -> Collection<Warehouse> {
        self.db.collection("warehouses")
    }

    pub(crate) fn catalogs(&self) -> Collection<Catalog> {
        self.db.collection("catalogs")
    }

    pub(crate) fn namespaces(&self) -> Collection<Namespace> {
        self.db.collection("namespaces")
    }

    pub(crate) fn assets(&self) -> Collection<Asset> {
        self.db.collection("assets")
    }

    pub(crate) fn branches(&self) -> Collection<Branch> {
        self.db.collection("branches")
    }

    pub(crate) fn tags(&self) -> Collection<Tag> {
        self.db.collection("tags")
    }

    pub(crate) fn commits(&self) -> Collection<Commit> {
        self.db.collection("commits")
    }

    pub(crate) fn audit_logs(&self) -> Collection<AuditLogEntry> {
        self.db.collection("audit_logs")
    }
    
    pub(crate) fn users(&self) -> Collection<User> {
        self.db.collection("users")
    }
    
    pub(crate) fn roles(&self) -> Collection<Role> {
        self.db.collection("roles")
    }
    
    pub(crate) fn user_roles(&self) -> Collection<pangolin_core::permission::UserRole> {
        self.db.collection("user_roles")
    }
    
    pub(crate) fn permissions(&self) -> Collection<Permission> {
        self.db.collection("permissions")
    }

    pub(crate) fn access_requests(&self) -> Collection<AccessRequest> {
        self.db.collection("access_requests")
    }

    pub(crate) fn business_metadata(&self) -> Collection<BusinessMetadata> {
        self.db.collection("business_metadata")
    }

    pub(crate) fn active_tokens(&self) -> Collection<Document> {
        self.db.collection("active_tokens")
    }

    pub(crate) fn system_settings(&self) -> Collection<Document> {
        self.db.collection("system_settings")
    }

    pub(crate) fn federated_sync_stats(&self) -> Collection<Document> {
        self.db.collection("federated_sync_stats")
    }

    pub(crate) fn merge_operations(&self) -> Collection<Document> {
        self.db.collection("merge_operations")
    }

    pub(crate) fn merge_conflicts(&self) -> Collection<Document> {
        self.db.collection("merge_conflicts")
    }

    pub(crate) fn service_users(&self) -> Collection<Document> {
        self.db.collection("service_users")
    }

    // Maintenance Operations
    pub async fn expire_snapshots(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _namespace: Vec<String>, _table: String, _retention_ms: i64) -> Result<()> {
        Ok(())
    }

    pub async fn remove_orphan_files(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _namespace: Vec<String>, _table: String, _older_than_ms: i64) -> Result<()> {
        Ok(())
    }

    // Metadata Location Operations
    pub async fn get_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String) -> Result<Option<String>> {
        if let Some(asset) = self.get_asset(tenant_id, catalog_name, branch, namespace, table).await? {
            Ok(asset.properties.get("metadata_location").cloned())
        } else {
            Ok(None)
        }
    }

    pub async fn update_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String, _expected_location: Option<String>, new_location: String) -> Result<()> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "branch": branch.unwrap_or_else(|| "main".to_string()),
            "namespace": namespace,
            "name": table
        };
        
        let update = doc! {
            "$set": {
                "properties.metadata_location": new_location
            }
        };
        
        self.db.collection::<Document>("assets").update_one(filter, update).await?;
        Ok(())
    }

}

pub(crate) fn to_bson_uuid(id: Uuid) -> Bson {
    Bson::Binary(Binary {
        subtype: BinarySubtype::Generic,
        bytes: id.as_bytes().to_vec(),
    })
}

pub(crate) fn from_bson_uuid(bson: &Bson) -> Result<Uuid> {
    match bson {
        Bson::Binary(Binary { subtype: BinarySubtype::Generic, bytes }) => {
            Ok(Uuid::from_slice(bytes)?)
        },
        _ => Err(anyhow::anyhow!("Invalid UUID bson")),
    }
}
