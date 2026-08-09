use super::main::to_bson_uuid;
use super::MongoStore;
use anyhow::Result;
use futures::stream::TryStreamExt;
use mongodb::bson::{doc, Document};
use pangolin_core::model::{Catalog, CatalogUpdate};
use uuid::Uuid;

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
            if let Err(e) = session.start_transaction().await {
                tracing::warn!(
                    error = %e,
                    "this MongoDB deployment does not support transactions (a replica set is \
                     required); deleting a catalog will not be atomic"
                );
                return self
                    .delete_catalog_unsafe(filter, child_filter, &name)
                    .await;
            }

            for collection in ["tags", "branches", "assets", "namespaces"] {
                self.db
                    .collection::<Document>(collection)
                    .delete_many(child_filter.clone())
                    .session(&mut *session)
                    .await?;
            }

            let result = self
                .db
                .collection::<Document>("catalogs")
                .delete_one(filter)
                .session(&mut *session)
                .await?;

            if result.deleted_count == 0 {
                let _ = session.abort_transaction().await;
                return Err(anyhow::anyhow!("Catalog '{}' not found", name));
            }

            session.commit_transaction().await?;
            return Ok(());
        }

        self.delete_catalog_unsafe(filter, child_filter, &name)
            .await
    }

    /// Non-atomic fallback for deployments without transaction support.
    async fn delete_catalog_unsafe(
        &self,
        filter: Document,
        child_filter: Document,
        name: &str,
    ) -> Result<()> {
        for collection in ["tags", "branches", "assets", "namespaces"] {
            self.db
                .collection::<Document>(collection)
                .delete_many(child_filter.clone())
                .await?;
        }

        let result = self
            .db
            .collection::<Document>("catalogs")
            .delete_one(filter)
            .await?;

        if result.deleted_count == 0 {
            return Err(anyhow::anyhow!("Catalog '{}' not found", name));
        }
        Ok(())
    }
}
