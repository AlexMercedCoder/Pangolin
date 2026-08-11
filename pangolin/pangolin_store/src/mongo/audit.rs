use super::main::to_bson_uuid;
use super::MongoStore;
use anyhow::Result;
use futures::stream::TryStreamExt;
use mongodb::bson::{doc, Bson, Document};
use pangolin_core::audit::{AuditLogEntry, AuditLogFilter};
use uuid::Uuid;

/// Default cap on a listing, matching the SQL backends' `LIMIT 100`.
const DEFAULT_AUDIT_LIMIT: usize = 100;

impl MongoStore {
    pub async fn log_audit_event(&self, entry: AuditLogEntry) -> Result<()> {
        let mut doc = mongodb::bson::to_document(&entry)?;
        // Every UUID field has to be stored as BSON Binary, because that is
        // what `Uuid`'s non-human-readable Deserialize expects on the way back.
        // `resource_id` was left as whatever `to_document` produced, so reading
        // an entry that had one failed with `invalid type: string ...,
        // expected bytes` and took the whole listing down with it.
        doc.insert("id", to_bson_uuid(entry.id));
        doc.insert("tenant_id", to_bson_uuid(entry.tenant_id));
        let user_id = entry.user_id.unwrap_or(Uuid::nil());
        doc.insert("user_id", to_bson_uuid(user_id));
        match entry.resource_id {
            Some(resource_id) => doc.insert("resource_id", to_bson_uuid(resource_id)),
            None => doc.insert("resource_id", Bson::Null),
        };

        // We use the raw collection to insert the document
        self.db
            .collection::<Document>("audit_logs")
            .insert_one(doc)
            .await?;
        Ok(())
    }

    /// Fetch one audit event, scoped to its tenant.
    ///
    /// B1: the filter was `{ "id": ... }` alone and the caller's `tenant_id`
    /// was discarded, so any tenant holding an audit-event UUID could read
    /// another tenant's audit record - username, IP, resource names, metadata.
    /// Postgres and SQLite both scoped by tenant; only Mongo did not.
    pub async fn get_audit_event(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Option<AuditLogEntry>> {
        let filter = doc! {
            "id": to_bson_uuid(id),
            "tenant_id": to_bson_uuid(tenant_id),
        };
        let doc = self
            .db
            .collection::<AuditLogEntry>("audit_logs")
            .find_one(filter)
            .await?;
        Ok(doc)
    }

    pub async fn count_audit_events(
        &self,
        tenant_id: Uuid,
        filter: Option<AuditLogFilter>,
    ) -> Result<usize> {
        let mongo_filter = self.build_audit_filter(tenant_id, filter)?;
        let count = self
            .db
            .collection::<Document>("audit_logs")
            .count_documents(mongo_filter)
            .await?;
        Ok(count as usize)
    }

    pub async fn list_audit_events(
        &self,
        tenant_id: Uuid,
        filter: Option<AuditLogFilter>,
    ) -> Result<Vec<AuditLogEntry>> {
        // B23: this applied no sort, no limit and no offset while the SQL
        // backends used `ORDER BY timestamp DESC LIMIT 100`. On a busy tenant
        // Mongo streamed the entire audit collection into memory and returned
        // it in storage order.
        let (limit, offset) = filter
            .as_ref()
            .map(|f| {
                (
                    f.limit.unwrap_or(DEFAULT_AUDIT_LIMIT),
                    f.offset.unwrap_or(0),
                )
            })
            .unwrap_or((DEFAULT_AUDIT_LIMIT, 0));

        let mongo_filter = self.build_audit_filter(tenant_id, filter)?;
        let cursor = self
            .db
            .collection::<AuditLogEntry>("audit_logs")
            .find(mongo_filter)
            .sort(doc! { "timestamp": -1 })
            .skip(offset as u64)
            .limit(limit as i64)
            .await?;
        let entries: Vec<AuditLogEntry> = cursor.try_collect().await?;
        Ok(entries)
    }

    fn build_audit_filter(
        &self,
        tenant_id: Uuid,
        filter: Option<AuditLogFilter>,
    ) -> Result<Document> {
        let mut mongo_filter = doc! { "tenant_id": to_bson_uuid(tenant_id) };
        if let Some(f) = filter {
            // B23: these used `format!("{:?}", ..)` - the Debug spelling,
            // `"CreateBranch"` - against documents serde wrote in snake_case
            // (`"create_branch"`). The filters could never match, so an
            // action- or resource-type-filtered listing always returned zero
            // rows and `count_audit_events` always returned 0. Going through
            // `bson::to_bson` uses the same serde naming as the write path.
            if let Some(rt) = f.resource_type {
                mongo_filter.insert("resource_type", mongodb::bson::to_bson(&rt)?);
            }
            if let Some(ra) = f.action {
                mongo_filter.insert("action", mongodb::bson::to_bson(&ra)?);
            }
            if let Some(uid) = f.user_id {
                mongo_filter.insert("user_id", to_bson_uuid(uid));
            }
            // B23: `resource_id` and `result` were accepted by the filter type
            // and then silently ignored.
            if let Some(rid) = f.resource_id {
                mongo_filter.insert("resource_id", to_bson_uuid(rid));
            }
            if let Some(result) = f.result {
                mongo_filter.insert("result", mongodb::bson::to_bson(&result)?);
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
