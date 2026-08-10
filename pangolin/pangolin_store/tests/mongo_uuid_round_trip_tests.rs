//! Every MongoDB entity must survive a write/read round trip.
//!
//! There are three ways this codebase turns a `Uuid` into BSON, and they do not
//! agree:
//!
//! | route | produces |
//! |---|---|
//! | `to_bson_uuid` | `Binary`, generic subtype |
//! | `doc! { "k": uuid }` | `Binary`, *UUID* subtype |
//! | `bson::to_document(&value)` | a `String` |
//!
//! And two ways of reading one back, which also disagree: a typed
//! `Collection<T>` deserializes non-human-readably and demands binary, while
//! `bson::from_bson` demands a string. So a write and a read chosen
//! independently - which is how every one of these modules was written - agree
//! only by luck. The failure is never a compile error and usually not a runtime
//! error either: the filter simply matches nothing.
//!
//! What that cost, before this file existed:
//!
//! * **B1** audit events were readable across tenants, because the tenant
//!   filter was dropped rather than fixed;
//! * **B2** token revocation was a silent no-op - revoked JWTs stayed valid,
//!   and logout did nothing;
//! * role assignments were unreadable, so every role-derived permission
//!   silently vanished and **a user holding an admin role was authorized as
//!   though they held none**;
//! * permission records could not be deserialized at all;
//! * business metadata could be written but never read back;
//! * **every** service-user method was a no-op, including the API-key lookup,
//!   so API-key authentication could not succeed on Mongo at all;
//! * listing active tokens failed outright;
//! * a branch with a head commit - that is, any branch that has been committed
//!   to - could not be read.
//!
//! Each was found separately, by a different failing test, months apart. Fixing
//! them one at a time treats eight symptoms of one cause. This file is the
//! cause-level test: it round-trips every entity that carries a `Uuid`, so a
//! new collection whose write and read disagree fails here immediately rather
//! than in whatever feature happens to touch it next.
//!
//! The rule the fixes settled on: **write through `to_bson_uuid`, read through
//! `from_bson_uuid`**, which accepts all three encodings so data already in a
//! deployed database still loads.
//!
//! Requires a MongoDB; skips with a note when `PANGOLIN_TEST_MONGO_URL` is
//! unset. CI sets it for both topologies.

use chrono::Utc;
use pangolin_core::audit::{AuditAction, AuditLogEntry, ResourceType};
use pangolin_core::business_metadata::{AccessRequest, BusinessMetadata, RequestStatus};
use pangolin_core::model::*;
use pangolin_core::permission::{Action, Permission, PermissionScope, Role, UserRole};
use pangolin_core::token::TokenInfo;
use pangolin_core::user::{ServiceUser, User, UserRole as UserRoleEnum};
use pangolin_store::{CatalogStore, MongoStore};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Open a store against a database unique to this test, or skip.
macro_rules! store_or_skip {
    () => {{
        let Some(url) = pangolin_store::test_support::mongo_url() else {
            println!("skipping: set PANGOLIN_TEST_MONGO_URL to run this test");
            return;
        };
        let db = format!("pangolin_roundtrip_{}", Uuid::new_v4().simple());
        match MongoStore::new(&url, &db).await {
            Ok(store) => store,
            Err(e) => panic!("PANGOLIN_TEST_MONGO_URL is set but unusable: {e}"),
        }
    }};
}

async fn seeded_tenant(store: &MongoStore) -> Uuid {
    let tenant_id = Uuid::new_v4();
    store
        .create_tenant(Tenant {
            id: tenant_id,
            name: format!("roundtrip-{tenant_id}"),
            properties: HashMap::new(),
        })
        .await
        .expect("create tenant");
    tenant_id
}

#[tokio::test]
async fn tenant_round_trips() {
    let store = store_or_skip!();
    let tenant_id = seeded_tenant(&store).await;

    let read = store
        .get_tenant(tenant_id)
        .await
        .expect("get tenant")
        .expect("tenant should exist");
    assert_eq!(read.id, tenant_id, "tenant id did not round-trip");
}

#[tokio::test]
async fn warehouse_round_trips() {
    let store = store_or_skip!();
    let tenant_id = seeded_tenant(&store).await;

    let warehouse = Warehouse {
        id: Uuid::new_v4(),
        name: "wh".to_string(),
        tenant_id,
        storage_config: HashMap::new(),
        use_sts: false,
        vending_strategy: None,
    };
    store
        .create_warehouse(tenant_id, warehouse.clone())
        .await
        .expect("create warehouse");

    let read = store
        .get_warehouse(tenant_id, "wh".to_string())
        .await
        .expect("get warehouse")
        .expect("warehouse should exist");
    assert_eq!(read.id, warehouse.id, "warehouse id did not round-trip");
    assert_eq!(
        read.tenant_id, tenant_id,
        "warehouse tenant_id did not round-trip"
    );
}

#[tokio::test]
async fn catalog_and_asset_round_trip() {
    let store = store_or_skip!();
    let tenant_id = seeded_tenant(&store).await;

    let catalog = Catalog {
        id: Uuid::new_v4(),
        name: "cat".to_string(),
        catalog_type: CatalogType::Local,
        warehouse_name: None,
        storage_location: None,
        federated_config: None,
        properties: HashMap::new(),
    };
    store
        .create_catalog(tenant_id, catalog.clone())
        .await
        .expect("create catalog");

    let read = store
        .get_catalog(tenant_id, "cat".to_string())
        .await
        .expect("get catalog")
        .expect("catalog should exist");
    assert_eq!(read.id, catalog.id, "catalog id did not round-trip");

    let namespace = vec!["ns".to_string()];
    store
        .create_namespace(
            tenant_id,
            "cat",
            Namespace {
                name: namespace.clone(),
                properties: HashMap::new(),
            },
        )
        .await
        .expect("create namespace");

    let asset = Asset {
        id: Uuid::new_v4(),
        name: "tbl".to_string(),
        kind: AssetType::DeltaTable,
        location: "s3://bucket/tbl".to_string(),
        properties: HashMap::new(),
    };
    store
        .create_asset(tenant_id, "cat", None, namespace.clone(), asset.clone())
        .await
        .expect("create asset");

    let read = store
        .get_asset(tenant_id, "cat", None, namespace, "tbl".to_string())
        .await
        .expect("get asset")
        .expect("asset should exist");
    assert_eq!(read.id, asset.id, "asset id did not round-trip");
    assert_eq!(
        read.kind,
        AssetType::DeltaTable,
        "asset kind did not round-trip"
    );

    // The by-id index is a separate lookup path with its own encoding.
    let by_id = store
        .get_asset_by_id(tenant_id, asset.id)
        .await
        .expect("get asset by id");
    assert!(
        by_id.is_some(),
        "asset was not findable by id - the id was stored in a form the lookup \
         cannot match"
    );
}

#[tokio::test]
async fn user_round_trips() {
    let store = store_or_skip!();
    let tenant_id = seeded_tenant(&store).await;

    let user = User {
        id: Uuid::new_v4(),
        username: format!("rt_{}", Uuid::new_v4().simple()),
        email: format!("rt_{}@example.test", Uuid::new_v4().simple()),
        password_hash: None,
        oauth_provider: None,
        oauth_subject: None,
        tenant_id: Some(tenant_id),
        role: UserRoleEnum::TenantUser,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_login: None,
        active: true,
    };
    store.create_user(user.clone()).await.expect("create user");

    let read = store
        .get_user(user.id)
        .await
        .expect("get user")
        .expect("user should exist");
    assert_eq!(read.id, user.id, "user id did not round-trip");
    assert_eq!(
        read.tenant_id,
        Some(tenant_id),
        "user tenant_id did not round-trip"
    );
}

/// Roles and role assignments.
///
/// The assignment is the one that silently mis-authorized users: written by
/// serde as a string, queried as Binary, so `get_user_roles` returned nothing
/// and an admin's grants evaporated.
#[tokio::test]
async fn role_and_assignment_round_trip() {
    let store = store_or_skip!();
    let tenant_id = seeded_tenant(&store).await;
    let user_id = Uuid::new_v4();

    let mut role = Role::new("rt-role".to_string(), None, tenant_id, Uuid::new_v4());
    let mut actions = HashSet::new();
    actions.insert(Action::Read);
    role.add_permission(PermissionScope::Tenant, actions);
    store.create_role(role.clone()).await.expect("create role");

    let read = store
        .get_role(role.id)
        .await
        .expect("get role")
        .expect("role should be findable by id");
    assert_eq!(read.id, role.id, "role id did not round-trip");
    assert_eq!(
        read.tenant_id, tenant_id,
        "role tenant_id did not round-trip"
    );
    assert_eq!(
        read.permissions.len(),
        1,
        "the role's grants did not round-trip"
    );

    let listed = store.list_roles(tenant_id, None).await.expect("list roles");
    assert!(
        listed.iter().any(|r| r.id == role.id),
        "the role was not returned by a tenant listing"
    );

    let assignment = UserRole::new(user_id, role.id, Uuid::new_v4());
    store
        .assign_role(assignment.clone())
        .await
        .expect("assign role");

    let assignments = store.get_user_roles(user_id).await.expect("get user roles");
    assert_eq!(
        assignments.len(),
        1,
        "the role assignment was not readable back - this is the defect that \
         silently authorized an admin as though they held no roles"
    );
    assert_eq!(assignments[0].role_id, role.id);
    assert_eq!(assignments[0].user_id, user_id);

    // The grant must actually reach the permission list, which is what
    // authorization consults.
    let perms = store
        .list_user_permissions(user_id, None)
        .await
        .expect("list user permissions");
    assert!(
        perms.iter().any(|p| p.scope == PermissionScope::Tenant),
        "a role-derived permission did not reach list_user_permissions"
    );
}

#[tokio::test]
async fn direct_permission_round_trips() {
    let store = store_or_skip!();
    let tenant_id = seeded_tenant(&store).await;
    let user_id = Uuid::new_v4();

    let mut actions = HashSet::new();
    actions.insert(Action::Write);
    let permission = Permission::new(
        user_id,
        tenant_id,
        PermissionScope::Catalog {
            catalog_id: Uuid::new_v4(),
        },
        actions,
        Uuid::new_v4(),
    );
    store
        .create_permission(permission.clone())
        .await
        .expect("create permission");

    let perms = store
        .list_user_permissions(user_id, None)
        .await
        .expect("list user permissions");
    assert!(
        perms.iter().any(|p| p.id == permission.id),
        "a direct permission was not readable back"
    );
    let found = perms.iter().find(|p| p.id == permission.id).unwrap();
    assert_eq!(found.user_id, user_id, "user_id did not round-trip");
    assert_eq!(found.tenant_id, tenant_id, "tenant_id did not round-trip");
}

#[tokio::test]
async fn service_user_round_trips() {
    let store = store_or_skip!();
    let tenant_id = seeded_tenant(&store).await;

    let api_key_hash = format!("hash-{}", Uuid::new_v4().simple());
    let service_user = ServiceUser::new(
        "rt-svc".to_string(),
        None,
        tenant_id,
        api_key_hash.clone(),
        UserRoleEnum::TenantUser,
        Uuid::new_v4(),
        None,
    );
    store
        .create_service_user(service_user.clone())
        .await
        .expect("create service user");

    let read = store
        .get_service_user(service_user.id)
        .await
        .expect("get service user")
        .expect("service user should be findable by id");
    assert_eq!(
        read.id, service_user.id,
        "service user id did not round-trip"
    );
    assert_eq!(
        read.tenant_id, tenant_id,
        "service user tenant_id did not round-trip"
    );

    let listed = store
        .list_service_users(tenant_id, None)
        .await
        .expect("list service users");
    assert!(
        listed.iter().any(|s| s.id == service_user.id),
        "the service user was not returned by a tenant listing"
    );

    // The authentication path. If this lookup misses, an API key is simply
    // rejected - service-user auth is down, not bypassed.
    let by_hash = store
        .get_service_user_by_api_key_hash(&api_key_hash)
        .await
        .expect("look up service user by api key hash")
        .expect("the api key hash should resolve to its service user");
    assert_eq!(
        by_hash.id, service_user.id,
        "the api key hash resolved to the wrong principal"
    );

    // Updating the role rewrites a field the reader has to parse back into an
    // enum; writing the Rust variant name instead of its serde form made the
    // record unreadable from that point on.
    let updated = store
        .update_service_user(
            service_user.id,
            None,
            Some("promoted".to_string()),
            Some(UserRoleEnum::TenantAdmin),
            None,
        )
        .await
        .expect("update service user");
    assert_eq!(
        updated.role,
        UserRoleEnum::TenantAdmin,
        "the updated role did not round-trip"
    );

    store
        .update_service_user_last_used(service_user.id)
        .await
        .expect("touch last_used");
    let touched = store
        .get_service_user(service_user.id)
        .await
        .expect("re-read after touch")
        .expect("service user should still exist");
    assert!(
        touched.last_used.is_some(),
        "last_used was written to a field nothing reads back"
    );

    store
        .delete_service_user(service_user.id)
        .await
        .expect("delete service user");
    assert!(
        store
            .get_service_user(service_user.id)
            .await
            .expect("get after delete")
            .is_none(),
        "the service user survived its own deletion - the delete filter matched nothing"
    );
}

/// Token issuance and revocation.
///
/// B2: revocation was written by serde and queried as Binary, so the check
/// never matched and a revoked token - including after logout - stayed valid.
#[tokio::test]
async fn token_and_revocation_round_trip() {
    let store = store_or_skip!();
    let tenant_id = seeded_tenant(&store).await;
    let user_id = Uuid::new_v4();
    let token_id = Uuid::new_v4();

    store
        .store_token(TokenInfo {
            id: token_id,
            tenant_id,
            user_id,
            username: "rt".to_string(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            created_at: Utc::now(),
            is_valid: true,
            token: Some("opaque".to_string()),
        })
        .await
        .expect("store token");

    let listed = store
        .list_active_tokens(tenant_id, Some(user_id), None)
        .await
        .expect("list active tokens");
    assert!(
        listed.iter().any(|t| t.id == token_id),
        "the token was not readable back"
    );

    assert!(
        !store
            .is_token_revoked(token_id)
            .await
            .expect("check revocation"),
        "a fresh token must not read as revoked"
    );

    store
        .revoke_token(token_id, Utc::now() + chrono::Duration::hours(1), None)
        .await
        .expect("revoke token");

    assert!(
        store
            .is_token_revoked(token_id)
            .await
            .expect("check revocation"),
        "a revoked token must read back as revoked (B2)"
    );
}

/// B1: the audit event's tenant scoping.
#[tokio::test]
async fn audit_event_round_trips() {
    let store = store_or_skip!();
    let tenant_id = seeded_tenant(&store).await;
    let user_id = Uuid::new_v4();
    let resource_id = Uuid::new_v4();

    let entry = AuditLogEntry::success(
        tenant_id,
        Some(user_id),
        "rt".to_string(),
        AuditAction::CreateBranch,
        ResourceType::Branch,
        Some(resource_id),
        "cat/feature".to_string(),
    );
    let event_id = entry.id;
    CatalogStore::log_audit_event(&store, tenant_id, entry)
        .await
        .expect("log audit event");

    let read = store
        .get_audit_event(tenant_id, event_id)
        .await
        .expect("get audit event")
        .expect("audit event should exist");
    assert_eq!(read.id, event_id, "audit event id did not round-trip");
    assert_eq!(read.user_id, Some(user_id), "user_id did not round-trip");
    assert_eq!(
        read.resource_id,
        Some(resource_id),
        "resource_id did not round-trip"
    );
    assert_eq!(
        read.action,
        AuditAction::CreateBranch,
        "the action must round-trip, not collapse to a default"
    );

    let listed = store
        .list_audit_events(tenant_id, None)
        .await
        .expect("list audit events");
    assert!(
        listed.iter().any(|e| e.id == event_id),
        "the audit event was not returned by a listing"
    );
}

#[tokio::test]
async fn business_metadata_round_trips() {
    let store = store_or_skip!();
    let asset_id = Uuid::new_v4();
    let author = Uuid::new_v4();

    let mut metadata = BusinessMetadata::new(asset_id, author);
    metadata.tags = vec!["pii".to_string()];
    metadata.description = Some("round trip".to_string());
    let metadata_id = metadata.id;

    store
        .upsert_business_metadata(metadata)
        .await
        .expect("upsert business metadata");

    let read = store
        .get_business_metadata(asset_id)
        .await
        .expect("get business metadata")
        .expect("metadata should exist");
    assert_eq!(read.id, metadata_id, "metadata id did not round-trip");
    assert_eq!(read.asset_id, asset_id, "asset_id did not round-trip");
    assert_eq!(read.created_by, author, "created_by did not round-trip");
    assert_eq!(read.tags, vec!["pii".to_string()]);
}

#[tokio::test]
async fn merge_operation_and_conflict_round_trip() {
    let store = store_or_skip!();
    let tenant_id = seeded_tenant(&store).await;

    let operation = MergeOperation::new(
        tenant_id,
        "cat".to_string(),
        "feature".to_string(),
        "main".to_string(),
        None,
        Uuid::new_v4(),
    );
    let operation_id = operation.id;
    store
        .create_merge_operation(operation)
        .await
        .expect("create merge operation");

    let read = store
        .get_merge_operation(operation_id)
        .await
        .expect("get merge operation")
        .expect("merge operation should be findable by id");
    assert_eq!(read.id, operation_id, "operation id did not round-trip");
    assert_eq!(read.tenant_id, tenant_id, "tenant_id did not round-trip");

    let listed = store
        .list_merge_operations(tenant_id, "cat", None)
        .await
        .expect("list merge operations");
    assert!(
        listed.iter().any(|o| o.id == operation_id),
        "the merge operation was not returned by a listing"
    );

    let conflict = MergeConflict::new(
        operation_id,
        ConflictType::MetadataConflict {
            asset_name: "tbl".to_string(),
            conflicting_properties: vec!["owner".to_string()],
        },
        Some(Uuid::new_v4()),
        "owner differs".to_string(),
    );
    let conflict_id = conflict.id;
    store
        .create_merge_conflict(conflict)
        .await
        .expect("create merge conflict");

    let conflicts = store
        .list_merge_conflicts(operation_id, None)
        .await
        .expect("list merge conflicts");
    assert!(
        conflicts.iter().any(|c| c.id == conflict_id),
        "the conflict was not readable back by its operation id"
    );
}

#[tokio::test]
async fn commit_round_trips() {
    let store = store_or_skip!();
    let tenant_id = seeded_tenant(&store).await;

    let commit = Commit {
        id: Uuid::new_v4(),
        parent_id: None,
        timestamp: Utc::now().timestamp_millis(),
        author: "rt".to_string(),
        message: "round trip".to_string(),
        operations: vec![],
    };
    let commit_id = commit.id;
    store
        .create_commit(tenant_id, commit)
        .await
        .expect("create commit");

    let read = store
        .get_commit(tenant_id, commit_id)
        .await
        .expect("get commit")
        .expect("commit should be findable by id");
    assert_eq!(read.id, commit_id, "commit id did not round-trip");
}

/// Branches and tags.
///
/// Both are written with a hand-built `doc!` rather than serde, which converts
/// a `Uuid` by a *third* route again (`impl From<Uuid> for Bson`). Whether that
/// agrees with what the read path expects is not something the type system
/// checks, so it is asserted here.
#[tokio::test]
async fn branch_and_tag_round_trip() {
    let store = store_or_skip!();
    let tenant_id = seeded_tenant(&store).await;
    store
        .create_catalog(
            tenant_id,
            Catalog {
                id: Uuid::new_v4(),
                name: "cat".to_string(),
                catalog_type: CatalogType::Local,
                warehouse_name: None,
                storage_location: None,
                federated_config: None,
                properties: HashMap::new(),
            },
        )
        .await
        .expect("create catalog");

    let head = Uuid::new_v4();
    store
        .create_branch(
            tenant_id,
            "cat",
            Branch {
                name: "feature".to_string(),
                head_commit_id: Some(head),
                branch_type: BranchType::Experimental,
                assets: vec!["tbl".to_string()],
            },
        )
        .await
        .expect("create branch");

    let read = store
        .get_branch(tenant_id, "cat", "feature".to_string())
        .await
        .expect("get branch")
        .expect("branch should exist");
    assert_eq!(
        read.head_commit_id,
        Some(head),
        "the branch head commit id did not round-trip"
    );
    assert_eq!(read.assets, vec!["tbl".to_string()]);

    // A branch with no head is the common case right after creation, and it
    // takes a different path through the same conversion.
    store
        .create_branch(
            tenant_id,
            "cat",
            Branch {
                name: "empty".to_string(),
                head_commit_id: None,
                branch_type: BranchType::Ingest,
                assets: vec![],
            },
        )
        .await
        .expect("create headless branch");
    let headless = store
        .get_branch(tenant_id, "cat", "empty".to_string())
        .await
        .expect("get headless branch")
        .expect("headless branch should exist");
    assert_eq!(headless.head_commit_id, None);
    assert!(
        matches!(headless.branch_type, BranchType::Ingest),
        "the branch type did not round-trip"
    );

    let commit_id = Uuid::new_v4();
    store
        .create_tag(
            tenant_id,
            "cat",
            Tag {
                name: "v1".to_string(),
                commit_id,
            },
        )
        .await
        .expect("create tag");
    let tag = store
        .get_tag(tenant_id, "cat", "v1".to_string())
        .await
        .expect("get tag")
        .expect("tag should exist");
    assert_eq!(
        tag.commit_id, commit_id,
        "the tag commit id did not round-trip"
    );
    let tags = store
        .list_tags(tenant_id, "cat", None)
        .await
        .expect("list tags");
    assert!(
        tags.iter().any(|t| t.name == "v1"),
        "the tag was not returned by a listing"
    );
}

/// Access requests.
///
/// The listing joins `access_requests.user-id` to `users.id`, so it only
/// returns anything if the two collections encode a UUID the same way - a
/// cross-collection agreement no single-module test can check.
#[tokio::test]
async fn access_request_round_trips() {
    let store = store_or_skip!();
    let tenant_id = seeded_tenant(&store).await;

    let user = User {
        id: Uuid::new_v4(),
        username: format!("rt_{}", Uuid::new_v4().simple()),
        email: format!("rt_{}@example.test", Uuid::new_v4().simple()),
        password_hash: None,
        oauth_provider: None,
        oauth_subject: None,
        tenant_id: Some(tenant_id),
        role: UserRoleEnum::TenantUser,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_login: None,
        active: true,
    };
    store.create_user(user.clone()).await.expect("create user");

    let request = AccessRequest::new(
        tenant_id,
        user.id,
        Uuid::new_v4(),
        Some("need read".to_string()),
    );
    let request_id = request.id;
    store
        .create_access_request(request)
        .await
        .expect("create access request");

    let read = store
        .get_access_request(request_id)
        .await
        .expect("get access request")
        .expect("access request should be findable by id");
    assert_eq!(read.user_id, user.id, "user_id did not round-trip");
    assert_eq!(read.asset_id, read.asset_id);

    let listed = store
        .list_access_requests(tenant_id, None)
        .await
        .expect("list access requests");
    assert!(
        listed.iter().any(|r| r.id == request_id),
        "the access request was not returned by its tenant listing - the join \
         between access_requests.user-id and users.id did not match"
    );

    let mut approved = read;
    approved.approve(Uuid::new_v4(), Some("ok".to_string()));
    store
        .update_access_request(approved.clone())
        .await
        .expect("update access request");
    let after = store
        .get_access_request(request_id)
        .await
        .expect("re-read access request")
        .expect("access request should still exist");
    assert_eq!(
        after.status,
        RequestStatus::Approved,
        "the approval did not round-trip"
    );
    assert!(
        after.reviewed_by.is_some(),
        "reviewed_by did not round-trip"
    );
}

/// The merge mutation paths.
///
/// `create_merge_operation` is covered above, but the status transitions build
/// their update documents by hand - including one that writes a status literal
/// as a Rust `Debug` string. Those writes are never read back by any other
/// test, so a value that no longer deserializes would go unnoticed until a
/// merge was actually completed in production.
#[tokio::test]
async fn merge_status_transitions_round_trip() {
    let store = store_or_skip!();
    let tenant_id = seeded_tenant(&store).await;

    let operation = MergeOperation::new(
        tenant_id,
        "cat".to_string(),
        "feature".to_string(),
        "main".to_string(),
        Some(Uuid::new_v4()),
        Uuid::new_v4(),
    );
    let operation_id = operation.id;
    store
        .create_merge_operation(operation)
        .await
        .expect("create merge operation");

    store
        .update_merge_operation_status(operation_id, MergeStatus::Conflicted)
        .await
        .expect("set status");
    let read = store
        .get_merge_operation(operation_id)
        .await
        .expect("re-read after status change")
        .expect("operation should still exist");
    assert!(
        matches!(read.status, MergeStatus::Conflicted),
        "the status write produced a value that does not deserialize back"
    );

    let conflict = MergeConflict::new(
        operation_id,
        ConflictType::MetadataConflict {
            asset_name: "tbl".to_string(),
            conflicting_properties: vec!["owner".to_string()],
        },
        Some(Uuid::new_v4()),
        "owner differs".to_string(),
    );
    let conflict_id = conflict.id;
    store
        .create_merge_conflict(conflict)
        .await
        .expect("create conflict");
    store
        .add_conflict_to_operation(operation_id, conflict_id)
        .await
        .expect("attach conflict");

    let with_conflict = store
        .get_merge_operation(operation_id)
        .await
        .expect("re-read after attaching a conflict")
        .expect("operation should still exist");
    assert!(
        with_conflict.conflicts.contains(&conflict_id),
        "the attached conflict id was written in a form the operation cannot \
         read back"
    );

    store
        .resolve_merge_conflict(
            conflict_id,
            ConflictResolution {
                conflict_id,
                strategy: ResolutionStrategy::TakeSource,
                resolved_value: None,
                resolved_by: Uuid::new_v4(),
                resolved_at: Utc::now(),
            },
        )
        .await
        .expect("resolve conflict");
    let resolved = store
        .get_merge_conflict(conflict_id)
        .await
        .expect("re-read conflict")
        .expect("conflict should still exist");
    assert!(
        resolved.resolution.is_some(),
        "the resolution did not round-trip"
    );

    let result_commit = Uuid::new_v4();
    store
        .complete_merge_operation(operation_id, result_commit)
        .await
        .expect("complete merge");
    let completed = store
        .get_merge_operation(operation_id)
        .await
        .expect("re-read after completion")
        .expect("operation should still exist");
    assert!(
        matches!(completed.status, MergeStatus::Completed),
        "the completed status did not round-trip"
    );
    assert_eq!(
        completed.result_commit_id,
        Some(result_commit),
        "the result commit id did not round-trip"
    );
    assert!(
        completed.completed_at.is_some(),
        "completed_at did not round-trip"
    );
}
