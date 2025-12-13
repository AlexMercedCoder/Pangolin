use axum::Router;
use std::sync::Arc;
use pangolin_store::CatalogStore;
use pangolin_store::memory::MemoryStore;
use axum::routing::{get, post, delete, put};

pub mod iceberg_handlers;
pub mod pangolin_handlers;
pub mod tenant_handlers;
pub mod warehouse_handlers;
pub mod asset_handlers;
pub mod auth;
pub mod signing_handlers;
pub mod token_handlers;
pub mod user_handlers;
pub mod oauth_handlers;
pub mod auth_middleware;
pub mod authz;
pub mod business_metadata_handlers;

#[cfg(test)]
pub mod auth_test;

#[cfg(test)]
pub mod business_metadata_test;

pub mod permission_handlers; // Registered new module

#[cfg(test)]
#[path = "iceberg_handlers_test.rs"]
mod iceberg_handlers_test;

#[cfg(test)]
#[path = "signing_handlers_test.rs"]
mod signing_handlers_test;

pub fn app(store: Arc<dyn CatalogStore + Send + Sync>) -> Router {
    Router::new()
        .route("/v1/config", get(iceberg_handlers::config))
        .route("/v1/:prefix/config", get(iceberg_handlers::config))
        .route("/v1/:prefix/namespaces", get(iceberg_handlers::list_namespaces).post(iceberg_handlers::create_namespace))
        .route("/v1/:prefix/namespaces/:namespace", delete(iceberg_handlers::delete_namespace))
        .route("/v1/:prefix/namespaces/:namespace/properties", post(iceberg_handlers::update_namespace_properties))
        .route("/v1/:prefix/namespaces/:namespace/tables", get(iceberg_handlers::list_tables).post(iceberg_handlers::create_table))
        .route("/v1/:prefix/namespaces/:namespace/tables/:table", get(iceberg_handlers::load_table).post(iceberg_handlers::update_table).delete(iceberg_handlers::delete_table).head(iceberg_handlers::table_exists))
        .route("/v1/:prefix/namespaces/:namespace/tables/:table/maintenance", post(iceberg_handlers::perform_maintenance))
        .route("/v1/:prefix/namespaces/:namespace/tables/:table/metrics", post(iceberg_handlers::report_metrics))
        .route("/v1/:prefix/tables/rename", post(iceberg_handlers::rename_table))
        // Pangolin Extended APIs
        .route("/api/v1/branches", get(pangolin_handlers::list_branches).post(pangolin_handlers::create_branch))
        .route("/api/v1/branches/merge", post(pangolin_handlers::merge_branch))
        .route("/api/v1/branches/:name", get(pangolin_handlers::get_branch))
        .route("/api/v1/branches/:name/commits", get(pangolin_handlers::list_commits))
        // Tag Management
        .route("/api/v1/tags", get(pangolin_handlers::list_tags).post(pangolin_handlers::create_tag))
        .route("/api/v1/tags/:name", delete(pangolin_handlers::delete_tag))
        // Audit Logs
        .route("/api/v1/audit", get(pangolin_handlers::list_audit_events))
        // Tenant Management
        .route("/api/v1/tenants", get(tenant_handlers::list_tenants).post(tenant_handlers::create_tenant))
        .route("/api/v1/tenants/:id", get(tenant_handlers::get_tenant))
        // Warehouse Management
        .route("/api/v1/warehouses", get(warehouse_handlers::list_warehouses).post(warehouse_handlers::create_warehouse))
        .route("/api/v1/warehouses/:name", get(warehouse_handlers::get_warehouse))
        // Catalog Management
        .route("/api/v1/catalogs", get(pangolin_handlers::list_catalogs).post(pangolin_handlers::create_catalog))
        .route("/api/v1/catalogs/:name", get(pangolin_handlers::get_catalog))
        // Asset Management (Views)
        .route("/v1/:prefix/namespaces/:namespace/views", post(asset_handlers::create_view))
        .route("/v1/:prefix/namespaces/:namespace/views/:view", get(asset_handlers::get_view))
        // Signing APIs
        .route("/v1/:prefix/namespaces/:namespace/tables/:table/credentials", get(signing_handlers::get_table_credentials))
        .route("/v1/:prefix/namespaces/:namespace/tables/:table/presign", get(signing_handlers::get_presigned_url))
        // Business Metadata
        .route("/api/v1/assets/:id/metadata", post(business_metadata_handlers::add_business_metadata).get(business_metadata_handlers::get_business_metadata).delete(business_metadata_handlers::delete_business_metadata))
        .route("/api/v1/assets/search", get(business_metadata_handlers::search_assets))
        .route("/api/v1/assets/:id", get(business_metadata_handlers::get_asset_details))
        .route("/api/v1/assets/:id/request-access", post(business_metadata_handlers::request_access))
        .route("/api/v1/access-requests", get(business_metadata_handlers::list_access_requests))
        .route("/api/v1/access-requests/:id", put(business_metadata_handlers::update_access_request))
        // User Management
        .route("/api/v1/users", post(user_handlers::create_user).get(user_handlers::list_users))
        .route("/api/v1/users/:id", get(user_handlers::get_user).put(user_handlers::update_user).delete(user_handlers::delete_user))
        .route("/api/v1/users/login", post(user_handlers::login))
        .route("/api/v1/users/me", get(user_handlers::get_current_user))
        .route("/api/v1/users/logout", post(user_handlers::logout))
        // Role Management
        .route("/api/v1/roles", post(permission_handlers::create_role).get(permission_handlers::list_roles))
        .route("/api/v1/roles/:id", get(permission_handlers::get_role).put(permission_handlers::update_role).delete(permission_handlers::delete_role))
        // Permission Management
        .route("/api/v1/permissions", post(permission_handlers::grant_permission))
        .route("/api/v1/permissions/:id", delete(permission_handlers::revoke_permission))
        .route("/api/v1/permissions/user/:id", get(permission_handlers::get_user_permissions))
        // Role Assignment
        .route("/api/v1/users/:id/roles", post(permission_handlers::assign_role))
        .route("/api/v1/users/:id/roles/:role_id", delete(permission_handlers::revoke_role))
        // Token Generation
        .route("/api/v1/tokens", post(token_handlers::generate_token))
        // OAuth
        .route("/oauth/authorize/:provider", get(oauth_handlers::oauth_authorize))
        .route("/oauth/callback/:provider", get(oauth_handlers::oauth_callback))
        .layer(axum::middleware::from_fn(auth_middleware::auth_middleware))
        .with_state(store)
}
