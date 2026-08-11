//! Service users (API-key principals) for the MongoDB backend.
//!
//! Every method here was broken, in three overlapping ways, and none of it was
//! visible from reading a single function:
//!
//! * **The lookup key did not exist.** `create_service_user` inserted the serde
//!   document as-is, so Mongo generated an `ObjectId` for `_id`. The four
//!   by-id methods then filtered on `{"_id": id.to_string()}`, which matched
//!   nothing, ever. Fetching, updating, deleting and touching a service user
//!   were all no-ops - `update_service_user` reported "Service user not found"
//!   for a user that had just been created.
//! * **The field names were wrong.** `ServiceUser` is `rename_all =
//!   "kebab-case"`, so the stored fields are `tenant-id` and `api-key-hash`.
//!   The filters used `tenant_id` and `api_key_hash`, so listing returned
//!   nothing and - the one that matters - **API-key authentication could never
//!   resolve a service user**. It fails closed, so this was a total outage of
//!   service-user auth on Mongo rather than a bypass.
//! * **The UUIDs were the wrong type.** The usual asymmetry: `to_document`
//!   writes a `Uuid` as a string, while the deserializer and `to_bson_uuid`
//!   filters want BSON Binary.
//!
//! Found by the entity round-trip suite, which asserts that every UUID-bearing
//! collection can be written and read back.
//!
//! Note that the `chrono` fields go the *other* way: bson's serializer reports
//! itself human-readable, so `to_document` writes them as RFC3339 strings and
//! that is what `from_document` reads back. `update_service_user_last_used`
//! must therefore write a string too - a `Bson::DateTime` there would be
//! written happily and then break every subsequent read.

use super::main::{to_bson_uuid, with_binary_uuids};
use super::MongoStore;
use anyhow::Result;
use futures::stream::TryStreamExt;
use mongodb::bson::{doc, Document};
use pangolin_core::user::{ServiceUser, UserRole};
use uuid::Uuid;

impl MongoStore {
    pub async fn create_service_user(&self, service_user: ServiceUser) -> Result<()> {
        let doc = with_binary_uuids(
            mongodb::bson::to_document(&service_user)?,
            &[
                ("id", service_user.id),
                ("tenant-id", service_user.tenant_id),
                ("created-by", service_user.created_by),
            ],
        );
        self.service_users().insert_one(doc).await?;
        Ok(())
    }

    pub async fn get_service_user(&self, id: Uuid) -> Result<Option<ServiceUser>> {
        let filter = doc! { "id": to_bson_uuid(id) };
        if let Some(doc) = self.service_users().find_one(filter).await? {
            Ok(Some(mongodb::bson::from_document(doc)?))
        } else {
            Ok(None)
        }
    }

    /// Resolve an API key to its principal.
    ///
    /// This is the authentication path. The hash is stored verbatim, so an
    /// exact match is right; only the field name was wrong.
    pub async fn get_service_user_by_api_key_hash(
        &self,
        api_key_hash: &str,
    ) -> Result<Option<ServiceUser>> {
        let filter = doc! { "api-key-hash": api_key_hash };
        if let Some(doc) = self.service_users().find_one(filter).await? {
            Ok(Some(mongodb::bson::from_document(doc)?))
        } else {
            Ok(None)
        }
    }

    pub async fn list_service_users(
        &self,
        tenant_id: Uuid,
        pagination: Option<crate::PaginationParams>,
    ) -> Result<Vec<ServiceUser>> {
        let filter = doc! { "tenant-id": to_bson_uuid(tenant_id) };

        let collection = self.service_users();
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
        let docs: Vec<Document> = cursor.try_collect().await?;
        let mut users = Vec::new();
        for doc in docs {
            users.push(mongodb::bson::from_document(doc)?);
        }
        Ok(users)
    }

    pub async fn update_service_user(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        role: Option<UserRole>,
        active: Option<bool>,
    ) -> Result<ServiceUser> {
        let filter = doc! { "id": to_bson_uuid(id) };
        let mut update_doc = doc! {};

        if let Some(n) = name {
            update_doc.insert("name", n);
        }
        if let Some(d) = description {
            update_doc.insert("description", d);
        }
        if let Some(r) = role {
            // `format!("{:?}", r)` wrote the Rust variant name (`TenantAdmin`).
            // `UserRole` is kebab-case, so the record then failed to
            // deserialize: changing a service user's role made it unreadable.
            update_doc.insert("role", mongodb::bson::to_bson(&r)?);
        }
        if let Some(a) = active {
            update_doc.insert("active", a);
        }

        if !update_doc.is_empty() {
            let update = doc! { "$set": update_doc };
            self.service_users().update_one(filter, update).await?;
        }

        self.get_service_user(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Service user not found"))
    }

    pub async fn delete_service_user(&self, id: Uuid) -> Result<()> {
        let filter = doc! { "id": to_bson_uuid(id) };
        self.service_users().delete_one(filter).await?;
        Ok(())
    }

    /// Stamp the last time this key was used.
    ///
    /// The field is `last-used`; `last_used_at` was a field no reader ever
    /// looked at, so the timestamp shown for every service user stayed at its
    /// creation value forever. See the module note on why this is a string.
    pub async fn update_service_user_last_used(&self, id: Uuid) -> Result<()> {
        let filter = doc! { "id": to_bson_uuid(id) };
        let update = doc! { "$set": { "last-used": chrono::Utc::now().to_rfc3339() } };
        self.service_users().update_one(filter, update).await?;
        Ok(())
    }
}
