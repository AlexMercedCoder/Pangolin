use utoipa::OpenApi;
use utoipa::openapi::security::{SecurityScheme, HttpAuthScheme, HttpBuilder};

// Import all handler modules to access their path annotations
use crate::tenant_handlers::*;
use crate::warehouse_handlers::*;
use crate::pangolin_handlers::*;
use crate::user_handlers::*;
use crate::token_handlers::*;
use crate::permission_handlers::*;
use crate::federated_catalog_handlers::*;
use crate::service_user_handlers::*;
use crate::oauth_handlers::*;
use crate::merge_handlers::*;
use crate::business_metadata_handlers::*;
use crate::audit_handlers::*;
use crate::system_config_handlers::*;
use crate::iceberg_handlers::*;

// Import all schema types
use pangolin_core::model::{
    Tenant, TenantUpdate, Warehouse, WarehouseUpdate, VendingStrategy,
    Catalog, CatalogUpdate, CatalogType, FederatedCatalogConfig,
    FederatedAuthType, FederatedCredentials,
    MergeOperation, MergeConflict, ConflictResolution, ResolutionStrategy, MergeStatus, ConflictType,
};
use pangolin_core::user::{User, UserRole, UserSession, ServiceUser, OAuthProvider, ApiKeyResponse};
use pangolin_core::permission::{Permission, PermissionScope, Action, Role, PermissionGrant};
use pangolin_core::business_metadata::{BusinessMetadata, AccessRequest, RequestStatus};

#[derive(OpenApi)]
#[openapi(
    paths(
        // Tenant endpoints
        list_tenants,
        create_tenant,
        get_tenant,
        update_tenant,
        delete_tenant,
        
        // Warehouse endpoints
        list_warehouses,
        create_warehouse,
        get_warehouse,
        update_warehouse,
        delete_warehouse,
        
        // Catalog endpoints
        list_catalogs,
        create_catalog,
        get_catalog,
        update_catalog,
        delete_catalog,
        
        // User endpoints
        create_user,
        list_users,
        get_user,
        update_user,
        delete_user,
        login,
        
        // Token endpoints
        generate_token,
        revoke_current_token,
        revoke_token_by_id,
        list_user_tokens,
        delete_token,
        
        // System Configuration endpoints
        get_system_settings,
        update_system_settings,
        
        // Role endpoints
        create_role,
        list_roles,
        get_role,
        update_role,
        delete_role,
        
        // Permission endpoints
        grant_permission,
        revoke_permission,
        list_permissions,
        
        // Federated catalog endpoints
        create_federated_catalog,
        list_federated_catalogs,
        get_federated_catalog,
        delete_federated_catalog,
        test_federated_connection,
        sync_federated_catalog,
        get_federated_catalog_stats,
        
        // Data Explorer endpoints
        list_namespaces_tree,
        
        // Service user endpoints
        create_service_user,
        list_service_users,
        get_service_user,
        update_service_user,
        delete_service_user,
        rotate_api_key,
        
        // OAuth endpoints
        oauth_authorize,
        oauth_callback,
        
        // Branch endpoints
        list_branches,
        create_branch,
        get_branch,
        merge_branch,
        list_commits,
        
        // Tag endpoints
        create_tag,
        list_tags,
        delete_tag,
        
        // Merge operation endpoints
        list_merge_operations,
        get_merge_operation,
        list_merge_conflicts,
        resolve_conflict,
        complete_merge,
        abort_merge,
        
        // Business metadata endpoints
        add_business_metadata,
        get_business_metadata,
        delete_business_metadata,
        search_assets,
        request_access,
        list_access_requests,
        update_access_request,
        get_access_request,
        get_asset_details,
        
        // Audit logging endpoints
        list_audit_events,
        get_audit_event,
        count_audit_events,
    ),
    components(
        schemas(
            // Core models
            Tenant, TenantUpdate,
            Warehouse, WarehouseUpdate, VendingStrategy,
            Catalog, CatalogUpdate, CatalogType,
            FederatedCatalogConfig, FederatedAuthType, FederatedCredentials,
            
            // User models
            User, UserRole, UserSession, ServiceUser, OAuthProvider, ApiKeyResponse,
            
            // Permission models
            Permission, PermissionScope, Action, Role,
            
            // Request/Response types
            CreateTenantRequest, TenantResponse,
            CreateWarehouseRequest, UpdateWarehouseRequest, WarehouseResponse,
            CreateCatalogRequest, UpdateCatalogRequest, CatalogResponse,
            CreateUserRequest, UpdateUserRequest, LoginRequest, LoginResponse, UserInfo,
            GenerateTokenRequest, GenerateTokenResponse,
            RevokeTokenRequest, RevokeTokenResponse,
            CreateRoleRequest, AssignRoleRequest, GrantPermissionRequest,
            CreateFederatedCatalogRequest, FederatedCatalogResponse,
            CreateServiceUserRequest, UpdateServiceUserRequest,
            OAuthCallback, AuthorizeParams,
            
            // Branch/Tag/Merge types
            CreateBranchRequest, ListBranchParams, MergeBranchRequest, BranchResponse,
            ResolveConflictRequest, MergeOperationResponse,
            MergeOperation, MergeConflict, ConflictResolution, ResolutionStrategy, MergeStatus, ConflictType,
            
            // Business metadata types
            AddMetadataRequest, MetadataResponse, SearchRequest,
            CreateAccessRequestPayload, UpdateRequestStatus,
            BusinessMetadata, AccessRequest, RequestStatus,
            
            // Audit logging types
            AuditListQuery, AuditCountResponse,
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
        (name = "Data Explorer", description = "Data exploration and discovery endpoints"),
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
