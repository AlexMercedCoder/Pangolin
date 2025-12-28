use axum::Router;
use std::sync::Arc;
use pangolin_store::CatalogStore;
use pangolin_store::memory::MemoryStore;
use axum::routing::{get, post, delete, put};
use tower_http::cors::{CorsLayer, Any};
use axum::http::{HeaderValue, Method};
use utoipa::OpenApi;

pub mod iceberg;
pub mod cached_store;
pub mod pangolin_handlers;
pub mod tenant_handlers;
pub mod warehouse_handlers;
pub mod asset_handlers;
pub mod auth;
pub mod signing_handlers;
pub mod credential_signers;
pub mod credential_vending;


pub mod token_handlers;
pub mod user_handlers;
pub mod oauth_handlers;
pub mod auth_middleware;
pub mod authz;
pub mod authz_utils; // Permission filtering utilities
pub mod business_metadata_handlers;
pub mod conflict_detector;
pub mod merge_handlers;
pub mod federated_proxy;
pub mod federated_catalog_handlers;
pub mod tests_common;
pub mod audit_handlers;
#[cfg(test)]
pub mod verification_tests;
#[cfg(test)]
pub mod audit_tests;





pub mod permission_handlers; // Registered new module
pub mod system_config_handlers; // System configuration
pub mod service_user_handlers; // Service user management
pub mod cleanup_job; // Token cleanup background job
pub mod openapi; // OpenAPI documentation
pub mod error; // Custom error types
pub mod dashboard_handlers; // Dashboard statistics
pub mod optimization_handlers; // Search, bulk ops, validation









pub fn app(store: Arc<dyn CatalogStore + Send + Sync>) -> Router {
    let cors = CorsLayer::new()
        //.allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH, Method::OPTIONS])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
            "x-pangolin-tenant".parse::<axum::http::HeaderName>().unwrap(),
        ]);
        //.allow_credentials(true);

    Router::new()
        .route("/health", get(|| async { "OK" }))
        // Swagger UI for API documentation
        .merge(utoipa_swagger_ui::SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", openapi::ApiDoc::openapi()))
        // System Config
        .route("/api/v1/config/settings", get(system_config_handlers::get_system_settings).put(system_config_handlers::update_system_settings))
        .route("/v1/config", get(iceberg::config::get_iceberg_catalog_config_handler))
        .route("/v1/:prefix/config", get(iceberg::config::get_iceberg_catalog_config_handler))
        .route("/v1/:prefix/namespaces", get(iceberg::namespaces::list_namespaces).post(iceberg::namespaces::create_namespace))
        .route("/v1/:prefix/namespaces/:namespace", delete(iceberg::namespaces::delete_namespace))
        .route("/v1/:prefix/namespaces/:namespace/properties", post(iceberg::namespaces::update_namespace_properties))
        .route("/v1/:prefix/namespaces/:namespace/tables", get(iceberg::tables::list_tables).post(iceberg::tables::create_table))
        .route("/v1/:prefix/namespaces/:namespace/tables/:table", get(iceberg::tables::load_table).post(iceberg::tables::update_table).delete(iceberg::tables::delete_table).head(iceberg::tables::table_exists))
        .route("/v1/:prefix/namespaces/:namespace/tables/:table/maintenance", post(iceberg::tables::perform_maintenance))
        .route("/v1/:prefix/namespaces/:namespace/tables/:table/metrics", post(iceberg::tables::report_metrics))
        .route("/v1/:prefix/tables/rename", post(iceberg::tables::rename_table))
        // PyIceberg compatibility: it might append v1/config to a path that already includes v1/prefix
        .route("/v1/:prefix/v1/config", get(iceberg::config::get_iceberg_catalog_config_handler))
        .route("/v1/:prefix/v1/namespaces", get(iceberg::namespaces::list_namespaces).post(iceberg::namespaces::create_namespace))
        .route("/v1/:prefix/v1/namespaces/:namespace", delete(iceberg::namespaces::delete_namespace))
        .route("/v1/:prefix/v1/namespaces/:namespace/properties", post(iceberg::namespaces::update_namespace_properties))
        .route("/v1/:prefix/v1/namespaces/:namespace/tables", get(iceberg::tables::list_tables).post(iceberg::tables::create_table))
        .route("/v1/:prefix/v1/namespaces/:namespace/tables/:table", get(iceberg::tables::load_table).post(iceberg::tables::update_table).delete(iceberg::tables::delete_table).head(iceberg::tables::table_exists))
        .route("/v1/:prefix/v1/namespaces/:namespace/tables/:table/maintenance", post(iceberg::tables::perform_maintenance))
        .route("/v1/:prefix/v1/namespaces/:namespace/tables/:table/metrics", post(iceberg::tables::report_metrics))
        .route("/v1/:prefix/v1/tables/rename", post(iceberg::tables::rename_table))
        .route("/v1/:prefix/v1/oauth/tokens", post(iceberg::oauth::handle_oauth_token))
        .route("/v1/:prefix/oauth/tokens", post(iceberg::oauth::handle_oauth_token)) // Support non-nested v1 too just in case
        // Support /api/v1/iceberg prefix style
        .route("/api/v1/iceberg/:prefix/v1/oauth/tokens", post(iceberg::oauth::handle_oauth_token))
        .route("/api/v1/iceberg/:prefix/oauth/tokens", post(iceberg::oauth::handle_oauth_token))
        // Pangolin Extended APIs
        // Branch Operations
        .route("/api/v1/branches", post(pangolin_handlers::create_branch).get(pangolin_handlers::list_branches))
        .route("/api/v1/branches/merge", post(pangolin_handlers::merge_branch))
        .route("/api/v1/branches/:name/rebase", post(pangolin_handlers::rebase_branch)) // Rebase endpoint
        .route("/api/v1/branches/:name", get(pangolin_handlers::get_branch))
        .route("/api/v1/branches/:name/commits", get(pangolin_handlers::list_commits))
        // Merge Operations
        .route("/api/v1/catalogs/:catalog_name/merge-operations", get(merge_handlers::list_merge_operations))
        .route("/api/v1/merge-operations/:operation_id", get(merge_handlers::get_merge_operation))
        .route("/api/v1/merge-operations/:operation_id/conflicts", get(merge_handlers::list_merge_conflicts))
        .route("/api/v1/merge-operations/:operation_id/complete", post(merge_handlers::complete_merge))
        .route("/api/v1/merge-operations/:operation_id/abort", post(merge_handlers::abort_merge))
        .route("/api/v1/conflicts/:conflict_id/resolve", post(merge_handlers::resolve_conflict))
        // Tag Operations
        .route("/api/v1/tags", post(pangolin_handlers::create_tag).get(pangolin_handlers::list_tags))
        .route("/api/v1/tags/:name", delete(pangolin_handlers::delete_tag))
        // Audit Logs (Enhanced with filtering)
        .route("/api/v1/audit", get(audit_handlers::list_audit_events))
        .route("/api/v1/audit/count", get(audit_handlers::count_audit_events))
        .route("/api/v1/audit/:event_id", get(audit_handlers::get_audit_event))
        // Tenant Management
        .route("/api/v1/tenants", get(tenant_handlers::list_tenants).post(tenant_handlers::create_tenant))
        .route("/api/v1/tenants/:id", get(tenant_handlers::get_tenant).put(tenant_handlers::update_tenant).delete(tenant_handlers::delete_tenant))
        // Warehouse Management
        .route("/api/v1/warehouses", get(warehouse_handlers::list_warehouses).post(warehouse_handlers::create_warehouse))
        .route("/api/v1/warehouses/:name", get(warehouse_handlers::get_warehouse).put(warehouse_handlers::update_warehouse).delete(warehouse_handlers::delete_warehouse))
        .route("/api/v1/warehouses/:name/credentials", get(warehouse_handlers::get_warehouse_credentials))
        // Catalog Management
        .route("/api/v1/catalogs", get(pangolin_handlers::list_catalogs).post(pangolin_handlers::create_catalog))
        .route("/api/v1/catalogs/:name", get(pangolin_handlers::get_catalog).put(pangolin_handlers::update_catalog).delete(pangolin_handlers::delete_catalog))
        .route("/api/v1/catalogs/:prefix/namespaces/tree", get(iceberg::namespaces::list_namespaces_tree))
        .route("/api/v1/catalogs/:catalog_name/namespaces/:namespace/assets", get(asset_handlers::list_assets).post(asset_handlers::register_asset))
        .route("/api/v1/catalogs/:catalog_name/namespaces/:namespace/assets/:asset", get(asset_handlers::get_asset))
        // Federated Catalog Management
        .route("/api/v1/federated-catalogs", post(federated_catalog_handlers::create_federated_catalog).get(federated_catalog_handlers::list_federated_catalogs))
        .route("/api/v1/federated-catalogs/:name", get(federated_catalog_handlers::get_federated_catalog).delete(federated_catalog_handlers::delete_federated_catalog))
        .route("/api/v1/federated-catalogs/:name/test", post(federated_catalog_handlers::test_federated_connection))
        .route("/api/v1/federated-catalogs/:name/sync", post(federated_catalog_handlers::sync_federated_catalog))
        .route("/api/v1/federated-catalogs/:name/stats", get(federated_catalog_handlers::get_federated_catalog_stats))
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
        .route("/api/v1/assets/:id/access-requests", post(business_metadata_handlers::request_access))
        .route("/api/v1/access-requests", get(business_metadata_handlers::list_access_requests))
        .route("/api/v1/access-requests/:id", put(business_metadata_handlers::update_access_request).get(business_metadata_handlers::get_access_request))
        // User Management
        .route("/api/v1/users", post(user_handlers::create_user).get(user_handlers::list_users))
        .route("/api/v1/users/:id", get(user_handlers::get_user).put(user_handlers::update_user).delete(user_handlers::delete_user))
        .route("/api/v1/users/login", post(user_handlers::login))
        .route("/api/v1/app-config", get(user_handlers::get_app_config))
        .route("/api/v1/users/me", get(user_handlers::get_current_user))
        .route("/api/v1/users/logout", post(user_handlers::logout))
        // Token Revocation & Management
        .route("/api/v1/auth/revoke", post(token_handlers::revoke_current_token))
        .route("/api/v1/auth/revoke/:token_id", post(token_handlers::revoke_token_by_id))
        .route("/api/v1/auth/cleanup-tokens", post(token_handlers::cleanup_expired_tokens))
        .route("/api/v1/users/me/tokens", get(token_handlers::list_my_tokens))
        .route("/api/v1/users/:user_id/tokens", get(token_handlers::list_user_tokens))
        .route("/api/v1/tokens/rotate", post(token_handlers::rotate_token))
        .route("/api/v1/tokens/:token_id", delete(token_handlers::delete_token))
        // Role Management
        .route("/api/v1/roles", post(permission_handlers::create_role).get(permission_handlers::list_roles))
        .route("/api/v1/roles/:id", get(permission_handlers::get_role).put(permission_handlers::update_role).delete(permission_handlers::delete_role))
        // Permission Management
        .route("/api/v1/permissions", post(permission_handlers::grant_permission).get(permission_handlers::list_permissions))
        .route("/api/v1/permissions/:id", delete(permission_handlers::revoke_permission))
        // User Role Assignment
        .route("/api/v1/users/:id/roles", post(permission_handlers::assign_role).get(permission_handlers::get_user_roles))
        .route("/api/v1/users/:id/roles/:role_id", delete(permission_handlers::revoke_role))
        // .route("/api/v1/users/:user_id/permissions", get(permission_handlers::list_user_permissions))
        // Service User Management
        .route("/api/v1/service-users", get(service_user_handlers::list_service_users).post(service_user_handlers::create_service_user))
        .route("/api/v1/service-users/:id", get(service_user_handlers::get_service_user).put(service_user_handlers::update_service_user).delete(service_user_handlers::delete_service_user))
        .route("/api/v1/service-users/:id/rotate", post(service_user_handlers::rotate_api_key))
        // Token Generation
        .route("/api/v1/tokens", post(token_handlers::generate_token))
        // OAuth
        .route("/oauth/authorize/:provider", get(oauth_handlers::oauth_authorize))
        .route("/oauth/callback/:provider", get(oauth_handlers::oauth_callback))
        // Business Metadata (commented out - handlers not yet fully implemented)
        .route("/api/v1/business-metadata/:asset_id", get(business_metadata_handlers::get_business_metadata).delete(business_metadata_handlers::delete_business_metadata))
        // .route("/api/v1/business-metadata/:asset_id", put(business_metadata_handlers::upsert_business_metadata))
        // Access Requests - DUPLICATE REMOVED (already defined on line 107-108)
        // .route("/api/v1/access-requests", get(business_metadata_handlers::list_access_requests))
        // .route("/api/v1/access-requests", post(business_metadata_handlers::create_access_request))
        // .route("/api/v1/access-requests/:id", get(business_metadata_handlers::get_access_request).put(business_metadata_handlers::update_access_request))
        // Dashboard & Statistics
        .route("/api/v1/dashboard/stats", get(dashboard_handlers::get_dashboard_stats))
        .route("/api/v1/catalogs/:name/summary", get(dashboard_handlers::get_catalog_summary))
        // Search & Filter
        .route("/api/v1/search", get(optimization_handlers::unified_search))
        .route("/api/v1/search/assets", get(optimization_handlers::search_assets_by_name))
        // Bulk Operations
        .route("/api/v1/bulk/assets/delete", post(optimization_handlers::bulk_delete_assets))
        // Validation
        .route("/api/v1/validate/names", post(optimization_handlers::validate_names))
        .layer(axum::middleware::from_fn(auth_middleware::auth_middleware_wrapper))
        .layer(cors)
        .with_state(store)
}
