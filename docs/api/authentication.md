# Authentication & Security

Pangolin implements a secure authentication system based on JSON Web Tokens (JWT) and Role-Based Access Control (RBAC).

## Authentication Flow

1.  **Login**: Users authenticate against the `/api/v1/users/login` endpoint using their credentials.
    - **Root Login**: Omit `tenant-id` or set to `null`
    - **Tenant-Scoped Login**: Include `tenant-id` with tenant UUID
2.  **Token Issuance**: Upon successful authentication, the server returns a signed JWT.
3.  **Authenticated Requests**: Clients must include this JWT in the `Authorization` header of subsequent requests:
    ```
    Authorization: Bearer <token>
    ```

### Login Examples

**Root Login**:
```bash
curl -X POST http://localhost:8080/api/v1/users/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"password","tenant-id":null}'
```

**Tenant-Scoped Login** (for users with duplicate usernames across tenants):
```bash
curl -X POST http://localhost:8080/api/v1/users/login \
  -H "Content-Type: application/json" \
  -d '{"username":"user","password":"pass123","tenant-id":"<tenant-uuid>"}'
```

> [!IMPORTANT]
> Use `tenant-id` (kebab-case), not `tenant_id` (underscore).

## Root User

A "Root User" is configured via environment variables to bootstrap the system. This user has full administrative privileges.

-   `PANGOLIN_ROOT_USER`: Username for the root user.
-   `PANGOLIN_ROOT_PASSWORD`: Password for the root user.

## Roles & Permissions

Pangolin supports the following roles:

-   **Root**: Full system access. Can manage tenants, users, and system configuration.
-   **TenantAdmin**: Tenant-level administration. Can manage warehouses, catalogs, and users within a tenant.
-   **TenantUser**: Standard access. Can read/write data based on catalog permissions.

## Token Generation

For automation and scripts, users can generate long-lived JWT tokens:

```bash
curl -X POST http://localhost:8080/api/v1/tokens \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_id": "your-tenant-id",
    "username": "your-username",
    "expires_in_hours": 720
  }'
```

## Token Revocation

Tokens can be revoked for security:

- **Self-revoke**: `POST /api/v1/auth/revoke` - Revoke your current token
- **Admin-revoke**: `POST /api/v1/auth/revoke/{token_id}` - Revoke any token (admin only)

## OAuth Authentication

Pangolin supports OAuth providers (Google, GitHub, etc.):

1. Navigate to `/oauth/authorize/{provider}` (e.g., `/oauth/authorize/google`)
2. Complete OAuth flow with provider
3. Receive JWT token upon successful authentication

## Middleware

The API is protected by authentication middleware that:
1.  Validates the JWT signature.
2.  Extracts user claims (ID, Role, Tenant).
3.  Enforces role-based access policies for specific endpoints.

## S3 Credential Vending

For direct S3 access (e.g., from engines like Spark or Trino), Pangolin provides a credential vending mechanism. See [Security & Vending](../features/security_vending.md) for details.
