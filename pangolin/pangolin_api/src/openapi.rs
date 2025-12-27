use utoipa::OpenApi;
use utoipa::openapi::security::{SecurityScheme, HttpAuthScheme, HttpBuilder};

// Import all handler modules to access their path annotations
use crate::tenant_handlers;
use crate::warehouse_handlers;
use crate::pangolin_handlers;
use crate::user_handlers;
use crate::token_handlers;
use crate::permission_handlers;
use crate::federated_catalog_handlers;
use crate::service_user_handlers;
use crate::oauth_handlers;
use crate::merge_handlers;
use crate::business_metadata_handlers;
use crate::audit_handlers;
use crate::system_config_handlers;
use crate::iceberg;
use crate::signing_handlers;
use crate::asset_handlers;
use crate::dashboard_handlers;
use crate::optimization_handlers;

// Import all schema types
use pangolin_core::model::{
    Tenant, TenantUpdate, Warehouse, WarehouseUpdate, VendingStrategy,
    Catalog, CatalogUpdate, CatalogType, FederatedCatalogConfig,
    MergeOperation, MergeConflict, ConflictResolution, ResolutionStrategy, MergeStatus, ConflictType,
};
use pangolin_core::iceberg_metadata::{
    TableMetadata, Schema as IcebergSchema, NestedField, Type as IcebergType, PartitionSpec, PartitionField,
    SortOrder, SortField, Snapshot, SnapshotLogEntry, MetadataLogEntry,
};
use pangolin_core::audit::{AuditLogEntry, AuditAction, ResourceType, AuditResult};
use pangolin_core::user::{User, UserRole, UserSession, ServiceUser, OAuthProvider, ApiKeyResponse};
use pangolin_core::permission::{Permission, PermissionScope, Action, Role, PermissionGrant};
use pangolin_core::business_metadata::{BusinessMetadata, AccessRequest, RequestStatus};

#[derive(OpenApi)]
#[openapi(
    paths(
        // Tenant endpoints
        tenant_handlers::list_tenants,
        tenant_handlers::create_tenant,
        tenant_handlers::get_tenant,
        tenant_handlers::update_tenant,
        tenant_handlers::delete_tenant,
        
        // Warehouse endpoints
        warehouse_handlers::list_warehouses,
        warehouse_handlers::create_warehouse,
        warehouse_handlers::get_warehouse,
        warehouse_handlers::update_warehouse,
        warehouse_handlers::delete_warehouse,
        
        // Catalog endpoints
        pangolin_handlers::list_catalogs,
        pangolin_handlers::create_catalog,
        pangolin_handlers::get_catalog,
        pangolin_handlers::update_catalog,
        pangolin_handlers::delete_catalog,
        
        // User endpoints
        user_handlers::create_user,
        user_handlers::list_users,
        user_handlers::get_user,
        user_handlers::update_user,
        user_handlers::delete_user,
        user_handlers::login,
        user_handlers::logout,
        user_handlers::get_current_user,
        user_handlers::get_app_config,
        
        // Token endpoints
        token_handlers::generate_token,
        token_handlers::revoke_current_token,
        token_handlers::revoke_token_by_id,
        token_handlers::list_my_tokens,
        token_handlers::list_user_tokens,
        token_handlers::cleanup_expired_tokens,
        token_handlers::rotate_token,
        token_handlers::delete_token,
        
        // Role & Permission endpoints
        permission_handlers::assign_role,
        permission_handlers::get_user_roles,
        permission_handlers::revoke_role,
        permission_handlers::grant_permission,
        permission_handlers::revoke_permission,
        permission_handlers::list_permissions,
        permission_handlers::create_role,
        permission_handlers::list_roles,
        permission_handlers::get_role,
        permission_handlers::update_role,
        permission_handlers::delete_role,
        
        // Federated catalog endpoints
        federated_catalog_handlers::list_federated_catalogs,
        federated_catalog_handlers::create_federated_catalog,
        federated_catalog_handlers::get_federated_catalog,
        federated_catalog_handlers::delete_federated_catalog,
        federated_catalog_handlers::sync_federated_catalog,
        federated_catalog_handlers::test_federated_connection,
        federated_catalog_handlers::get_federated_catalog_stats,
        
        // Service user endpoints
        service_user_handlers::list_service_users,
        service_user_handlers::create_service_user,
        service_user_handlers::get_service_user,
        service_user_handlers::update_service_user,
        service_user_handlers::delete_service_user,
        service_user_handlers::rotate_api_key,
        
        // OAuth endpoints
        oauth_handlers::oauth_authorize,
        oauth_handlers::oauth_callback,
        
        // Branch endpoints
        pangolin_handlers::list_branches,
        pangolin_handlers::create_branch,
        pangolin_handlers::get_branch,
        pangolin_handlers::merge_branch,
        pangolin_handlers::rebase_branch,
        
        // Tag endpoints
        pangolin_handlers::list_tags,
        pangolin_handlers::create_tag,
        pangolin_handlers::delete_tag,
        
        // Merge operation endpoints
        merge_handlers::list_merge_operations,
        merge_handlers::get_merge_operation,
        merge_handlers::list_merge_conflicts,
        merge_handlers::complete_merge,
        merge_handlers::abort_merge,
        merge_handlers::resolve_conflict,
        
        // Business metadata endpoints
        business_metadata_handlers::get_business_metadata,
        business_metadata_handlers::delete_business_metadata,
        business_metadata_handlers::search_assets,
        business_metadata_handlers::get_asset_details,
        business_metadata_handlers::request_access,
        business_metadata_handlers::list_access_requests,
        business_metadata_handlers::get_access_request,
        business_metadata_handlers::update_access_request,
        
        // Audit logging endpoints
        audit_handlers::list_audit_events,
        audit_handlers::get_audit_event,
        audit_handlers::count_audit_events,
        
        // Dashboard endpoints
        dashboard_handlers::get_dashboard_stats,
        dashboard_handlers::get_catalog_summary,
        
        // Optimization endpoints
        optimization_handlers::search_assets_by_name,
        optimization_handlers::bulk_delete_assets,
        optimization_handlers::validate_names,
        optimization_handlers::unified_search,
        
        // System Config
        system_config_handlers::get_system_settings,
        system_config_handlers::update_system_settings,

        // Iceberg REST endpoints
        iceberg::namespaces::list_namespaces,
        iceberg::config::get_iceberg_catalog_config_handler,
        iceberg::namespaces::create_namespace,
        iceberg::tables::list_tables,
        iceberg::tables::create_table,
        iceberg::tables::load_table,
        iceberg::tables::update_table,
        iceberg::tables::rename_table,
        iceberg::tables::delete_table,
        iceberg::namespaces::delete_namespace,
        iceberg::namespaces::update_namespace_properties,
        iceberg::tables::report_metrics,
        iceberg::tables::perform_maintenance,
        iceberg::tables::table_exists,
        iceberg::namespaces::list_namespaces_tree,
        
        // Iceberg OAuth
        iceberg::oauth::handle_oauth_token,

        // Signing / Credential Vending
        signing_handlers::get_table_credentials,
        signing_handlers::get_presigned_url,

        // Views
        asset_handlers::create_view,
        asset_handlers::get_view,
        
        // Generic Assets
        asset_handlers::register_asset,
        asset_handlers::list_assets,
    ),
    components(
        schemas(
            // Core models
            Tenant, TenantUpdate,
            Warehouse, WarehouseUpdate, VendingStrategy,
            Catalog, CatalogUpdate, CatalogType,
            FederatedCatalogConfig,
            
            // User models
            User, UserRole, UserSession, ServiceUser, OAuthProvider, ApiKeyResponse,
            
            // Permission models
            Permission, PermissionScope, Action, Role, PermissionGrant,
            
            // Request/Response types
            tenant_handlers::CreateTenantRequest, tenant_handlers::TenantResponse,
            warehouse_handlers::CreateWarehouseRequest, warehouse_handlers::UpdateWarehouseRequest, warehouse_handlers::WarehouseResponse,
            pangolin_handlers::CreateCatalogRequest, pangolin_handlers::UpdateCatalogRequest, pangolin_handlers::CatalogResponse,
            user_handlers::CreateUserRequest, user_handlers::UpdateUserRequest, user_handlers::LoginRequest, user_handlers::LoginResponse, user_handlers::UserInfo,
            token_handlers::GenerateTokenRequest, token_handlers::GenerateTokenResponse,
            token_handlers::RevokeTokenRequest, token_handlers::RevokeTokenResponse,
            permission_handlers::CreateRoleRequest, permission_handlers::AssignRoleRequest, permission_handlers::GrantPermissionRequest,
            federated_catalog_handlers::CreateFederatedCatalogRequest, federated_catalog_handlers::FederatedCatalogResponse,
            service_user_handlers::CreateServiceUserRequest, service_user_handlers::UpdateServiceUserRequest,
            oauth_handlers::OAuthCallback, oauth_handlers::AuthorizeParams,
            
            // Iceberg OAuth types
            iceberg::oauth::OAuthTokenRequest, iceberg::oauth::OAuthTokenResponse,
            
            // Branch/Tag/Merge types
            pangolin_handlers::CreateBranchRequest, pangolin_handlers::ListBranchParams, pangolin_handlers::MergeBranchRequest, pangolin_handlers::BranchResponse,
            merge_handlers::ResolveConflictRequest, merge_handlers::MergeOperationResponse,
            MergeOperation, MergeConflict, ConflictResolution, ResolutionStrategy, MergeStatus, ConflictType,
            
            // Business metadata types
            business_metadata_handlers::AddMetadataRequest, business_metadata_handlers::MetadataResponse, business_metadata_handlers::SearchRequest,
            business_metadata_handlers::CreateAccessRequestPayload, business_metadata_handlers::UpdateRequestStatus,
            BusinessMetadata, AccessRequest, RequestStatus,
            
            // Audit logging types
            audit_handlers::AuditListQuery, audit_handlers::AuditCountResponse,
            
            // Dashboard types
            dashboard_handlers::DashboardStats, dashboard_handlers::CatalogSummary,
            
            // Optimization types
            optimization_handlers::SearchQuery, optimization_handlers::AssetSearchResult, optimization_handlers::SearchResponse,
            optimization_handlers::BulkDeleteAssetsRequest, optimization_handlers::BulkOperationResponse,
            optimization_handlers::ValidateNamesRequest, optimization_handlers::NameValidationResult, optimization_handlers::ValidateNamesResponse,
            optimization_handlers::UnifiedSearchResult, optimization_handlers::UnifiedSearchResponse, optimization_handlers::UnifiedSearchQuery, optimization_handlers::SearchResultType,
            
            // Iceberg/Data models
            iceberg::types::ListNamespacesResponse, iceberg::types::CreateNamespaceRequest, iceberg::types::CreateNamespaceResponse,
            iceberg::types::CreateTableRequest, iceberg::types::TableResponse, iceberg::types::ListTablesResponse, iceberg::types::TableIdentifier,
            iceberg::types::CommitTableRequest, iceberg::types::CommitRequirement, iceberg::types::CommitUpdate, iceberg::types::RenameTableRequest,
            iceberg::types::UpdateNamespacePropertiesRequest, iceberg::types::UpdateNamespacePropertiesResponse,
            iceberg::types::NamespaceNode, iceberg::types::ListNamespacesTreeResponse,
            iceberg::tables::MaintenanceRequest,
            
            // Signing/Vending models
            signing_handlers::StorageCredential, signing_handlers::LoadCredentialsResponse, signing_handlers::PresignResponse, signing_handlers::PresignParams,
            
            // View models
            asset_handlers::CreateViewRequest, asset_handlers::ViewResponse,
            
            // Generic Asset models
            asset_handlers::RegisterAssetRequest, asset_handlers::RegisterAssetResponse, asset_handlers::AssetSummary,
            
            // App models
            user_handlers::AppConfig,
            
            // Core Iceberg models (from pangolin-core)
            TableMetadata, IcebergSchema, NestedField, IcebergType, PartitionSpec, PartitionField,
            SortOrder, SortField, Snapshot, SnapshotLogEntry, MetadataLogEntry, 
            AuditLogEntry, AuditAction, ResourceType, AuditResult,
        )
    ),
    tags(
        (name = "Tenants", description = "Tenant management endpoints"),
        (name = "Warehouses", description = "Warehouse management endpoints"),
        (name = "Catalogs", description = "Catalog management endpoints"),
        (name = "Users", description = "User management and authentication endpoints"),
        (name = "Tokens", description = "Token generation and revocation endpoints"),
        (name = "Roles & Permissions", description = "Role and permission management endpoints"),
        (name = "Federated Catalogs", description = "Federated catalog management endpoints"),
        (name = "Service Users", description = "Service user and API key management endpoints"),
        (name = "OAuth", description = "OAuth authentication endpoints"),
        (name = "Branches", description = "Branch management endpoints"),
        (name = "Tags", description = "Tag management endpoints"),
        (name = "Merge Operations", description = "Merge operation and conflict resolution endpoints"),
        (name = "Business Metadata", description = "Business metadata and asset discovery endpoints"),
        (name = "Audit Logging", description = "Audit log viewing and filtering endpoints"),
        (name = "System Config", description = "System configuration management endpoints"),
        (name = "Iceberg REST", description = "Iceberg REST Catalog endpoints"),
        (name = "Assets", description = "Generic asset registration and management"),
        (name = "Data Explorer", description = "Data exploration and discovery endpoints"),
        (name = "Dashboard", description = "Dashboard statistics and summaries"),
        (name = "Search", description = "Asset search and discovery"),
        (name = "Bulk Operations", description = "Bulk operations for efficiency"),
        (name = "Validation", description = "Resource name validation"),
    ),
    modifiers(&SecurityAddon),
    info(
        title = "Pangolin Catalog API",
        version = "1.0.0",
        description = "Multi-tenant Apache Iceberg REST Catalog with advanced features including federated catalogs, credential vending, and fine-grained permissions",
        contact(
            name = "Pangolin API Support",
            email = "support@pangolin.dev"
        ),
        license(
            name = "Apache 2.0",
            url = "https://www.apache.org/licenses/LICENSE-2.0.html"
        )
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some("JWT token obtained from /api/v1/users/login or /api/v1/tokens"))
                        .build()
                ),
            )
        }
    }
}
