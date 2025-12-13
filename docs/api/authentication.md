# Authentication & Security

Pangolin implements a secure authentication system based on JSON Web Tokens (JWT) and Role-Based Access Control (RBAC).

## Authentication Flow

1.  **Login**: Users authenticate against the `/api/v1/login` endpoint using their credentials.
2.  **Token Issuance**: Upon successful authentication, the server returns a signed JWT.
3.  **Authenticated Requests**: Clients must include this JWT in the `Authorization` header of subsequent requests:
    ```
    Authorization: Bearer <token>
    ```

## Root User

A "Root User" is configured via environment variables to bootstrap the system. This user has full administrative privileges.

-   `PANGOLIN_ROOT_USER`: Username for the root user.
-   `PANGOLIN_ROOT_PASSWORD`: Password for the root user.

## Roles & Permissions

Pangolin supports the following roles:

-   **Root**: Full system access. Can manage tenants, users, and system configuration.
-   **Admin**: Tenant-level administration. Can manage warehouses, catalogs, and users within a tenant.
-   **User**: Standard access. Can read/write data based on catalog permissions.
-   **Viewer**: Read-only access to data.

## Middleware

The API is protected by authentication middleware that:
1.  Validates the JWT signature.
2.  Extracts user claims (ID, Role, Tenant).
3.  Enforces role-based access policies for specific endpoints.

## S3 Credential Vending

For direct S3 access (e.g., from engines like Spark or Trino), Pangolin provides a credential vending mechanism. See [Security & Vending](security_vending.md) for details.
