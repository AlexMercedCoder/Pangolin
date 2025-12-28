use super::MongoStore;
use super::main::to_bson_uuid;
use anyhow::Result;
use mongodb::bson::{doc, Document, Bson};
use pangolin_core::audit::{AuditLogEntry, AuditLogFilter};
use uuid::Uuid;
use futures::stream::TryStreamExt;

impl MongoStore {
    pub async fn log_audit_event(&self, entry: AuditLogEntry) -> Result<()> {
        let mut doc = mongodb::bson::to_document(&entry)?;
        // Ensure UUIDs are stored as Binary
        doc.insert("id", to_bson_uuid(entry.id));
        doc.insert("tenant_id", to_bson_uuid(entry.tenant_id));
        let user_id = entry.user_id.unwrap_or(Uuid::nil());
        doc.insert("user_id", to_bson_uuid(user_id));
        
        // We use the raw collection to insert the document
        self.db.collection::<Document>("audit_logs").insert_one(doc).await?;
        Ok(())
    }

    pub async fn get_audit_event(&self, id: Uuid) -> Result<Option<AuditLogEntry>> {
        let filter = doc! { "id": to_bson_uuid(id) };
        let doc = self.db.collection::<AuditLogEntry>("audit_logs").find_one(filter).await?;
        Ok(doc)
    }

    pub async fn count_audit_events(&self, tenant_id: Uuid, filter: Option<AuditLogFilter>) -> Result<usize> {
        let mongo_filter = self.build_audit_filter(tenant_id, filter)?;
        let count = self.db.collection::<Document>("audit_logs").count_documents(mongo_filter).await?;
        Ok(count as usize)
    }

    pub async fn list_audit_events(&self, tenant_id: Uuid, filter: Option<AuditLogFilter>) -> Result<Vec<AuditLogEntry>> {
        let mongo_filter = self.build_audit_filter(tenant_id, filter)?;
        let cursor = self.db.collection::<AuditLogEntry>("audit_logs").find(mongo_filter).await?;
        let entries: Vec<AuditLogEntry> = cursor.try_collect().await?;
        Ok(entries)
    }

    fn build_audit_filter(&self, tenant_id: Uuid, filter: Option<AuditLogFilter>) -> Result<Document> {
        let mut mongo_filter = doc! { "tenant_id": to_bson_uuid(tenant_id) };
        if let Some(f) = filter {
            if let Some(rt) = f.resource_type {
                mongo_filter.insert("resource_type", format!("{:?}", rt));
            }
            if let Some(ra) = f.action {
                mongo_filter.insert("action", format!("{:?}", ra));
            }
            if let Some(uid) = f.user_id {
                mongo_filter.insert("user_id", to_bson_uuid(uid));
            }
            if let Some(from) = f.start_time {
                mongo_filter.insert("timestamp", doc! { "$gte": Bson::DateTime(from.into()) });
            }
            if let Some(to) = f.end_time {
                if let Some(ts_filter) = mongo_filter.get_mut("timestamp") {
                    if let Some(ts_doc) = ts_filter.as_document_mut() {
                        ts_doc.insert("$lte", Bson::DateTime(to.into()));
                    }
                } else {
                    mongo_filter.insert("timestamp", doc! { "$lte": Bson::DateTime(to.into()) });
                }
            }
        }
        
        Ok(mongo_filter)
    }
}
