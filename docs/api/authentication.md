# Authentication & Security

Pangolin implements a secure authentication system based on JSON Web Tokens (JWT) and Role-Based Access Control (RBAC).

## Authentication Flow

1.  **Login**: Users authenticate against the `/api/v1/users/login` endpoint using their credentials.
    - **Root Login**: Omit `tenant-id` or set to `null`
    - **Tenant-Scoped Login**: Include `tenant-id` with tenant UUID
2.  **Token Issuance**: Upon successful authentication, the server returns a signed JWT.
3.  **API Key Authentication (Service Users)**: Machine accounts use a static API key passed in the `X-API-Key` header.
    ```
    X-API-Key: <your-api-key>
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

## API Key Authentication (Service Users)

Service users are intended for machine-to-machine communication (CI/CD, automated scripts). They do not use JWT tokens; instead, they use a persistent API key.

```bash
curl http://localhost:8080/api/v1/catalogs \
  -H "X-API-Key: pgl_key_abc123..." \
  -H "X-Pangolin-Tenant: <tenant-uuid>"
```

> [!TIP]
> API keys are only displayed once upon creation. If lost, the key must be rotated via the `/api/v1/service-users/{id}/rotate` endpoint.

## Token Generation (Users)

For temporary programmatic access by human users, you can generate long-lived JWT tokens:

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
