use super::main::to_bson_uuid;
use super::MongoStore;
use anyhow::Result;
use futures::stream::TryStreamExt;
use mongodb::bson::{doc, Document};
use pangolin_core::model::{Catalog, CatalogUpdate};
use uuid::Uuid;

/// Why a transactional catalog delete stopped.
enum CatalogDeleteError {
    /// The catalog does not exist.
    NotFound,
    /// The deployment has no transaction support - a standalone `mongod`.
    ///
    /// MongoDB reports this as `IllegalOperation` (code 20) with "Transaction
    /// numbers are only allowed on a replica set member or mongos", and only
    /// once the first operation inside the transaction reaches the server.
    TransactionsUnsupported(mongodb::error::Error),
    Other(anyhow::Error),
}

impl CatalogDeleteError {
    fn from_mongo(e: mongodb::error::Error) -> Self {
        // MongoDB raises `IllegalOperation` for "Transaction numbers are only
        // allowed on a replica set member or mongos". The driver replaces that
        // server text with its own "does not support retryable writes" wording,
        // so matching on the message is unreliable - the code is the stable
        // signal. This classifier only ever runs on failures from inside a
        // transaction attempt, where an `IllegalOperation` means the deployment
        // cannot do transactions; the cost of a false positive is a non-atomic
        // delete rather than a wrong result.
        const ILLEGAL_OPERATION: i32 = 20;

        if let mongodb::error::ErrorKind::Command(command_error) = &*e.kind {
            if command_error.code == ILLEGAL_OPERATION {
                return Self::TransactionsUnsupported(e);
            }
        }
        Self::Other(anyhow::anyhow!(e))
    }
}

impl MongoStore {
    pub async fn create_catalog(&self, tenant_id: Uuid, catalog: Catalog) -> Result<()> {
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
            doc.insert(
                "federated_config",
                mongodb::bson::to_bson(federated_config)?,
            );
        }

        self.db
            .collection::<Document>("catalogs")
            .insert_one(doc)
            .await?;
        Ok(())
    }

    pub async fn get_catalog(&self, tenant_id: Uuid, name: String) -> Result<Option<Catalog>> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id), "name": name };
        let doc = self
            .db
            .collection::<Catalog>("catalogs")
            .find_one(filter)
            .await?;
        Ok(doc)
    }

    pub async fn list_catalogs(
        &self,
        tenant_id: Uuid,
        pagination: Option<crate::PaginationParams>,
    ) -> Result<Vec<Catalog>> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id) };

        let collection = self.db.collection::<Catalog>("catalogs");
        let mut find = collection.find(filter);
        if let Some(p) = pagination {
            if let Some(l) = p.limit {
                find = find.limit(l as i64);
            }
            if let Some(o) = p.offset {
                find = find.skip(o as u64);
            }
        }

        let cursor = find.await?;
        let catalogs: Vec<Catalog> = cursor.try_collect().await?;
        Ok(catalogs)
    }

    pub async fn update_catalog(
        &self,
        tenant_id: Uuid,
        name: String,
        updates: CatalogUpdate,
    ) -> Result<Catalog> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id), "name": &name };
        let mut update_doc = doc! {};

        if let Some(warehouse_name) = updates.warehouse_name {
            update_doc.insert("warehouse_name", warehouse_name);
        }
        if let Some(storage_location) = updates.storage_location {
            update_doc.insert("storage_location", storage_location);
        }
        if let Some(properties) = updates.properties {
            update_doc.insert("properties", mongodb::bson::to_bson(&properties)?);
        }

        if update_doc.is_empty() {
            return self
                .get_catalog(tenant_id, name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Catalog not found"));
        }

        let update = doc! { "$set": update_doc };
        self.db
            .collection::<Document>("catalogs")
            .update_one(filter, update)
            .await?;

        self.get_catalog(tenant_id, name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Catalog not found"))
    }

    /// Delete a catalog and everything under it.
    ///
    /// Attempts the five deletes inside a transaction so a failure partway
    /// through does not leave a half-deleted catalog (A-24). MongoDB
    /// transactions require a replica set or a sharded cluster; against a
    /// standalone `mongod` the driver refuses to start a transaction, and this
    /// falls back to sequential deletes with a warning rather than failing the
    /// operation outright.
    pub async fn delete_catalog(&self, tenant_id: Uuid, name: String) -> Result<()> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id), "name": &name };
        let child_filter = doc! { "tenant_id": to_bson_uuid(tenant_id), "catalog_name": &name };

        let mut session = match self.client.start_session().await {
            Ok(session) => Some(session),
            Err(e) => {
                tracing::warn!(error = %e, "could not open a MongoDB session; deleting without a transaction");
                None
            }
        };

        if let Some(session) = session.as_mut() {
            // `start_transaction` is a *local* call in the Rust driver: it
            // allocates a transaction number and returns without contacting the
            // server, so it cannot fail for "this deployment has no transaction
            // support". The fallback below was therefore unreachable - the
            // topology error surfaced on the first operation *inside* the
            // transaction and propagated through `?`, failing the delete
            // outright on any standalone `mongod`. Found by running the parity
            // suite against a live standalone MongoDB.
            //
            // The transactional attempt now runs in full and a topology error
            // anywhere within it degrades to the sequential path, which is what
            // the original comment promised.
            if let Err(e) = session.start_transaction().await {
                tracing::warn!(
                    error = %e,
                    "could not begin a MongoDB transaction; deleting a catalog will not be atomic"
                );
                return self
                    .delete_catalog_unsafe(filter, child_filter, &name)
                    .await;
            }

            match self
                .delete_catalog_in_transaction(session, &filter, &child_filter)
                .await
            {
                Ok(()) => return Ok(()),
                Err(CatalogDeleteError::NotFound) => {
                    let _ = session.abort_transaction().await;
                    return Err(anyhow::anyhow!("Catalog '{}' not found", name));
                }
                Err(CatalogDeleteError::TransactionsUnsupported(e)) => {
                    tracing::warn!(
                        error = %e,
                        "this MongoDB deployment does not support transactions (a replica set \
                         is required); deleting a catalog will not be atomic"
                    );
                    let _ = session.abort_transaction().await;
                    // Falls through to the sequential path below.
                }
                Err(CatalogDeleteError::Other(e)) => {
                    let _ = session.abort_transaction().await;
                    return Err(e);
                }
            }
        }

        self.delete_catalog_unsafe(filter, child_filter, &name)
            .await
    }

    /// The transactional half of [`Self::delete_catalog`].
    ///
    /// Split out so a topology error can be told apart from a genuine failure
    /// and from "no such catalog" - the caller handles each differently.
    async fn delete_catalog_in_transaction(
        &self,
        session: &mut mongodb::ClientSession,
        filter: &Document,
        child_filter: &Document,
    ) -> std::result::Result<(), CatalogDeleteError> {
        for collection in ["tags", "branches", "assets", "namespaces"] {
            self.db
                .collection::<Document>(collection)
                .delete_many(child_filter.clone())
                .session(&mut *session)
                .await
                .map_err(CatalogDeleteError::from_mongo)?;
        }

        let result = self
            .db
            .collection::<Document>("catalogs")
            .delete_one(filter.clone())
            .session(&mut *session)
            .await
            .map_err(CatalogDeleteError::from_mongo)?;

        if result.deleted_count == 0 {
            return Err(CatalogDeleteError::NotFound);
        }

        session
            .commit_transaction()
            .await
            .map_err(CatalogDeleteError::from_mongo)?;

        Ok(())
    }

    /// Non-atomic fallback for deployments without transaction support.
    async fn delete_catalog_unsafe(
        &self,
        filter: Document,
        child_filter: Document,
        name: &str,
    ) -> Result<()> {
        // Existence is checked *first*. This path used to delete every matching
        // tag, branch, asset and namespace and only then discover the catalog
        // did not exist - returning "not found" to a caller who had every reason
        // to believe nothing had happened. That is B21, which was fixed for
        // SQLite during the roadmap work; the same shape survived here because
        // this branch only runs on a deployment without transactions, and
        // nothing had ever run it.
        //
        // Without a transaction the cascade below is still not atomic - a
        // failure partway through leaves a partial delete. Checking first at
        // least means a *no-op* call destroys nothing, which is the case an
        // operator is most likely to hit by typo.
        let exists = self
            .db
            .collection::<Document>("catalogs")
            .find_one(filter.clone())
            .await?
            .is_some();

        if !exists {
            return Err(anyhow::anyhow!("Catalog '{}' not found", name));
        }

        for collection in ["tags", "branches", "assets", "namespaces"] {
            self.db
                .collection::<Document>(collection)
                .delete_many(child_filter.clone())
                .await?;
        }

        self.db
            .collection::<Document>("catalogs")
            .delete_one(filter)
            .await?;

        Ok(())
    }
}
