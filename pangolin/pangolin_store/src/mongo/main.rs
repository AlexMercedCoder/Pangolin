use anyhow::Result;
use mongodb::bson::spec::BinarySubtype;
use mongodb::bson::{doc, Binary, Bson, Document};
use mongodb::options::ClientOptions;
use mongodb::{Client, Collection, Database};
use pangolin_core::audit::*;
use pangolin_core::business_metadata::*;
use pangolin_core::model::*;
use pangolin_core::permission::*;
use pangolin_core::user::*;
use uuid::Uuid;

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

        // Indexes. There used to be two here - commits(parent_id) and
        // active_tokens(user_id) - created with their errors thrown away by
        // `.ok()`, so everything else was a collection scan and a failure was
        // invisible. `super::indexes` holds the full set, derived from the
        // filters this backend actually issues, and reports what it could not
        // create.
        super::indexes::ensure_indexes(&db).await;

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
    pub async fn expire_snapshots(
        &self,
        _tenant_id: Uuid,
        _catalog_name: &str,
        _branch: Option<String>,
        _namespace: Vec<String>,
        _table: String,
        _retention_ms: i64,
    ) -> Result<()> {
        Ok(())
    }

    pub async fn remove_orphan_files(
        &self,
        _tenant_id: Uuid,
        _catalog_name: &str,
        _branch: Option<String>,
        _namespace: Vec<String>,
        _table: String,
        _older_than_ms: i64,
    ) -> Result<()> {
        Ok(())
    }

    // Metadata Location Operations
    pub async fn get_metadata_location(
        &self,
        tenant_id: Uuid,
        catalog_name: &str,
        branch: Option<String>,
        namespace: Vec<String>,
        table: String,
    ) -> Result<Option<String>> {
        if let Some(asset) = self
            .get_asset(tenant_id, catalog_name, branch, namespace, table)
            .await?
        {
            // Found by running the parity suite against a live MongoDB: this
            // read only `properties["metadata_location"]` and had no fallback to
            // the asset's own `location`, which is what memory, SQLite and
            // Postgres all fall back to. A table created with a location but no
            // explicit metadata-location property therefore reported *no*
            // metadata location on Mongo alone - so `load_table` could not find
            // its metadata and every commit's compare-and-swap was working from
            // a different notion of "current" than the read path.
            Ok(current_metadata_location(&asset))
        } else {
            Ok(None)
        }
    }

    /// Publish a new metadata location, but only if the current one still
    /// matches `expected_location`.
    ///
    /// B5: `expected_location` was ignored (`_expected_location`) and the update
    /// was an unconditional `$set`. Memory, Postgres and SQLite all enforce the
    /// compare-and-swap; on Mongo two concurrent Iceberg commits both
    /// "succeeded" and one snapshot was silently lost - the exact failure class
    /// the 0.6.0 work fixed at the API layer, still wide open one layer down.
    ///
    /// Folding the expectation into the *filter* keeps this a single-document
    /// atomic update, so it works on a standalone `mongod` with no multi-document
    /// transaction required.
    pub async fn update_metadata_location(
        &self,
        tenant_id: Uuid,
        catalog_name: &str,
        branch: Option<String>,
        namespace: Vec<String>,
        table: String,
        expected_location: Option<String>,
        new_location: String,
    ) -> Result<()> {
        let mut filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "branch": branch.unwrap_or_else(|| "main".to_string()),
            "namespace": namespace,
            "name": table
        };

        // The expectation has to be expressed against the *same* notion of
        // "current location" that `get_metadata_location` returns - property
        // first, then the asset's own `location`. Keeping it inside the filter
        // rather than reading-then-comparing preserves the single-document
        // atomicity that makes this a real CAS on a standalone mongod.
        match &expected_location {
            Some(expected) => {
                filter.insert(
                    "$or",
                    vec![
                        doc! { "properties.metadata_location": expected.clone() },
                        doc! { "$and": vec![
                            doc! { "properties.metadata_location": { "$exists": false } },
                            doc! { "location": expected.clone() },
                        ]},
                    ],
                );
            }
            // `None` means "there must not be one yet" - the create-path CAS.
            None => {
                filter.insert(
                    "$and",
                    vec![
                        doc! { "properties.metadata_location": { "$exists": false } },
                        doc! { "$or": vec![
                            doc! { "location": { "$exists": false } },
                            doc! { "location": "" },
                        ]},
                    ],
                );
            }
        }

        let update = doc! {
            "$set": {
                "properties.metadata_location": &new_location,
                "location": &new_location,
            }
        };

        let result = self
            .db
            .collection::<Document>("assets")
            .update_one(filter, update)
            .await?;

        if result.matched_count == 0 {
            return Err(anyhow::anyhow!(
                "CAS failure: metadata location did not match {:?}",
                expected_location
            ));
        }
        Ok(())
    }
}

/// The metadata location a reader would see for `asset`.
///
/// Property first, then the asset's own `location` when it is non-empty - the
/// same resolution memory, SQLite and Postgres use. Defined once so the read
/// path and the compare-and-swap cannot drift apart again.
fn current_metadata_location(asset: &pangolin_core::model::Asset) -> Option<String> {
    asset
        .properties
        .get("metadata_location")
        .cloned()
        .or_else(|| {
            if asset.location.is_empty() {
                None
            } else {
                Some(asset.location.clone())
            }
        })
}

/// Rewrite the named keys of a serde-produced document as BSON Binary UUIDs.
///
/// `bson::to_document` writes a `Uuid` as a *string*, but the driver's
/// deserializer expects Binary - so a document written through serde alone
/// cannot be read back into a struct with `Uuid` fields ("invalid type: string,
/// expected bytes"), and a filter built with [`to_bson_uuid`] never matches it.
///
/// That asymmetry is the single cause of the Mongo RBAC failures: role
/// assignments were written by serde and queried as Binary, so `get_user_roles`
/// always returned empty and every role-derived permission silently vanished.
/// The audit-log and token-revocation paths (B1, B2) were the same bug in two
/// other collections.
///
/// The keys given are the *serialized* names - kebab-case for these types - so
/// the document stays deserializable into its struct.
pub(crate) fn with_binary_uuids(
    mut doc: mongodb::bson::Document,
    fields: &[(&str, Uuid)],
) -> mongodb::bson::Document {
    for (key, value) in fields {
        doc.insert(*key, to_bson_uuid(*value));
    }
    doc
}

pub(crate) fn to_bson_uuid(id: Uuid) -> Bson {
    Bson::Binary(Binary {
        subtype: BinarySubtype::Generic,
        bytes: id.as_bytes().to_vec(),
    })
}

/// Decode a UUID that may have been written by any of the three routes.
///
/// There are three, and they disagree:
///
/// 1. [`to_bson_uuid`] - `Binary` with the *generic* subtype;
/// 2. `doc! { "k": some_uuid }` - `Binary` with the *UUID* subtype, via bson's
///    `From<Uuid> for Bson`;
/// 3. `bson::to_document` - a plain `String`.
///
/// Writes should use `to_bson_uuid` so new data is uniform, but reads have to
/// accept all three: documents written by the other two are already in
/// deployed databases. Being strict here is what made a branch with a head
/// commit unreadable - `create_branch` used route 2 and the reader accepted
/// only route 1.
pub(crate) fn from_bson_uuid(bson: &Bson) -> Result<Uuid> {
    match bson {
        Bson::Binary(Binary {
            subtype: BinarySubtype::Generic | BinarySubtype::Uuid,
            bytes,
        }) => Ok(Uuid::from_slice(bytes)?),
        Bson::String(s) => Ok(Uuid::parse_str(s)?),
        _ => Err(anyhow::anyhow!("Invalid UUID bson")),
    }
}

/// Decode an optional UUID field.
///
/// A missing key and an explicit `null` both mean "absent"; anything else has
/// to decode, because silently returning `None` for a value that is present
/// but unreadable would turn a corrupt record into a plausible-looking one.
///
/// `bson::from_bson::<Option<Uuid>>` cannot be used for this: handed a
/// `Bson::Binary` it reports `invalid type: map, expected a UUID string`,
/// because the deserializer presents binary data as the extended-JSON map
/// `{"$binary": ...}` while `Uuid`'s `Deserialize` wants a string.
pub(crate) fn read_optional_uuid(doc: &mongodb::bson::Document, key: &str) -> Result<Option<Uuid>> {
    match doc.get(key) {
        None | Some(Bson::Null) => Ok(None),
        Some(value) => from_bson_uuid(value).map(Some),
    }
}
