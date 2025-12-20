# Multi-Tenancy

Pangolin is designed from the ground up as a multi-tenant platform, providing strong isolation between different organizations (tenants) while sharing the same underlying infrastructure.

## Core Hierarchy

The hierarchy in Pangolin follows a strict tenant-first model:

1.  **Tenant**: The root isolation unit (e.g., "Acme Corp").
2.  **Warehouse**: Storage configurations (S3, Azure, GCS) belong to a tenant.
3.  **Catalog**: Data catalogs (Iceberg) belong to a tenant and reference a warehouse.
4.  **User**: Users are scoped to a tenant (except for the global Root user).

## Isolation Levels

### 1. Data Isolation
Each tenant typically has its own **Warehouse**. This ensures that data files for different tenants are stored in separate S3 buckets or prefixes, preventing accidental cross-tenant data access.

### 2. Identity Isolation
Users created within a tenant cannot see or access resources in another tenant unless explicitly granted cross-tenant permissions. Each tenant can manage its own set of users and service accounts.

### 3. Namespace Isolation
Identifiers (Catalogs, Namespaces, Assets) are scoped to the tenant. Two different tenants can have a catalog named `production` without any conflict.

## Tenant Management

Multi-tenancy is managed by the **Root** user (Platform Administrator).

- **Root User (Platform Admin)**: Global administrator with full access to all tenants. Responsible for tenant creation, listing, and deletion, as well as global system settings.
- **Tenant Admin**: A highly privileged user within a specific tenant. Capable of managing all resources within their tenant, including users, service users, warehouses, catalogs, and permissions.
- **Tenant User**: A standard user within a tenant, with access dictated by assigned RBAC policies.

## Configuration

When using the API or CLI, the tenant context is derived from:
- **JWT Claims**: For standard user authentication.
- **API Key Context**: For service users.
- **X-Pangolin-Tenant Header**: Required for certain administrative operations or when using PyIceberg with cross-tenant access.

For more details on managing tenants, see the **[CLI: Admin Tenants](../cli/admin-tenants.md)** guide.
