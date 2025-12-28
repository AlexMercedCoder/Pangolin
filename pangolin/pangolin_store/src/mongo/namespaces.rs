use super::MongoStore;
use super::main::to_bson_uuid;
use anyhow::Result;
use mongodb::bson::{doc, Document};
use pangolin_core::model::Namespace;
use uuid::Uuid;
use futures::stream::TryStreamExt;
use std::collections::HashMap;

impl MongoStore {
    pub async fn create_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Namespace) -> Result<()> {
        let doc = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "name": &namespace.name,
            "properties": mongodb::bson::to_bson(&namespace.properties)?
        };
        self.db.collection::<Document>("namespaces").insert_one(doc).await?;
        Ok(())
    }

    pub async fn get_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<Option<Namespace>> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "name": namespace
        };
        let doc = self.db.collection::<Namespace>("namespaces").find_one(filter).await?;
        Ok(doc)
    }

    pub async fn list_namespaces(&self, tenant_id: Uuid, catalog_name: &str, _parent: Option<String>, pagination: Option<crate::PaginationParams>) -> Result<Vec<Namespace>> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name
        };
        
        // TODO: Implement prefix filtering for 'parent' if needed

        let collection = self.db.collection::<Namespace>("namespaces");
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
        let namespaces: Vec<Namespace> = cursor.try_collect().await?;
        Ok(namespaces)
    }

    pub async fn delete_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<()> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "name": namespace
        };
        self.db.collection::<Document>("namespaces").delete_one(filter).await?;
        Ok(())
    }

    pub async fn update_namespace_properties(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>, properties: HashMap<String, String>) -> Result<()> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "name": namespace
        };
        
        let mut set_doc = doc! {};
        for (k, v) in properties {
            set_doc.insert(format!("properties.{}", k), v);
        }
        
        let update = doc! { "$set": set_doc };
        self.db.collection::<Document>("namespaces").update_one(filter, update).await?;
        Ok(())
    }

    pub async fn count_namespaces(&self, tenant_id: Uuid) -> Result<usize> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id) };
        let count = self.namespaces().count_documents(filter).await?;
        Ok(count as usize)
    }
}
