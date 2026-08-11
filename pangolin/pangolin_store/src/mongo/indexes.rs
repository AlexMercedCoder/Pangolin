//! Index management for the MongoDB backend.
//!
//! MongoDB had two indexes — `commits(parent_id)` and `active_tokens(user_id)`
//! — created at startup with their errors discarded by `.ok()`. Everything else
//! was a collection scan: every catalog lookup, every asset resolution on the
//! Iceberg commit path, every permission check. The other three backends get
//! these for free from primary keys and `UNIQUE` constraints; MongoDB is the
//! only one where the schema is whatever the first write happened to create.
//!
//! Two distinct things are being fixed here.
//!
//! **Performance.** The index set below is derived from the filters the code
//! actually issues, not from what seemed likely — see the field lists in each
//! entry. A missing index on `{tenant_id, name}` does not fail anything; it
//! just makes every catalog lookup read the whole collection, and that only
//! becomes visible under a load nobody ran.
//!
//! **Integrity.** Several of these are `unique`, which is the constraint the
//! SQL backends express as a primary key. Without it MongoDB will happily hold
//! two catalogs with the same name in one tenant, and which one a lookup
//! returns is arbitrary. That is a real parity gap, and it is why
//! `create_catalog` on MongoDB could not detect a duplicate the way Postgres
//! does.
//!
//! ## Failures are reported
//!
//! Creating a unique index on a collection that already contains duplicates
//! fails, and it should: silently continuing would leave the operator believing
//! a constraint exists. Each failure is logged with the collection, the keys and
//! what to do about it, and the count is returned so the caller can decide.
//! Startup does not abort — refusing to boot because an index is missing would
//! turn a performance problem into an outage — but it is impossible to miss in
//! the log.

use mongodb::bson::{doc, Document};
use mongodb::{Database, IndexModel};

/// One index this backend needs.
struct Index {
    collection: &'static str,
    keys: Document,
    unique: bool,
    /// Why it exists, for the log line when creation fails.
    reason: &'static str,
}

fn required_indexes() -> Vec<Index> {
    vec![
        // ---- Identity and tenancy ----
        Index {
            collection: "tenants",
            keys: doc! { "id": 1 },
            unique: true,
            reason: "every tenant lookup; the SQL backends have this as a primary key",
        },
        Index {
            collection: "users",
            keys: doc! { "id": 1 },
            unique: true,
            reason: "user lookup by id on every authenticated request",
        },
        Index {
            collection: "users",
            keys: doc! { "username": 1 },
            unique: false,
            reason: "login by username",
        },
        Index {
            collection: "users",
            keys: doc! { "email": 1 },
            unique: false,
            reason: "OAuth matches users on the provider-supplied email",
        },
        Index {
            collection: "users",
            keys: doc! { "tenant-id": 1 },
            unique: false,
            reason: "listing a tenant's users",
        },
        // ---- Catalog objects ----
        Index {
            collection: "catalogs",
            keys: doc! { "tenant_id": 1, "name": 1 },
            unique: true,
            reason: "catalog lookup, and the uniqueness Postgres gets from its primary key",
        },
        Index {
            collection: "warehouses",
            keys: doc! { "tenant_id": 1, "name": 1 },
            unique: true,
            reason: "warehouse lookup on every credential-vending request",
        },
        Index {
            collection: "namespaces",
            keys: doc! { "tenant_id": 1, "catalog_name": 1 },
            unique: false,
            reason: "listing namespaces in a catalog",
        },
        Index {
            collection: "assets",
            keys: doc! { "tenant_id": 1, "catalog_name": 1, "branch_name": 1 },
            unique: false,
            reason: "listing and copying a branch's assets",
        },
        Index {
            collection: "assets",
            keys: doc! { "tenant_id": 1, "id": 1 },
            unique: false,
            reason: "get_asset_by_id, used by the business-metadata join",
        },
        Index {
            collection: "branches",
            keys: doc! { "tenant_id": 1, "catalog_name": 1, "name": 1 },
            unique: true,
            reason: "branch lookup; two branches of one name in a catalog is corruption",
        },
        Index {
            collection: "tags",
            keys: doc! { "tenant_id": 1, "catalog_name": 1, "name": 1 },
            unique: true,
            reason: "tag lookup, and tag names must be unique within a catalog",
        },
        Index {
            collection: "commits",
            keys: doc! { "id": 1 },
            unique: true,
            reason: "commit lookup by id",
        },
        Index {
            collection: "commits",
            keys: doc! { "parent_id": 1 },
            unique: false,
            reason: "walking commit history",
        },
        // ---- Authorization ----
        //
        // These are the hot path: every request resolves the caller's roles and
        // permissions, so an unindexed scan here is paid on *every* request.
        Index {
            collection: "roles",
            keys: doc! { "id": 1 },
            unique: true,
            reason: "role lookup",
        },
        Index {
            collection: "roles",
            keys: doc! { "tenant-id": 1 },
            unique: false,
            reason: "listing a tenant's roles",
        },
        Index {
            collection: "user_roles",
            keys: doc! { "user-id": 1 },
            unique: false,
            reason: "resolving a caller's roles, on every authorized request",
        },
        Index {
            collection: "user_roles",
            keys: doc! { "role-id": 1 },
            unique: false,
            reason: "finding who holds a role, for revocation",
        },
        Index {
            collection: "permissions",
            keys: doc! { "user-id": 1 },
            unique: false,
            reason: "resolving direct grants, on every authorized request",
        },
        Index {
            collection: "permissions",
            keys: doc! { "tenant-id": 1 },
            unique: false,
            reason: "listing a tenant's grants",
        },
        // ---- Credentials and sessions ----
        Index {
            collection: "service_users",
            keys: doc! { "id": 1 },
            unique: true,
            reason: "service-user lookup",
        },
        Index {
            collection: "service_users",
            keys: doc! { "api-key-hash": 1 },
            unique: false,
            reason: "API-key authentication resolves the principal by hash",
        },
        Index {
            collection: "service_users",
            keys: doc! { "tenant-id": 1 },
            unique: false,
            reason: "API-key auth enumerates a tenant's service users",
        },
        Index {
            collection: "active_tokens",
            keys: doc! { "tenant_id": 1, "user_id": 1 },
            unique: false,
            reason: "listing a user's active sessions",
        },
        Index {
            collection: "active_tokens",
            keys: doc! { "token_id": 1 },
            unique: false,
            reason: "token lookup by jti",
        },
        Index {
            collection: "revoked_tokens",
            keys: doc! { "token_id": 1 },
            unique: true,
            reason: "the revocation check runs on every authenticated request",
        },
        Index {
            collection: "revoked_tokens",
            keys: doc! { "expires_at": 1 },
            unique: false,
            reason: "the cleanup job deletes by expiry",
        },
        // ---- Everything else ----
        Index {
            collection: "audit_logs",
            keys: doc! { "tenant_id": 1, "timestamp": -1 },
            unique: false,
            reason: "audit listing is tenant-scoped and newest-first",
        },
        Index {
            collection: "business_metadata",
            keys: doc! { "asset-id": 1 },
            unique: true,
            reason: "an asset has at most one metadata record",
        },
        Index {
            collection: "access_requests",
            keys: doc! { "id": 1 },
            unique: true,
            reason: "access-request lookup",
        },
        Index {
            collection: "access_requests",
            keys: doc! { "user-id": 1 },
            unique: false,
            reason: "the listing joins access_requests.user-id to users.id",
        },
        Index {
            collection: "merge_operations",
            keys: doc! { "id": 1 },
            unique: true,
            reason: "merge-operation lookup",
        },
        Index {
            collection: "merge_conflicts",
            keys: doc! { "merge_operation_id": 1 },
            unique: false,
            reason: "listing an operation's conflicts",
        },
        Index {
            collection: "system_settings",
            keys: doc! { "tenant_id": 1 },
            unique: true,
            reason: "one settings document per tenant",
        },
    ]
}

/// Create every index this backend needs. Returns how many could not be made.
///
/// Idempotent: MongoDB treats `createIndex` for an index that already exists
/// with the same specification as a no-op, so this runs on every startup.
pub(crate) async fn ensure_indexes(db: &Database) -> usize {
    let mut failures = 0;

    for index in required_indexes() {
        let options = mongodb::options::IndexOptions::builder()
            .background(true)
            .unique(index.unique)
            .build();

        let model = IndexModel::builder()
            .keys(index.keys.clone())
            .options(options)
            .build();

        if let Err(e) = db
            .collection::<Document>(index.collection)
            .create_index(model)
            .await
        {
            failures += 1;
            if index.unique {
                // Much the most likely cause, and the operator can act on it.
                tracing::error!(
                    collection = index.collection,
                    keys = %index.keys,
                    reason = index.reason,
                    error = %e,
                    "could not create a unique index. If this is a duplicate-key error, \
                     the collection already holds rows that violate the constraint - find \
                     and remove them, then restart. Until then this uniqueness is NOT \
                     enforced."
                );
            } else {
                tracing::warn!(
                    collection = index.collection,
                    keys = %index.keys,
                    reason = index.reason,
                    error = %e,
                    "could not create an index; queries against this collection will scan it"
                );
            }
        }
    }

    if failures == 0 {
        tracing::info!(
            count = required_indexes().len(),
            "MongoDB indexes are in place"
        );
    } else {
        tracing::error!(
            failures,
            total = required_indexes().len(),
            "some MongoDB indexes could not be created; see the errors above"
        );
    }

    failures
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_authorization_hot_path_is_indexed() {
        // Every authenticated request resolves roles and direct grants. An
        // unindexed scan here is paid per request, which is the difference
        // between a catalog that scales and one that does not.
        let indexes = required_indexes();
        for (collection, field) in [
            ("user_roles", "user-id"),
            ("permissions", "user-id"),
            ("revoked_tokens", "token_id"),
        ] {
            assert!(
                indexes
                    .iter()
                    .any(|i| i.collection == collection && i.keys.contains_key(field)),
                "{collection}({field}) is on the per-request path and must be indexed"
            );
        }
    }

    #[test]
    fn kebab_case_fields_are_spelled_as_stored() {
        // The RBAC collections serialize with `rename_all = "kebab-case"`, so an
        // index on `user_id` would index a field that does not exist and silently
        // do nothing. This is the same spelling trap that made every role
        // assignment unreadable.
        let indexes = required_indexes();
        for index in &indexes {
            if matches!(
                index.collection,
                "user_roles" | "permissions" | "service_users" | "business_metadata"
            ) {
                for key in index.keys.keys() {
                    assert!(
                        !key.contains('_') || key == "tenant_id",
                        "{}: {key} looks snake_case, but this collection stores \
                         kebab-case field names",
                        index.collection
                    );
                }
            }
        }
    }

    #[test]
    fn uniqueness_matches_what_the_sql_backends_enforce() {
        let indexes = required_indexes();
        for collection in ["catalogs", "warehouses", "branches", "tags"] {
            assert!(
                indexes
                    .iter()
                    .any(|i| i.collection == collection && i.unique),
                "{collection} has a uniqueness constraint in the SQL backends and \
                 must have one here, or MongoDB accepts duplicates the others reject"
            );
        }
    }

    #[test]
    fn every_index_explains_itself() {
        for index in required_indexes() {
            assert!(
                !index.reason.is_empty(),
                "{} has no stated reason; an index nobody can justify is one \
                 nobody can safely remove",
                index.collection
            );
        }
    }
}
